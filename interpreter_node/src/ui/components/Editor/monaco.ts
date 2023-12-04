import {
  RegisteredFileSystemProvider,
  registerFileSystemOverlay,
  RegisteredMemoryFile,
} from '@codingame/monaco-vscode-files-service-override';
import * as monaco from 'monaco-editor';
import EditorWorker from 'url:monaco-editor/esm/vs/editor/editor.worker.js';
import * as vscode from 'vscode';
import { createConfiguredEditor } from 'vscode/monaco';
import * as proto from 'vscode-languageserver-protocol';

import Client from './client';
import { Language } from '../../../types';

window.MonacoEnvironment = { getWorkerUrl: () => EditorWorker };

const fileSystemProvider = new RegisteredFileSystemProvider(false);
registerFileSystemOverlay(1, fileSystemProvider);

export function createModel(path: string, content: string) {
  const filePath = `/workspace/${path}`;
  const file = new RegisteredMemoryFile(vscode.Uri.file(filePath), '');
  fileSystemProvider.registerFile(file);

  const uri = monaco.Uri.parse(filePath);
  return monaco.editor.createModel(content, path.split('.').pop(), uri);
}

function asRange(range: monaco.IRange) {
  return {
    start: {
      character: range.startColumn - 1,
      line: range.startLineNumber - 1,
    },
    end: { character: range.endColumn - 1, line: range.endLineNumber - 1 },
  };
}

export function createEditor(
  client: Client,
  container: HTMLElement,
  onChange?: (source: string) => void,
  readOnly?: boolean,
): monaco.editor.IStandaloneCodeEditor {
  const editor = createConfiguredEditor(container, {
    'semanticHighlighting.enabled': true,
    automaticLayout: true,
    lightbulb: { enabled: false },
    readOnly,
    theme: 'rgTheme',
  });

  editor.onDidChangeModelContent(() => {
    const model = editor.getModel();

    if (!model) {
      return;
    }
    const text = model.getValue();
    onChange?.(text);

    if (model.getLanguageId() !== Language.rg) {
      return;
    }

    client.notify(proto.DidChangeTextDocumentNotification.type.method, {
      textDocument: {
        uri: model.uri.toString(),
        version: model.getVersionId(),
      },
      contentChanges: [{ range: asRange(model.getFullModelRange()), text }],
    });
  });

  editor.onDidChangeModel(event => {
    if (event.newModelUrl) {
      const model = editor.getModel();
      if (!model || model.getLanguageId() !== Language.rg) {
        return;
      }

      client.notify(proto.DidOpenTextDocumentNotification.type.method, {
        textDocument: {
          uri: model.uri.toString(),
          languageId: model.getLanguageId(),
          version: model.getVersionId(),
          text: model.getValue(),
        },
      });
    }

    if (event.oldModelUrl?.path.endsWith('rg')) {
      client.notify(proto.DidCloseTextDocumentNotification.type.method, {
        textDocument: { uri: event.oldModelUrl.toString() },
      });
    }
  });

  client.addMethod(proto.PublishDiagnosticsNotification.type.method, params => {
    const { uri, diagnostics } = params as proto.PublishDiagnosticsParams;
    // Map range to Monaco editor.
    const diags = diagnostics.map(diagnostic => ({
      endColumn: diagnostic.range.end.character + 1,
      endLineNumber: diagnostic.range.end.line + 1,
      message: diagnostic.message,
      severity: monaco.MarkerSeverity.Error,
      startColumn: diagnostic.range.start.character + 1,
      startLineNumber: diagnostic.range.start.line + 1,
    }));

    const model = monaco.editor.getModel(monaco.Uri.parse(uri));
    if (model?.getLanguageId() === Language.rg) {
      monaco.editor.setModelMarkers(model, Language.rg, diags);
    }
  });

  return editor;
}
