import * as monaco from "monaco-editor";
import * as proto from "vscode-languageserver-protocol";
import { ProviderResult, languages } from "vscode";
import * as vscode from "vscode";
import Client from "./client";

let language: null | Language;

export default class Language implements monaco.languages.ILanguageExtensionPoint {
  readonly id: string;
  readonly aliases: string[];
  readonly extensions: string[];
  readonly mimetypes: string[];

  private constructor(client: Client) {
    const { id, aliases, extensions, mimetypes } = Language.extensionPoint();
    this.id = id;
    this.aliases = aliases;
    this.extensions = extensions;
    this.mimetypes = mimetypes;
    this.registerLanguage(client);
  }

  static extensionPoint(): monaco.languages.ILanguageExtensionPoint & {
    aliases: string[];
    extensions: string[];
    mimetypes: string[];
  } {
    const id = "rg";
    const aliases = ["RegularGames", "regulargames"];
    const extensions = [".rg"];
    const mimetypes = ["text/plain"];
    return { id, extensions, aliases, mimetypes };
  }

  private registerLanguage(client: Client): void {
    monaco.languages.register(Language.extensionPoint());


    languages.setLanguageConfiguration(this.id, {
      brackets: [
        ["{", "}"],
        ["[", "]"],
        ["(", ")"],
      ],
      comments: {
        lineComment: "//",
      },
      autoClosingPairs: [
        { open: "{", close: "}" },
        { open: "[", close: "]" },
        { open: "(", close: ")" },
        { open: '"', close: '"', notIn: [vscode.SyntaxTokenType.String] },
        { open: "'", close: "'", notIn: [vscode.SyntaxTokenType.String] },
      ],
    });

    languages.registerDocumentSymbolProvider(this.id, {
      async provideDocumentSymbols(document, token): Promise<vscode.SymbolInformation[]> {
        void token;
        let result = await (client.request(proto.DocumentSymbolRequest.type.method, {
          textDocument: { uri: document.uri.toString() },
        } as proto.DocumentSymbolParams) as Promise<vscode.SymbolInformation[]>);
        result.forEach(elem => elem.location.uri = vscode.Uri.parse(elem.location.uri.toString()));
        return result;
      },
    });

    languages.registerDefinitionProvider(this.id, {
      async provideDefinition(document, position, token): Promise<vscode.Definition | vscode.DefinitionLink[]> {
        void token;
        let result = await (client.request(proto.DefinitionRequest.type.method, {
          textDocument: { uri: document.uri.toString() },
          position: position,
        } as proto.DefinitionParams) as Promise<vscode.Location | null>);
        if (result == null) {
          return [];
        }
        result.uri = vscode.Uri.parse(result.uri.toString());
        return result;
      },
    })

    languages.registerReferenceProvider(this.id, {
      async provideReferences(document, position, context, token): Promise<vscode.Location[]> {
        void context;
        void token;
        let result = await (client.request(proto.ReferencesRequest.type.method, {
          textDocument: { uri: document.uri.toString() },
          position: position,
          context: { includeDeclaration: true },
        } as proto.ReferenceParams) as Promise<vscode.Location[] | null>);
        if (result == null) {
          return [];
        }
        result.forEach(elem => elem.uri = vscode.Uri.parse(elem.uri.toString()));
        return result;
      },
    });



    languages.registerRenameProvider(this.id, {
      async provideRenameEdits(document, position, newName, token): Promise<vscode.WorkspaceEdit> {
        void token;
        let result = await (client.request(proto.RenameRequest.type.method, {
          textDocument: { uri: document.uri.toString() },
          position: position,
          newName,
        } as proto.RenameParams) as Promise<{
          changes: {
            [uri: string]: vscode.TextEdit[];
          }
        }>);
        let new_result = new vscode.WorkspaceEdit();
        new_result.set(document.uri, result.changes[document.uri.toString()]);
        return new_result;
      },

      async prepareRename(document, position, token): Promise<{
        range: vscode.Range;
        placeholder: string;
      }> {
        void token;
        const result = await (client.request(proto.PrepareRenameRequest.type.method, {
          textDocument: { uri: document.uri.toString() },
          position: position,
        } as proto.PrepareRenameParams) as Promise<{
          range: vscode.Range;
          placeholder: string;
        } | null>);
        if (result == null) {
          throw new Error("This element can't be renamed");
        }
        return result;
      }
    });

    languages.registerDocumentHighlightProvider(this.id, {
      async provideDocumentHighlights(document, position, token): Promise<vscode.DocumentHighlight[]> {
        void token;
        const result = await (client.request(proto.DocumentHighlightRequest.type.method, {
          textDocument: { uri: document.uri.toString() },
          position: position,
        } as proto.DocumentHighlightParams) as Promise<vscode.DocumentHighlight[]>);
        return result;
      },
    });

    languages.registerDocumentSemanticTokensProvider(this.id, {
      async provideDocumentSemanticTokens(document, token): Promise<vscode.SemanticTokens> {
        void token;
        const result = await (client.request(proto.SemanticTokensRequest.type.method, {
          textDocument: { uri: document.uri.toString() },
        } as proto.SemanticTokensParams) as Promise<vscode.SemanticTokens>);
        return result;
      },
    },
      new vscode.SemanticTokensLegend(
        ["keyword", "type", "parameter", "variable", "function", "comment", "operator", "macro", "member", "constant"],
        ["definition", "readonly"],
      )
    );

    languages.registerHoverProvider(this.id, {
      async provideHover(document, position, token): Promise<vscode.Hover | null> {
        void token;
        const result = await (client.request(proto.HoverRequest.type.method, {
          textDocument: { uri: document.uri.toString() },
          position: position,
        } as proto.HoverParams) as Promise<vscode.Hover | null>);
        return result;
      },
    });

    languages.registerCompletionItemProvider(this.id, {
      async provideCompletionItems(document, position, token, context): Promise<vscode.CompletionItem[]> {
        void token;
        const result = await (client.request(proto.CompletionRequest.type.method, {
          textDocument: { uri: document.uri.toString() },
          position: position,
          context: {
            triggerKind: context.triggerKind,
            triggerCharacter: context.triggerCharacter,
          },
        } as proto.CompletionParams) as Promise<vscode.CompletionItem[]>);

        return result;
      },
    });


  }

  static initialize(client: Client): Language {
    if (null == language) {
      language = new Language(client);
    } else {
      console.warn("Language already initialized; ignoring");
    }
    return language;
  }
}
