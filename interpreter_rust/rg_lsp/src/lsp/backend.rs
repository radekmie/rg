use std::collections::HashMap;
use std::rc::Rc;

use super::document::Document;
use super::features;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::notification::Notification;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

struct Backend {
    client: Client,
    document_map: DashMap<String, Document>,
    // ast_map: DashMap<String, HashMap<String, Func>>,
    // semantic_token_map: DashMap<String, Vec<ImCompleteSemanticToken>>,
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
                completion_provider: None,
                execute_command_provider: None,
                semantic_tokens_provider: None,
                definition_provider: Some(OneOf::Left(true)),
                references_provider: Some(OneOf::Left(true)),
                rename_provider: None,
                ..ServerCapabilities::default()
            },
        })
    }
    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "initialized!")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri.to_string();
        let text = params.text_document.text;
        let document = Document::new(text);
        self.document_map.insert(uri, document);
    }

    async fn did_change(&self, mut params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri.to_string();
        let text = params.content_changes.pop().unwrap().text;
        let document = Document::new(text);
        self.document_map.insert(uri, document);
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
        let uri = params.text_document.uri;
        let mut document = self.document_map.get_mut(&uri.to_string()).unwrap();
        let document_symbols = features::document_symbol(&uri, document.get_symbol_table());
        Ok(document_symbols)
    }

    async fn references(&self, params: ReferenceParams) -> Result<Option<Vec<Location>>> {
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
        let uri = params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;
        let mut document = self.document_map.get_mut(&uri.to_string()).unwrap();
        let definition = features::definitions(&uri, position, document.get_symbol_table());
        Ok(definition)
    }
}
