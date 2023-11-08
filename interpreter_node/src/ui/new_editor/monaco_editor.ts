import debounce from "debounce";
import * as monaco from "monaco-editor";
import { createConfiguredEditor, createModelReference } from 'vscode/monaco'
import { RegisteredFileSystemProvider, registerFileSystemOverlay, RegisteredMemoryFile } from '@codingame/monaco-vscode-files-service-override';
import * as proto from "vscode-languageserver-protocol";
import * as vscode from "vscode";

import Client from "./client";
import { FromServer, IntoServer } from "./codec";
import Language from "./language";
import Server from "./server";
import { buildWorkerDefinition } from 'monaco-editor-workers';
buildWorkerDefinition('../../../node_modules/monaco-editor-workers/dist/workers', new URL('', window.location.href).href, false);

// class Environment implements monaco.Environment {
//   getWorkerUrl(moduleId: string, label: string) {
//     if (label === "editorWorkerService") {
//       return "./editor.worker.bundle.js";
//     }
//     throw new Error(`getWorkerUrl: unexpected ${JSON.stringify({ moduleId, label })}`);
//   }
// }

export default class MonacoEditor {
  // readonly #window: Window & monaco.Window & typeof globalThis = self;

  // initializeMonaco(): void {
  //   this.#window.MonacoEnvironment = new Environment();
  // }

  async createModel(client: Client, content: string): Promise<monaco.editor.ITextModel> {
    const language = Language.initialize(client);

    const fileSystemProvider = new RegisteredFileSystemProvider(false);
    fileSystemProvider.registerFile(new RegisteredMemoryFile(vscode.Uri.file('/workspace/demo.rg'), 'print("Hello, World!")'));
    registerFileSystemOverlay(1, fileSystemProvider);

    const id = language.id;
    const uri = monaco.Uri.parse("/workspace/demo.rg");
    const model = monaco.editor.createModel(content, id, uri);
    monaco.editor.defineTheme('myCustomTheme', {
      base: 'vs',
      inherit: true,
      colors: {},
      rules: [
        {
          token: "comment",
          foreground: "6a737d",
          fontStyle: "italic"
        },
        {
          token: "keyword",
          foreground: "0000ff",
          fontStyle: "bold"
        },
        {
          token: "type",
          foreground: "2b91af"
        },
        {
          token: "member",
          foreground: "000000"
        },
        {
          token: "constant",
          foreground: "c5060b",
        },
        {
          token: "variable",
          foreground: "005cc5"
        },
        {
          token: "function",
          foreground: "986801"
        },
        {
          token: "macro",
          // dark red foreground
          foreground: "ff0000"
        }

      ],

    });
    // const modelRef = await createModelReference(uri);
    // const model = modelRef.object.textEditorModel!;

    // modelRef.object.setLanguageId(id);


    model.onDidChangeContent(
      debounce(() => {
        console.log("onDidChangeContent");
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

    client.addMethod(proto.PublishDiagnosticsNotification.type.method, (params) => {
      const { uri, diagnostics } = params as proto.PublishDiagnosticsParams;
      console.log(`[diagnostics] ${uri}`);
      diagnostics.forEach((diagnostic) => {
        console.log(`  ${diagnostic.message}`);
      });
      const diags = diagnostics.map((diagnostic) => {
        // We have to map range to Monaco editor
        return {
          severity: monaco.MarkerSeverity.Error,
          message: diagnostic.message,
          startLineNumber: diagnostic.range.start.line + 1,
          startColumn: diagnostic.range.start.character + 1,
          endLineNumber: diagnostic.range.end.line + 1,
          endColumn: diagnostic.range.end.character + 1,
        };
      })
      monaco.editor.setModelMarkers(model, "rg", diags)

      return;
    });


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

  async createEditor(client: Client, container: HTMLElement, content: string): Promise<void> {
    console.log("initialize monaco");
    // this.initializeMonaco();
    const model = await this.createModel(client, content);
    createConfiguredEditor(container, {
      model: model,
      automaticLayout: true,
      "semanticHighlighting.enabled": true,
      theme: 'myCustomTheme',
    })
  }

}
