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
    async provideDocumentSymbols(
      document,
      token,
    ): Promise<vscode.SymbolInformation[]> {
      token;
      const result = await (client.request(
        proto.DocumentSymbolRequest.type.method,
        {
          textDocument: { uri: document.uri.toString() },
        } as proto.DocumentSymbolParams,
      ) as Promise<vscode.SymbolInformation[]>);
      result.forEach(elem => {
        elem.location.uri = vscode.Uri.parse(elem.location.uri.toString());
        const start = elem.location.range.start;
        elem.location.range.start.compareTo = (other: vscode.Position) => {
          if (start.line === other.line) {
            return start.character - other.character;
          }
          return start.line - other.line;
        };
      });
      return result;
    },
  });

  languages.registerDefinitionProvider(LanguageID.rg, {
    async provideDefinition(
      document,
      position,
      token,
    ): Promise<vscode.Definition | vscode.DefinitionLink[]> {
      token;
      const result = await (client.request(
        proto.DefinitionRequest.type.method,
        {
          textDocument: { uri: document.uri.toString() },
          position,
        } as proto.DefinitionParams,
      ) as Promise<vscode.Location | null>);
      if (result === null) {
        return [];
      }
      result.uri = vscode.Uri.parse(result.uri.toString());
      return result;
    },
  });

  languages.registerReferenceProvider(LanguageID.rg, {
    async provideReferences(
      document,
      position,
      context,
      token,
    ): Promise<vscode.Location[]> {
      context;
      token;
      const result = await (client.request(
        proto.ReferencesRequest.type.method,
        {
          textDocument: { uri: document.uri.toString() },
          position,
          context: { includeDeclaration: true },
        } as proto.ReferenceParams,
      ) as Promise<vscode.Location[] | null>);
      if (result === null) {
        return [];
      }
      result.forEach(
        elem => (elem.uri = vscode.Uri.parse(elem.uri.toString())),
      );
      return result;
    },
  });

  languages.registerRenameProvider(LanguageID.rg, {
    async provideRenameEdits(
      document,
      position,
      newName,
      token,
    ): Promise<vscode.WorkspaceEdit> {
      token;
      const result = await (client.request(proto.RenameRequest.type.method, {
        textDocument: { uri: document.uri.toString() },
        position,
        newName,
      } as proto.RenameParams) as Promise<{
        changes: {
          [uri: string]: vscode.TextEdit[];
        };
      }>);
      const edit = new vscode.WorkspaceEdit();
      edit.set(document.uri, result.changes[document.uri.toString()]);
      return edit;
    },

    async prepareRename(
      document,
      position,
      token,
    ): Promise<{
      range: vscode.Range;
      placeholder: string;
    }> {
      token;
      const result = await (client.request(
        proto.PrepareRenameRequest.type.method,
        {
          textDocument: { uri: document.uri.toString() },
          position,
        } as proto.PrepareRenameParams,
      ) as Promise<{
        range: vscode.Range;
        placeholder: string;
      } | null>);
      if (result === null) {
        throw new Error("This element can't be renamed");
      }
      return result;
    },
  });

  languages.registerDocumentHighlightProvider(LanguageID.rg, {
    async provideDocumentHighlights(
      document,
      position,
      token,
    ): Promise<vscode.DocumentHighlight[]> {
      token;
      const result = await (client.request(
        proto.DocumentHighlightRequest.type.method,
        {
          textDocument: { uri: document.uri.toString() },
          position,
        } as proto.DocumentHighlightParams,
      ) as Promise<vscode.DocumentHighlight[]>);
      return result;
    },
  });

  languages.registerDocumentSemanticTokensProvider(
    LanguageID.rg,
    {
      async provideDocumentSemanticTokens(
        document,
        token,
      ): Promise<vscode.SemanticTokens> {
        token;
        const result = await (client.request(
          proto.SemanticTokensRequest.type.method,
          {
            textDocument: { uri: document.uri.toString() },
          } as proto.SemanticTokensParams,
        ) as Promise<vscode.SemanticTokens>);
        return result;
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
    async provideHover(
      document,
      position,
      token,
    ): Promise<vscode.Hover | null> {
      token;
      const result = await (client.request(proto.HoverRequest.type.method, {
        textDocument: { uri: document.uri.toString() },
        position,
      } as proto.HoverParams) as Promise<vscode.Hover | null>);
      return result;
    },
  });

  languages.registerCodeActionsProvider(LanguageID.rg, {
    async provideCodeActions(document, range, context, token) {
      token;
      const result = await (client.request(
        proto.CodeActionRequest.type.method,
        {
          textDocument: { uri: document.uri.toString() },
          range,
          context: { diagnostics: [] },
        } as proto.CodeActionParams,
      ) as Promise<
        {
          edit: {
            changes: {
              [uri: string]: vscode.TextEdit[];
            };
          };
          kind: vscode.CodeActionKind;
          title: string;
        }[]
      >);
      return result.map(elem => {
        const changes = new vscode.WorkspaceEdit();
        changes.set(document.uri, elem.edit.changes[document.uri.toString()]);
        const action = new vscode.CodeAction(elem.title, elem.kind);
        action.edit = changes;
        return action;
      });
    },
  });

  languages.registerCompletionItemProvider(LanguageID.rg, {
    async provideCompletionItems(
      document,
      position,
      token,
      context,
    ): Promise<vscode.CompletionItem[]> {
      token;
      context;
      const result = await (client.request(
        proto.CompletionRequest.type.method,
        {
          textDocument: { uri: document.uri.toString() },
          position,
          context: undefined,
        } as proto.CompletionParams,
      ) as Promise<
        | {
            label: string;
            kind: vscode.CompletionItemKind;
            labelDetails?: {
              detail?: string;
            };
          }[]
        | null
      >);
      if (result === null) {
        return [];
      }
      return result.map(elem => {
        const label = {
          label: elem.label,
          detail: elem.labelDetails?.detail,
        };
        return new vscode.CompletionItem(label, elem.kind);
      });
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
