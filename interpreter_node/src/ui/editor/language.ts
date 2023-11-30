import * as monaco from 'monaco-editor';
import { languages } from 'vscode';
import * as vscode from 'vscode';
import * as proto from 'vscode-languageserver-protocol';

import Client from './client';
import { conf, theme, monarch } from './syntax/conf';
import { LanguageID } from '../../types';

let initialized: boolean;

const registerLanguage = (client: Client) => {
  monaco.editor.defineTheme('rgTheme', theme);

  [LanguageID.rg, LanguageID.rbg, LanguageID.hrg].forEach(lang => {
    monaco.languages.register({ id: lang });
    monaco.languages.setLanguageConfiguration(lang, conf);
    monaco.languages.setMonarchTokensProvider(lang, monarch(lang));
  });

  monaco.languages.register({
    id: 'json',
    extensions: ['.json', '.jsonc'],
    aliases: ['JSON', 'json'],
    mimetypes: ['application/json'],
  });
  monaco.languages.register({ id: 'javascript' });

  languages.registerDocumentSymbolProvider(LanguageID.rg, {
    async provideDocumentSymbols(document) {
      const result: WasmSymbolInformation[] = await client.request(
        proto.DocumentSymbolRequest.type.method,
        { textDocument: documentToUri(document) },
      );
      return result.map(wasmSymbolInformationToVscodeSymbolInformation);
    },
  });

  languages.registerDefinitionProvider(LanguageID.rg, {
    async provideDefinition(document, position) {
      const result: WasmLocation | null = await client.request(
        proto.DefinitionRequest.type.method,
        { position, textDocument: documentToUri(document) },
      );
      return result && wasmLocationToVscodeLocation(result);
    },
  });

  languages.registerReferenceProvider(LanguageID.rg, {
    async provideReferences(document, position) {
      const result: WasmLocation[] | null = await client.request(
        proto.ReferencesRequest.type.method,
        {
          textDocument: documentToUri(document),
          position,
          context: { includeDeclaration: true },
        },
      );
      return result ? result.map(wasmLocationToVscodeLocation) : [];
    },
  });

  languages.registerRenameProvider(LanguageID.rg, {
    async provideRenameEdits(document, position, newName) {
      const result: WasmWorkspaceEdit = await client.request(
        proto.RenameRequest.type.method,
        {
          textDocument: documentToUri(document),
          position,
          newName,
        },
      );
      return wasmWorkspaceEditToVscodeWorkspaceEdit(result);
    },

    async prepareRename(document, position) {
      const result: { range: WasmRange; placeholder: string } | null =
        await client.request(proto.PrepareRenameRequest.type.method, {
          textDocument: documentToUri(document),
          position,
        });
      return (
        result && {
          range: wasmRangeToVscodeRange(result.range),
          placeholder: result.placeholder,
        }
      );
    },
  });

  languages.registerDocumentHighlightProvider(LanguageID.rg, {
    async provideDocumentHighlights(document, position) {
      const result: WasmHighlight[] | null = await client.request(
        proto.DocumentHighlightRequest.type.method,
        { textDocument: documentToUri(document), position },
      );
      return result && result.map(wasmHighlightToVscodeHighlight);
    },
  });

  languages.registerDocumentSemanticTokensProvider(
    LanguageID.rg,
    {
      async provideDocumentSemanticTokens(document) {
        const result: { data: Uint32Array } = await client.request(
          proto.SemanticTokensRequest.type.method,
          { textDocument: documentToUri(document) },
        );
        return new vscode.SemanticTokens(result.data);
      },
    },
    new vscode.SemanticTokensLegend(
      [
        'declarationKeyword',
        'type',
        'parameter',
        'variable',
        'function',
        'comment',
        'operator',
        'macro',
        'member',
        'constant',
      ],
      ['definition', 'readonly'],
    ),
  );

  languages.registerHoverProvider(LanguageID.rg, {
    async provideHover(document, position) {
      const result: WasmHover | null = await client.request(
        proto.HoverRequest.type.method,
        { textDocument: documentToUri(document), position },
      );
      return result && wasmHoverToVscodeHover(result);
    },
  });

  languages.registerCodeActionsProvider(LanguageID.rg, {
    async provideCodeActions(document, range) {
      const result: WasmCodeAction[] = await client.request(
        proto.CodeActionRequest.type.method,
        {
          textDocument: documentToUri(document),
          range,
          context: { diagnostics: [] },
        },
      );
      return result.map(wasmCodeActionToVscodeCodeAction);
    },
  });

  languages.registerCompletionItemProvider(LanguageID.rg, {
    async provideCompletionItems(document, position) {
      const result: WasmCompletionItem[] | null = await client.request(
        proto.CompletionRequest.type.method,
        {
          textDocument: documentToUri(document),
          position,
          context: undefined,
        },
      );
      return result ? result.map(wasmCompletionItemToVscodeCompletionItem) : [];
    },
  });
};

export const initialize = (client: Client) => {
  if (initialized) {
    console.warn('Language already initialized; ignoring');
  } else {
    initialized = true;
    registerLanguage(client);
    console.log('Language initialized');
  }
};

type WasmLocation = { uri: WasmUri; range: WasmRange };
const wasmLocationToVscodeLocation = (x: WasmLocation) =>
  new vscode.Location(
    wasmUriToVscodeUri(x.uri),
    wasmRangeToVscodeRange(x.range),
  );

type WasmUri = string;
const wasmUriToVscodeUri = (x: WasmUri) => vscode.Uri.parse(x);

type WasmRange = { start: WasmPosition; end: WasmPosition };
const wasmRangeToVscodeRange = (x: WasmRange) =>
  new vscode.Range(
    wasmPositionToVscodePosition(x.start),
    wasmPositionToVscodePosition(x.end),
  );

type WasmHighlight = { range: WasmRange };
const wasmHighlightToVscodeHighlight = (x: WasmHighlight) =>
  new vscode.DocumentHighlight(wasmRangeToVscodeRange(x.range));

type WasmPosition = { character: number; line: number };
const wasmPositionToVscodePosition = (x: WasmPosition) =>
  new vscode.Position(x.line, x.character);

type WasmTextEdit = { range: WasmRange; newText: string };
const wasmTextEditToVscodeTextEdit = (x: WasmTextEdit) =>
  new vscode.TextEdit(wasmRangeToVscodeRange(x.range), x.newText);

type WasmWorkspaceEdit = {
  changes: {
    [uri: WasmUri]: WasmTextEdit[];
  };
};
const wasmWorkspaceEditToVscodeWorkspaceEdit = (x: WasmWorkspaceEdit) => {
  const edit = new vscode.WorkspaceEdit();
  Object.entries(x.changes).forEach(([uri, textEdits]) => {
    edit.set(
      wasmUriToVscodeUri(uri),
      textEdits.map(wasmTextEditToVscodeTextEdit),
    );
  });
  return edit;
};

type WasmHover = {
  contents: { language: string; value: string }[];
  range: WasmRange;
};
const wasmHoverToVscodeHover = (x: WasmHover) => {
  return new vscode.Hover(x.contents, wasmRangeToVscodeRange(x.range));
};

type WasmCodeAction = {
  edit: WasmWorkspaceEdit;
  kind: vscode.CodeActionKind;
  title: string;
};
const wasmCodeActionToVscodeCodeAction = (x: WasmCodeAction) => {
  const edit = wasmWorkspaceEditToVscodeWorkspaceEdit(x.edit);
  const action = new vscode.CodeAction(x.title, x.kind);
  action.edit = edit;
  return action;
};

type WasmCompletionItem = {
  label: string;
  kind: vscode.CompletionItemKind;
  labelDetails?: {
    detail?: string;
  };
};
const wasmCompletionItemToVscodeCompletionItem = (x: WasmCompletionItem) => {
  const label = {
    label: x.label,
    detail: x.labelDetails?.detail,
  };
  return new vscode.CompletionItem(label, x.kind);
};
type WasmSymbolInformation = {
  kind: vscode.SymbolKind;
  name: string;
  location: WasmLocation;
};

const wasmSymbolInformationToVscodeSymbolInformation = (
  x: WasmSymbolInformation,
) => {
  return new vscode.SymbolInformation(
    x.name,
    x.kind,
    '',
    wasmLocationToVscodeLocation(x.location),
  );
};

const documentToUri = (x: vscode.TextDocument) => ({ uri: x.uri.toString() });
