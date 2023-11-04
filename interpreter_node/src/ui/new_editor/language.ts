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
      // eslint-disable-next-line
      async provideDocumentSymbols(document, token): Promise<vscode.SymbolInformation[]> {
        void token;
        const result = await (client.request(proto.DocumentSymbolRequest.type.method, {
          textDocument: { uri: document.uri.toString() },
        } as proto.DocumentSymbolParams) as Promise<vscode.SymbolInformation[]>);
        return result;
      },
    });

    languages.registerDefinitionProvider(this.id, {
      async provideDefinition(document, position, token): Promise<vscode.Definition | null> {
        void token;
        const result = await (client.request(proto.DefinitionRequest.type.method, {
          textDocument: { uri: document.uri.toString() },
          position: position,
        } as proto.DefinitionParams) as Promise<vscode.Definition | null>);
        return result;
      },
    })

    languages.registerReferenceProvider(this.id, {
      // eslint-disable-next-line
      async provideReferences(document, position, context, token): Promise<vscode.Location[]> {
        void context;
        void token;
        const result = await (client.request(proto.ReferencesRequest.type.method, {
          textDocument: { uri: document.uri.toString() },
          position: position,
          context: { includeDeclaration: true },
        } as proto.ReferenceParams) as Promise<vscode.Location[]>);
        return result;
      },
    });




    languages.registerRenameProvider(this.id, {
      // eslint-disable-next-line
      async provideRenameEdits(document, position, newName, token): Promise<vscode.WorkspaceEdit> {
        void token;
        const result = await (client.request(proto.RenameRequest.type.method, {
          textDocument: { uri: document.uri.toString() },
          position: position,
          newName,
        } as proto.RenameParams) as Promise<vscode.WorkspaceEdit>);
        return result;
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
        }>);
        return result;
      }
    });

    languages.registerDocumentHighlightProvider(this.id, {
      // eslint-disable-next-line
      async provideDocumentHighlights(document, position, token): Promise<vscode.DocumentHighlight[]> {
        void token;
        const result = await (client.request(proto.DocumentHighlightRequest.type.method, {
          textDocument: { uri: document.uri.toString() },
          position: position,
        } as proto.DocumentHighlightParams) as Promise<vscode.DocumentHighlight[]>);
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
