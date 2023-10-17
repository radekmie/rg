use std::collections::HashMap;
use std::rc::Rc;

use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::notification::Notification;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};
use super::document::Document;

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
                inlay_hint_provider: None,
                text_document_sync: None,
                completion_provider: None,
                execute_command_provider: None,
                workspace: None,
                semantic_tokens_provider: None,
                // definition: Some(GotoCapability::default()),
                definition_provider: None,
                references_provider: None,
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
        let uri = params.text_document.uri.clone();
        let text = params.text_document.text;
        let document = Document::new(text);
        self.document_map.insert(uri.to_string(), document);
    }

    async fn did_change(&self, mut params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri.clone();
        let text = params.content_changes.pop().unwrap().text;
        let document = Document::new(text);
        self.document_map.insert(uri.to_string(), document);
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let uri = params.text_document.uri;
        self.document_map.remove(&uri.to_string());
    }
}
