use super::document::Document;
use super::utils::lsp_to_pos;
use super::{completions, features, semantic_tokens};
use dashmap::DashMap;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};

pub struct Backend {
    client: Client,
    document_map: DashMap<String, Document>,
}

impl Backend {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            document_map: DashMap::new(),
        }
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            server_info: None,
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Options(
                    TextDocumentSyncOptions {
                        open_close: Some(true),
                        change: Some(TextDocumentSyncKind::FULL),
                        ..TextDocumentSyncOptions::default()
                    },
                )),
                document_symbol_provider: Some(OneOf::Left(true)),
                document_highlight_provider: Some(OneOf::Left(true)),
                execute_command_provider: None,
                semantic_tokens_provider: Some(semantic_tokens::capabilities()),
                definition_provider: Some(OneOf::Left(true)),
                references_provider: Some(OneOf::Left(true)),
                rename_provider: Some(OneOf::Right(RenameOptions {
                    prepare_provider: Some(true),
                    work_done_progress_options: WorkDoneProgressOptions {
                        work_done_progress: None,
                    },
                })),
                diagnostic_provider: Some(DiagnosticServerCapabilities::Options(
                    DiagnosticOptions {
                        identifier: Some("rg-lsp".to_string()),
                        inter_file_dependencies: false,
                        workspace_diagnostics: false,
                        work_done_progress_options: WorkDoneProgressOptions {
                            work_done_progress: None,
                        },
                    },
                )),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                completion_provider: Some(completions::capabilities()),
                ..ServerCapabilities::default()
            },
        })
    }
    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "initialized")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri;
        let text = params.text_document.text;
        let (document, errors) = Document::new(text);
        self.document_map.insert(uri.to_string(), document);
        let diags = features::diagnostics(errors);
        self.client.publish_diagnostics(uri, diags, None).await;
    }

    async fn did_change(&self, mut params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri;
        let text = params.content_changes.pop().unwrap().text;
        let (document, errors) = Document::new(text);
        self.document_map.insert(uri.to_string(), document);
        let diags = features::diagnostics(errors);
        self.client.publish_diagnostics(uri, diags, None).await;
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        self.client
            .log_message(MessageType::INFO, "did close")
            .await;
        let uri = params.text_document.uri;
        self.document_map.remove(&uri.to_string());
    }

    async fn did_save(&self, _: DidSaveTextDocumentParams) {}

    async fn document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> Result<Option<DocumentSymbolResponse>> {
        let uri = params.text_document.uri;
        let document = self.document_map.get(&uri.to_string()).unwrap();
        let document_symbols = features::document_symbol(&uri, &document.symbol_table);
        Ok(document_symbols)
    }

    async fn references(&self, params: ReferenceParams) -> Result<Option<Vec<Location>>> {
        let uri = params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;
        let document = self.document_map.get(&uri.to_string()).unwrap();
        let locations = features::references(&uri, position, &document.symbol_table);
        Ok(locations)
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        let uri = params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;
        let document = self.document_map.get(&uri.to_string()).unwrap();
        let definition = features::definitions(&uri, position, &document.symbol_table);
        Ok(definition)
    }

    async fn semantic_tokens_full(
        &self,
        params: SemanticTokensParams,
    ) -> Result<Option<SemanticTokensResult>> {
        let uri = params.text_document.uri;
        let document = self.document_map.get(&uri.to_string()).unwrap();
        let semantic_tokens = semantic_tokens::semantic_tokens_full(&document);
        Ok(Some(SemanticTokensResult::Tokens(SemanticTokens {
            result_id: None,
            data: semantic_tokens,
        })))
    }

    async fn document_highlight(
        &self,
        params: DocumentHighlightParams,
    ) -> Result<Option<Vec<DocumentHighlight>>> {
        let uri = params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;
        let document = self.document_map.get(&uri.to_string()).unwrap();
        let symbol_table = &document.symbol_table;
        let document_highlights = features::document_highlight(position, symbol_table);
        Ok(document_highlights)
    }

    async fn prepare_rename(
        &self,
        params: TextDocumentPositionParams,
    ) -> Result<Option<PrepareRenameResponse>> {
        let uri = params.text_document.uri;
        let position = params.position;
        let document = self.document_map.get(&uri.to_string()).unwrap();
        let symbol_table = &document.symbol_table;
        let prepare_rename_response = features::prepare_rename(position, symbol_table);
        Ok(prepare_rename_response)
    }

    async fn rename(&self, params: RenameParams) -> Result<Option<WorkspaceEdit>> {
        let uri = params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;
        let document = self.document_map.get(&uri.to_string()).unwrap();
        let symbol_table = &document.symbol_table;
        let rename_response = features::rename(&uri, position, symbol_table, params.new_name);
        Ok(rename_response)
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;
        let document = self.document_map.get(&uri.to_string()).unwrap();
        let symbol_table = &document.symbol_table;
        let game = &document.game;
        let hover_response = features::hover(position, symbol_table, game);
        Ok(hover_response)
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let uri = params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;
        let document = self.document_map.get(&uri.to_string()).unwrap();
        let symbol_table = &document.symbol_table;
        let game = &document.game;
        let completion_response =
            completions::completions(lsp_to_pos(position), game, symbol_table);
        Ok(completion_response)
    }
}
