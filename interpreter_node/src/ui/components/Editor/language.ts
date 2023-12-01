import * as monaco from 'monaco-editor';
import * as vscode from 'vscode';
import * as proto from 'vscode-languageserver-protocol';

import Client from './client';
import { conf, theme, monarch } from './syntax/conf';
import { Language } from '../../../types';

function registerLanguage(client: Client) {
  monaco.editor.defineTheme('rgTheme', theme);

  [Language.rg, Language.rbg, Language.hrg].forEach(lang => {
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

  vscode.languages.registerDocumentSymbolProvider(Language.rg, {
    async provideDocumentSymbols(document) {
      const result: WasmSymbolInformation[] = await client.request(
        proto.DocumentSymbolRequest.type.method,
        { textDocument: documentToUri(document) },
      );
      return result.map(wasmSymbolInformationToVscodeSymbolInformation);
    },
  });

  vscode.languages.registerDefinitionProvider(Language.rg, {
    async provideDefinition(document, position) {
      const result: WasmLocation | null = await client.request(
        proto.DefinitionRequest.type.method,
        { position, textDocument: documentToUri(document) },
      );
      return result && wasmLocationToVscodeLocation(result);
    },
  });

  vscode.languages.registerReferenceProvider(Language.rg, {
    async provideReferences(document, position) {
      const result: WasmLocation[] | null = await client.request(
        proto.ReferencesRequest.type.method,
        {
          context: { includeDeclaration: true },
          position,
          textDocument: documentToUri(document),
        },
      );
      return result ? result.map(wasmLocationToVscodeLocation) : [];
    },
  });

  vscode.languages.registerRenameProvider(Language.rg, {
    async provideRenameEdits(document, position, newName) {
      const result: WasmWorkspaceEdit = await client.request(
        proto.RenameRequest.type.method,
        {
          newName,
          position,
          textDocument: documentToUri(document),
        },
      );
      return wasmWorkspaceEditToVscodeWorkspaceEdit(result);
    },

    async prepareRename(document, position) {
      const result: { range: WasmRange; placeholder: string } | null =
        await client.request(proto.PrepareRenameRequest.type.method, {
          position,
          textDocument: documentToUri(document),
        });
      return (
        result && {
          placeholder: result.placeholder,
          range: wasmRangeToVscodeRange(result.range),
        }
      );
    },
  });

  vscode.languages.registerDocumentHighlightProvider(Language.rg, {
    async provideDocumentHighlights(document, position) {
      const result: WasmHighlight[] | null = await client.request(
        proto.DocumentHighlightRequest.type.method,
        { position, textDocument: documentToUri(document) },
      );
      return result && result.map(wasmHighlightToVscodeHighlight);
    },
  });

  vscode.languages.registerDocumentSemanticTokensProvider(
    Language.rg,
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

  vscode.languages.registerHoverProvider(Language.rg, {
    async provideHover(document, position) {
      const result: WasmHover | null = await client.request(
        proto.HoverRequest.type.method,
        { textDocument: documentToUri(document), position },
      );
      return result && wasmHoverToVscodeHover(result);
    },
  });

  vscode.languages.registerCodeActionsProvider(Language.rg, {
    async provideCodeActions(document, range) {
      const result: WasmCodeAction[] = await client.request(
        proto.CodeActionRequest.type.method,
        {
          context: { diagnostics: [] },
          range,
          textDocument: documentToUri(document),
        },
      );
      return result.map(wasmCodeActionToVscodeCodeAction);
    },
  });

  vscode.languages.registerCompletionItemProvider(Language.rg, {
    async provideCompletionItems(document, position) {
      const result: WasmCompletionItem[] | null = await client.request(
        proto.CompletionRequest.type.method,
        {
          context: undefined,
          position,
          textDocument: documentToUri(document),
        },
      );
      return result ? result.map(wasmCompletionItemToVscodeCompletionItem) : [];
    },
  });
}

let initialized = false;
export function initialize(client: Client) {
  if (initialized) {
    console.warn('Language already initialized; ignoring');
  } else {
    initialized = true;
    registerLanguage(client);
    console.log('Language initialized');
  }
}

type WasmLocation = { range: WasmRange; uri: WasmUri };
const wasmLocationToVscodeLocation = (x: WasmLocation) =>
  new vscode.Location(
    wasmUriToVscodeUri(x.uri),
    wasmRangeToVscodeRange(x.range),
  );

type WasmUri = string;
const wasmUriToVscodeUri = (x: WasmUri) => vscode.Uri.parse(x);

type WasmRange = { end: WasmPosition; start: WasmPosition };
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

type WasmTextEdit = { newText: string; range: WasmRange };
const wasmTextEditToVscodeTextEdit = (x: WasmTextEdit) =>
  new vscode.TextEdit(wasmRangeToVscodeRange(x.range), x.newText);

type WasmWorkspaceEdit = { changes: { [uri: WasmUri]: WasmTextEdit[] } };
const wasmWorkspaceEditToVscodeWorkspaceEdit = (x: WasmWorkspaceEdit) =>
  Object.entries(x.changes).reduce((edit, [uri, textEdits]) => {
    edit.set(
      wasmUriToVscodeUri(uri),
      textEdits.map(wasmTextEditToVscodeTextEdit),
    );
    return edit;
  }, new vscode.WorkspaceEdit());

type WasmHover = {
  contents: { language: string; value: string }[];
  range: WasmRange;
};
const wasmHoverToVscodeHover = (x: WasmHover) =>
  new vscode.Hover(x.contents, wasmRangeToVscodeRange(x.range));

type WasmCodeAction = {
  edit: WasmWorkspaceEdit;
  kind: vscode.CodeActionKind;
  title: string;
};
const wasmCodeActionToVscodeCodeAction = (x: WasmCodeAction) =>
  Object.assign(new vscode.CodeAction(x.title, x.kind), {
    edit: wasmWorkspaceEditToVscodeWorkspaceEdit(x.edit),
  });

type WasmCompletionItem = {
  kind: vscode.CompletionItemKind;
  label: string;
  labelDetails?: { detail?: string };
};
const wasmCompletionItemToVscodeCompletionItem = (x: WasmCompletionItem) =>
  new vscode.CompletionItem(
    { detail: x.labelDetails?.detail, label: x.label },
    x.kind,
  );

type WasmSymbolInformation = {
  kind: vscode.SymbolKind;
  location: WasmLocation;
  name: string;
};
const wasmSymbolInformationToVscodeSymbolInformation = (
  x: WasmSymbolInformation,
) =>
  new vscode.SymbolInformation(
    x.name,
    x.kind,
    '',
    wasmLocationToVscodeLocation(x.location),
  );

const documentToUri = (x: vscode.TextDocument) => ({ uri: x.uri.toString() });
