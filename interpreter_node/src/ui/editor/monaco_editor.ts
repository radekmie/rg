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
import { LanguageID } from '../../types';

window.MonacoEnvironment = {
  getWorkerUrl(moduleId, label) {
    moduleId;
    label;
    return EditorWorker;
  },
};

const fileSystemProvider = new RegisteredFileSystemProvider(false);
registerFileSystemOverlay(1, fileSystemProvider);

export function createModel(
  path: string,
  content: string,
): monaco.editor.ITextModel {
  const lang = pathToLang(path);
  fileSystemProvider.registerFile(
    new RegisteredMemoryFile(vscode.Uri.file('/workspace/' + path), ''),
  );
  const uri = monaco.Uri.parse('/workspace/' + path);
  const model = monaco.editor.createModel(content, lang, uri);
  return model;
}

const asRange = (range: monaco.IRange): proto.Range => {
  return {
    start: {
      line: range.startLineNumber - 1,
      character: range.startColumn - 1,
    },
    end: { line: range.endLineNumber - 1, character: range.endColumn - 1 },
  };
};

const pathToLang = (path: string): string | undefined => {
  const extension = path.split('.').pop();
  switch (extension) {
    case 'rg':
      return LanguageID.rg;
    case 'rbg':
      return LanguageID.rbg;
    case 'hrg':
      return LanguageID.hrg;
    default:
      return extension;
  }
};

export function createEditor(
  client: Client,
  container: HTMLElement,
  onChange: (source: string) => void,
  readonly: boolean,
): monaco.editor.IStandaloneCodeEditor {
  const editor = createConfiguredEditor(container, {
    automaticLayout: true,
    'semanticHighlighting.enabled': true,
    theme: 'rgTheme',
    readOnly: readonly,
    lightbulb: {
      enabled: false,
    },
  });

  editor.onDidChangeModelContent(() => {
    const model = editor.getModel();
    if (model?.getLanguageId() !== LanguageID.rg) {
      return;
    }

    const text = model.getValue();
    onChange(text);
    client.notify(proto.DidChangeTextDocumentNotification.type.method, {
      textDocument: {
        version: model.getVersionId(),
        uri: model.uri.toString(),
      },
      contentChanges: [{ range: asRange(model.getFullModelRange()), text }],
    });
  });

  editor.onDidChangeModel(e => {
    if (e.newModelUrl) {
      const model = editor.getModel();
      if (!model || model.getLanguageId() !== LanguageID.rg) {
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
    if (e.oldModelUrl && e.oldModelUrl.path.endsWith('rg')) {
      client.notify(proto.DidCloseTextDocumentNotification.type.method, {
        textDocument: {
          uri: e.oldModelUrl.toString(),
        },
      });
    }
  });

  client.addMethod(proto.PublishDiagnosticsNotification.type.method, params => {
    const { uri, diagnostics } = params as proto.PublishDiagnosticsParams;
    const diags = diagnostics.map(diagnostic => {
      // We have to map range to Monaco editor
      return {
        severity: monaco.MarkerSeverity.Error,
        message: diagnostic.message,
        startLineNumber: diagnostic.range.start.line + 1,
        startColumn: diagnostic.range.start.character + 1,
        endLineNumber: diagnostic.range.end.line + 1,
        endColumn: diagnostic.range.end.character + 1,
      };
    });
    const model = monaco.editor.getModel(monaco.Uri.parse(uri));
    if (model && model.getLanguageId() === LanguageID.rg) {
      monaco.editor.setModelMarkers(model, LanguageID.rg, diags);
    }
    return;
  });

  return editor;
}
