import debounce from "debounce";
import * as monaco from "monaco-editor";
import { createConfiguredEditor, createModelReference } from 'vscode/monaco'
import { RegisteredFileSystemProvider, registerFileSystemOverlay, RegisteredMemoryFile } from '@codingame/monaco-vscode-files-service-override';
import * as proto from "vscode-languageserver-protocol";
import * as vscode from "vscode";
import Client from "./client";
import Language from "./language";
import EditorWorker from 'url:monaco-editor/esm/vs/editor/editor.worker.js';

export default class MonacoEditor {
  readonly #window: Window & monaco.Window & typeof globalThis = self;

  initializeMonaco(): void {
    this.#window.MonacoEnvironment = {
      getWorkerUrl: function (moduleId, label) {
        return EditorWorker;
      }
    }
  }

  async createModel(client: Client, path: string, content: string, onChange: (source: string) => void): Promise<monaco.editor.ITextModel> {
    const extension = this.pathToExtension(path);
    const language = Language.initialize(client, extension);

    const fileSystemProvider = new RegisteredFileSystemProvider(false);
    fileSystemProvider.registerFile(new RegisteredMemoryFile(vscode.Uri.file('/workspace/' + path), ''));
    registerFileSystemOverlay(1, fileSystemProvider);

    const uri = monaco.Uri.parse("/workspace/" + path);
    const model = monaco.editor.createModel(content, extension, uri);
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

    if (extension === "rg") {
      // eslint-disable-next-line @typescript-eslint/require-await
      client.notify(proto.DidOpenTextDocumentNotification.type.method, {
        textDocument: {
          uri: model.uri.toString(),
          languageId: extension,
          version: 0,
          text: model.getValue(),
        },
      } as proto.DidOpenTextDocumentParams);

      model.onDidChangeContent(
        debounce(() => {
          const text = model.getValue();
          onChange(text);
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
        }, 300),
      );

      client.addMethod(proto.PublishDiagnosticsNotification.type.method, (params) => {
        const { uri, diagnostics } = params as proto.PublishDiagnosticsParams;
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


      model.onWillDispose(() =>
        client.notify(proto.DidCloseTextDocumentNotification.type.method, {
          textDocument: {
            uri: model.uri.toString(),
          },
        } as proto.DidCloseTextDocumentParams),
      )

    } else {
      model.onDidChangeContent(
        debounce(() => onChange(model.getValue()), 300)
      )
    }
    return model;
  }

  private asRange(range: monaco.IRange): proto.Range {
    return {
      start: { line: range.startLineNumber - 1, character: range.startColumn - 1 },
      end: { line: range.endLineNumber - 1, character: range.endColumn - 1 },
    };
  }

  private pathToExtension(path: string): string {
    const parts = path.split(".");
    return parts[parts.length - 1];
  }

  async createEditor(client: Client, container: HTMLElement, path: string, content: string, onChange: (source: string) => void): Promise<monaco.editor.IStandaloneCodeEditor> {
    this.initializeMonaco();
    const model = await this.createModel(client, path, content, onChange);
    const editor = createConfiguredEditor(container, {
      model: model,
      automaticLayout: true,
      "semanticHighlighting.enabled": true,
      theme: 'myCustomTheme',
    })
    return editor;
  }

}
