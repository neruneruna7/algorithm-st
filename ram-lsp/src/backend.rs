use std::collections::HashMap;
use tokio::sync::RwLock;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::{
    CompletionOptions, CompletionParams, CompletionResponse, DidChangeTextDocumentParams,
    DidOpenTextDocumentParams, InitializeParams, InitializeResult, InitializedParams, MessageType,
    ServerCapabilities, TextDocumentSyncCapability, TextDocumentSyncKind, Url,
};
use tower_lsp::{Client, LanguageServer, async_trait};

use crate::completions::completion_items;
use crate::diagnostics::analyze_to_lsp_diagnostics;

/// RAM LSP サーバの状態。
///
/// diagnostics と completion のため、open/change で受け取った全文を URI ごとに保持する。
#[derive(Debug)]
pub struct Backend {
    /// VS Code へ診断やログを返すための LSP クライアントハンドル。
    client: Client,
    /// 開かれている RAM 文書の最新全文。
    ///
    /// LSP の completion request には通常、文書本文そのものは含まれない。
    /// そのため didOpen/didChange で受け取った本文を保持し、補完時に参照する。
    documents: RwLock<HashMap<Url, String>>,
}

impl Backend {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            documents: RwLock::new(HashMap::new()),
        }
    }

    /// RAM ソースを解析し、得られた診断をクライアントへ通知する。
    async fn publish_analysis(&self, uri: Url, text: String) {
        let diagnostics = analyze_to_lsp_diagnostics(&text);
        self.client
            .publish_diagnostics(uri, diagnostics, None)
            .await;
    }

    /// 文書キャッシュを更新してから診断を再計算する。
    ///
    /// 補完と診断は同じ最新版のテキストを見るべきなので、
    /// didOpen/didChange ではこの関数を経由して状態更新と解析をまとめて行う。
    async fn update_document_and_publish_analysis(&self, uri: Url, text: String) {
        self.documents
            .write()
            .await
            .insert(uri.clone(), text.clone());
        self.publish_analysis(uri, text).await;
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
                // completion request を VS Code から受け取れるようにする。
                // resolve_provider は使わず、候補一覧の生成時点ですべての情報を返す。
                completion_provider: Some(CompletionOptions {
                    resolve_provider: Some(false),
                    trigger_characters: None,
                    ..CompletionOptions::default()
                }),
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
        self.update_document_and_publish_analysis(
            params.text_document.uri,
            params.text_document.text,
        )
        .await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        // FULL 同期なので、最後の change に現在の全文が入っている想定で解析する。
        if let Some(change) = params.content_changes.into_iter().last() {
            self.update_document_and_publish_analysis(params.text_document.uri, change.text)
                .await;
        }
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        // completion request はカーソル位置を持つが、現在は文脈判定をしない。
        // URI から文書全文だけを引き、命令候補と文書内ラベル候補を返す。
        let uri = params.text_document_position.text_document.uri;
        let documents = self.documents.read().await;
        let text = documents.get(&uri).map(String::as_str);

        Ok(Some(CompletionResponse::Array(completion_items(text))))
    }
}
