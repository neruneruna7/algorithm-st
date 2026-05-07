mod backend;
mod completions;
mod diagnostics;

use tower_lsp::{LspService, Server};

use crate::backend::Backend;

#[tokio::main]
async fn main() {
    // LSP は標準入力・標準出力を通じてエディタと JSON-RPC 通信する。
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    // `Backend` が LSP の各リクエスト・通知を処理する実体になる。
    let (service, socket) = LspService::new(Backend::new);
    Server::new(stdin, stdout, socket).serve(service).await;
}
