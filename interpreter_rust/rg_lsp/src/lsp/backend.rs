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
                        ..DiagnosticOptions::default()
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
        let diagnostics = self.add_document(&uri, text);
        self.client
            .publish_diagnostics(uri, diagnostics, None)
            .await;
    }

    async fn did_change(&self, mut params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri;
        let text = params.content_changes.pop().unwrap().text;
        let diagnostics = self.add_document(&uri, text);
        self.client
            .publish_diagnostics(uri, diagnostics, None)
            .await;
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let uri = params.text_document.uri;
        self.document_map.remove(&uri.to_string());
    }

    async fn did_save(&self, _: DidSaveTextDocumentParams) {}

    async fn document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> Result<Option<DocumentSymbolResponse>> {
        self.with_document(&params.text_document.uri, |uri, document| {
            features::document_symbol(uri, &document.symbol_table)
        })
    }

    async fn references(&self, params: ReferenceParams) -> Result<Option<Vec<Location>>> {
        self.with_document_positioned(&params.text_document_position, |uri, position, document| {
            features::references(uri, position, &document.symbol_table)
        })
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        self.with_document_positioned(
            &params.text_document_position_params,
            |uri, position, document| features::definitions(uri, position, &document.symbol_table),
        )
    }

    async fn semantic_tokens_full(
        &self,
        params: SemanticTokensParams,
    ) -> Result<Option<SemanticTokensResult>> {
        self.with_document(&params.text_document.uri, |_, document| {
            Some(SemanticTokensResult::Tokens(SemanticTokens {
                result_id: None,
                data: semantic_tokens::semantic_tokens_full(document),
            }))
        })
    }

    async fn document_highlight(
        &self,
        params: DocumentHighlightParams,
    ) -> Result<Option<Vec<DocumentHighlight>>> {
        self.with_document_positioned(
            &params.text_document_position_params,
            |_, position, document| features::document_highlight(position, &document.symbol_table),
        )
    }

    async fn prepare_rename(
        &self,
        params: TextDocumentPositionParams,
    ) -> Result<Option<PrepareRenameResponse>> {
        self.with_document_positioned(&params, |_, position, document| {
            features::prepare_rename(position, &document.symbol_table)
        })
    }

    async fn rename(&self, params: RenameParams) -> Result<Option<WorkspaceEdit>> {
        self.with_document_positioned(&params.text_document_position, |uri, position, document| {
            features::rename(uri, position, &document.symbol_table, params.new_name)
        })
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        self.with_document_positioned(
            &params.text_document_position_params,
            |_, position, document| {
                features::hover(position, &document.symbol_table, &document.game)
            },
        )
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        self.with_document_positioned(&params.text_document_position, |_, position, document| {
            completions::completions(lsp_to_pos(position), &document.game, &document.symbol_table)
        })
    }
}

impl Backend {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            document_map: DashMap::new(),
        }
    }

    fn with_document_positioned<T>(
        &self,
        params: &TextDocumentPositionParams,
        f: impl FnOnce(&Url, &Position, &Document) -> T,
    ) -> Result<T> {
        let uri = &params.text_document.uri;
        let position = &params.position;
        let document = self.document_map.get(&uri.to_string()).unwrap();
        Ok(f(uri, position, &document))
    }

    fn with_document<T>(&self, uri: &Url, f: impl FnOnce(&Url, &Document) -> T) -> Result<T> {
        let document = self.document_map.get(&uri.to_string()).unwrap();
        Ok(f(uri, &document))
    }

    fn add_document(&self, uri: &Url, text: String) -> Vec<Diagnostic> {
        let (document, errors) = Document::new(text);
        self.document_map.insert(uri.to_string(), document);
        features::diagnostics(errors)
    }
}
