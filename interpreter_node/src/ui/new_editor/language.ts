import * as monaco from "monaco-editor";
import * as proto from "vscode-languageserver-protocol";
import { languages } from "vscode";
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
    void client;
    monaco.languages.register(Language.extensionPoint());
    monaco.languages.registerDocumentSymbolProvider(this.id, {
      // eslint-disable-next-line
      async provideDocumentSymbols(model, token): Promise<monaco.languages.DocumentSymbol[]> {
        void token;
        const response = await (client.request(proto.DocumentSymbolRequest.type.method, {
          textDocument: monacoToProtocol.asTextDocumentIdentifier(model),
        } as proto.DocumentSymbolParams) as Promise<proto.SymbolInformation[]>);

        const uri = model.uri;

        // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment
        const result: monaco.languages.DocumentSymbol[] = protocolToMonaco.asSymbolInformations(response, uri);

        return result;
      },
    });

    monaco.languages.registerReferenceProvider(this.id, {
      // eslint-disable-next-line
      async provideReferences(model, position, context, token): Promise<monaco.languages.Location[]> {
        void context;
        void token;
        const response = await (client.request(proto.ReferencesRequest.type.method, {
          textDocument: monacoToProtocol.asTextDocumentIdentifier(model),
          position: monacoToProtocol.asPosition(position.lineNumber, position.column),
          context: { includeDeclaration: true },
        } as proto.ReferenceParams) as Promise<proto.Location[]>);

        // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment
        const result: monaco.languages.Location[] = protocolToMonaco.asReferences(response);
        return result;
      },
    });

    monaco.languages.registerDefinitionProvider(this.id, {
      // eslint-disable-next-line
      async provideDefinition(model, position, token): Promise<monaco.languages.Definition> {
        void token;
        const response = await (client.request(proto.DefinitionRequest.type.method, {
          textDocument: monacoToProtocol.asTextDocumentIdentifier(model),
          position: monacoToProtocol.asPosition(position.lineNumber, position.column),
        } as proto.DefinitionParams) as Promise<proto.Location[]>);

        // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment
        const result: monaco.languages.Definition = protocolToMonaco.asDefinitionResult(response);
        return result;
      },
    });

    monaco.languages.registerRenameProvider(this.id, {
      // eslint-disable-next-line
      async provideRenameEdits(model, position, newName, token): Promise<monaco.languages.WorkspaceEdit> {
        void token;
        const response = await (client.request(proto.RenameRequest.type.method, {
          textDocument: monacoToProtocol.asTextDocumentIdentifier(model),
          position: monacoToProtocol.asPosition(position.lineNumber, position.column),
          newName,
        } as proto.RenameParams) as Promise<proto.WorkspaceEdit>);

        // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment
        const result: monaco.languages.WorkspaceEdit = protocolToMonaco.asWorkspaceEdit(response);
        return result;
      },

      async resolveRenameLocation(model, position, token): Promise<monaco.languages.RenameLocation | null> {
        void token;
        const response = await (client.request(proto.PrepareRenameRequest.type.method, {
          textDocument: monacoToProtocol.asTextDocumentIdentifier(model),
          position: monacoToProtocol.asPosition(position.lineNumber, position.column),
        } as proto.PrepareRenameParams) as Promise<{
          range: proto.Range;
          placeholder: string;
        }>);

        // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment
        const result: monaco.languages.RenameLocation | null = { range: protocolToMonaco.asRange(response.range), text: response.placeholder };
        return result;
      }
    });

    monaco.languages.registerDocumentHighlightProvider(this.id, {
      // eslint-disable-next-line
      async provideDocumentHighlights(model, position, token): Promise<monaco.languages.DocumentHighlight[]> {
        void token;
        const response = await (client.request(proto.DocumentHighlightRequest.type.method, {
          textDocument: monacoToProtocol.asTextDocumentIdentifier(model),
          position: monacoToProtocol.asPosition(position.lineNumber, position.column),
        } as proto.DocumentHighlightParams) as Promise<proto.DocumentHighlight[]>);

        // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment
        const result: monaco.languages.DocumentHighlight[] = response.map(protocolToMonaco.asDocumentHighlight);
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
