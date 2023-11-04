import debounce from "debounce";
import * as monaco from "monaco-editor";
import * as proto from "vscode-languageserver-protocol";
import * as vscode from "vscode";

import Client from "./client";
import { FromServer, IntoServer } from "./codec";
import Language from "./language";
import Server from "./server";

class Environment implements monaco.Environment {
  getWorkerUrl(moduleId: string, label: string) {
    if (label === "editorWorkerService") {
      return "./editor.worker.bundle.js";
    }
    throw new Error(`getWorkerUrl: unexpected ${JSON.stringify({ moduleId, label })}`);
  }
}

export default class MonacoEditor {
  readonly #window: Window & monaco.Window & typeof globalThis = self;

  initializeMonaco(): void {
    this.#window.MonacoEnvironment = new Environment();
  }

  createModel(client: Client, content: string): monaco.editor.ITextModel {
    const language = Language.initialize(client);

    const id = language.id;
    const uri = monaco.Uri.parse("inmemory://demo.rg");

    const model = monaco.editor.createModel(content, id, uri);

    model.onDidChangeContent(
      debounce(() => {
        const text = model.getValue();
        client.notify(proto.DidChangeTextDocumentNotification.type.method, {
          textDocument: {
            version: 0,
            uri: model.uri.toString(),
          },
          contentChanges: [
            {
              range: this.asRange(model.getFullModelRange()),
              text,
            },
          ],
        } as proto.DidChangeTextDocumentParams);
      }, 200),
    );

    // eslint-disable-next-line @typescript-eslint/require-await
    client.pushAfterInitializeHook(async () => {
      client.notify(proto.DidOpenTextDocumentNotification.type.method, {
        textDocument: {
          uri: model.uri.toString(),
          languageId: language.id,
          version: 0,
          text: model.getValue(),
        },
      } as proto.DidOpenTextDocumentParams);
    });

    return model;
  }

  private asRange(range: monaco.IRange): proto.Range {
    return {
      start: { line: range.startLineNumber - 1, character: range.startColumn - 1 },
      end: { line: range.endLineNumber - 1, character: range.endColumn - 1 },
    };
  }

  createEditor(client: Client, container: HTMLElement, content: string): void {
    this.initializeMonaco();
    const model = this.createModel(client, content);
    monaco.editor.create(container, {
      model,
      automaticLayout: true,
    });
  }

}
