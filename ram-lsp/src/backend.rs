use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::{
    DidChangeTextDocumentParams, DidOpenTextDocumentParams, InitializeParams, InitializeResult,
    InitializedParams, MessageType, ServerCapabilities, TextDocumentSyncCapability,
    TextDocumentSyncKind, Url,
};
use tower_lsp::{Client, LanguageServer, async_trait};

use crate::diagnostics::analyze_to_lsp_diagnostics;

/// RAM LSP サーバの状態。
///
/// 現段階では文書キャッシュを持たず、open/change で渡された全文を即時解析する。
#[derive(Debug)]
pub struct Backend {
    client: Client,
}

impl Backend {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    /// RAM ソースを解析し、得られた診断をクライアントへ通知する。
    async fn publish_analysis(&self, uri: Url, text: String) {
        let diagnostics = analyze_to_lsp_diagnostics(&text);
        self.client
            .publish_diagnostics(uri, diagnostics, None)
            .await;
    }
}

#[async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                // 実装を単純に保つため、まずは全文同期だけをサポートする。
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                ..ServerCapabilities::default()
            },
            server_info: None,
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "RAM language server initialized")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        // 開いた時点の全文を解析し、初回診断を出す。
        self.publish_analysis(params.text_document.uri, params.text_document.text)
            .await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        // FULL 同期なので、最後の change に現在の全文が入っている想定で解析する。
        if let Some(change) = params.content_changes.into_iter().last() {
            self.publish_analysis(params.text_document.uri, change.text)
                .await;
        }
    }
}
