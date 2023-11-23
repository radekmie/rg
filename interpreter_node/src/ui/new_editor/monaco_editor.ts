import debounce from 'debounce';
import * as monaco from 'monaco-editor';
import { createConfiguredEditor, createModelReference } from 'vscode/monaco';
import {
  RegisteredFileSystemProvider,
  registerFileSystemOverlay,
  RegisteredMemoryFile,
} from '@codingame/monaco-vscode-files-service-override';
import * as proto from 'vscode-languageserver-protocol';
import * as vscode from 'vscode';
import Client from './client';
import EditorWorker from 'url:monaco-editor/esm/vs/editor/editor.worker.js';
import { LanguageID } from '../../types';

window.MonacoEnvironment = {
  getWorkerUrl: function (moduleId, label) {
    return EditorWorker;
  },
};

export async function createModel(
  path: string,
  content: string,
): Promise<monaco.editor.ITextModel> {
  const lang = pathToLang(path);
  const fileSystemProvider = new RegisteredFileSystemProvider(false);
  fileSystemProvider.registerFile(
    new RegisteredMemoryFile(vscode.Uri.file('/workspace/' + path), ''),
  );
  registerFileSystemOverlay(1, fileSystemProvider);

  const uri = monaco.Uri.parse('/workspace/' + path);

  const model = monaco.editor.createModel(content, lang, uri);
  console.log('Model created', model.getLanguageId());
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

const pathToLang = (path: string): LanguageID => {
  switch (path.split('.').pop()) {
    case 'rg':
      return LanguageID.rg;
    case 'rbg':
      return LanguageID.rbg;
    case 'hrg':
      return LanguageID.hrg;
    default:
      throw new Error('Unknown extension');
  }
};

export async function createEditor(
  client: Client,
  container: HTMLElement,
  onChange: (source: string) => void,
): Promise<monaco.editor.IStandaloneCodeEditor> {
  const editor = createConfiguredEditor(container, {
    automaticLayout: true,
    'semanticHighlighting.enabled': true,
    theme: 'rgTheme',
  });

  editor.onDidChangeModelContent(e => {
    const model = editor.getModel();
    console.log('Model changed ', model?.uri.toString());
    if (!model || model.getLanguageId() !== LanguageID.rg) {
      return;
    }
    const text = model.getValue();
    onChange(text);
    client.notify(proto.DidChangeTextDocumentNotification.type.method, {
      textDocument: {
        version: model.getVersionId(),
        uri: model.uri.toString(),
      },
      contentChanges: [
        {
          range: asRange(model.getFullModelRange()),
          text,
        },
      ],
    } as proto.DidChangeTextDocumentParams);
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
      } as proto.DidOpenTextDocumentParams);
    }
    if (e.oldModelUrl) {
      client.notify(proto.DidCloseTextDocumentNotification.type.method, {
        textDocument: {
          uri: e.oldModelUrl.toString(),
        },
      } as proto.DidCloseTextDocumentParams);
    }
  });

  client.addMethod(proto.PublishDiagnosticsNotification.type.method, params => {
    const { uri, diagnostics } = params as proto.PublishDiagnosticsParams;
    console.log('Diagnostics received', uri);
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
    console.log('Model uri', model?.uri.toString());
    if (model && model.getLanguageId() === LanguageID.rg) {
      monaco.editor.setModelMarkers(model, LanguageID.rg, diags);
    }
    return;
  });

  return editor;
}
