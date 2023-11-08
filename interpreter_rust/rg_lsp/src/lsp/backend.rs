use super::document::Document;
use super::{features, logger, semantic_tokens};
use dashmap::DashMap;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

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
                completion_provider: None,
                execute_command_provider: None,
                semantic_tokens_provider: Some(semantic_tokens::semantic_tokens_capabilities()),
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
                ..ServerCapabilities::default()
            },
        })
    }
    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "initialized XD!")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        // self.client.log_message(MessageType::INFO, "did open").await;
        let uri = params.text_document.uri;
        let text = params.text_document.text;
        let mut document = Document::new(text);
        let errors = document.parse();
        self.document_map.insert(uri.to_string(), document);
        let diags = features::diagnostics(errors);
        self.client.publish_diagnostics(uri, diags, None).await;
    }

    async fn did_change(&self, mut params: DidChangeTextDocumentParams) {
        // self.client
        //     .log_message(MessageType::INFO, "did change")
        //     .await;
        let uri = params.text_document.uri;
        let text = params.content_changes.pop().unwrap().text;
        let mut document = Document::new(text);
        let errors = document.parse();
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
        logger::log(&"document symbol".into());
        let uri = params.text_document.uri;
        let mut document = self.document_map.get_mut(&uri.to_string()).unwrap();
        let document_symbols = features::document_symbol(&uri, document.get_symbol_table());
        Ok(document_symbols)
    }

    async fn references(&self, params: ReferenceParams) -> Result<Option<Vec<Location>>> {
        logger::log(&"references".into());
        let uri = params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;
        let mut document = self.document_map.get_mut(&uri.to_string()).unwrap();
        let locations = features::references(&uri, position, document.get_symbol_table());
        Ok(locations)
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        logger::log(&"goto definition".into());
        let uri = params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;
        let mut document = self.document_map.get_mut(&uri.to_string()).unwrap();
        let definition = features::definitions(&uri, position, document.get_symbol_table());
        Ok(definition)
    }

    async fn semantic_tokens_full(
        &self,
        params: SemanticTokensParams,
    ) -> Result<Option<SemanticTokensResult>> {
        logger::log(&"semantic tokens full".into());
        let uri = params.text_document.uri;
        let mut document = self.document_map.get_mut(&uri.to_string()).unwrap();
        let semantic_tokens = semantic_tokens::semantic_tokens_full(&mut document);
        Ok(Some(SemanticTokensResult::Tokens(SemanticTokens {
            result_id: None,
            data: semantic_tokens,
        })))
    }

    async fn document_highlight(
        &self,
        params: DocumentHighlightParams,
    ) -> Result<Option<Vec<DocumentHighlight>>> {
        logger::log(&"document highlight".into());
        let uri = params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;
        let mut document = self.document_map.get_mut(&uri.to_string()).unwrap();
        let symbol_table = document.get_symbol_table();
        let document_highlights = features::document_highlight(position, symbol_table);
        Ok(document_highlights)
    }

    async fn prepare_rename(
        &self,
        params: TextDocumentPositionParams,
    ) -> Result<Option<PrepareRenameResponse>> {
        logger::log(&"prepare rename".into());
        let uri = params.text_document.uri;
        let position = params.position;
        let mut document = self.document_map.get_mut(&uri.to_string()).unwrap();
        let symbol_table = document.get_symbol_table();
        let prepare_rename_response = features::prepare_rename(position, symbol_table);
        Ok(prepare_rename_response)
    }

    async fn rename(&self, params: RenameParams) -> Result<Option<WorkspaceEdit>> {
        logger::log(&"rename".into());
        let uri = params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;
        let mut document = self.document_map.get_mut(&uri.to_string()).unwrap();
        let symbol_table = document.get_symbol_table();
        let rename_response = features::rename(&uri, position, symbol_table, params.new_name);
        Ok(rename_response)
    }
}
