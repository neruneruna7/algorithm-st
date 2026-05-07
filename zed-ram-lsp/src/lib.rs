use zed_extension_api as zed;

/// RAM 用 Zed 拡張の本体。
///
/// この型自体は状態を持たない。現在の役割は、Zed から要求されたときに
/// `ram-lsp` language server の起動コマンドを返すことだけである。
struct RamExtension;

impl RamExtension {
    /// 起動する language server の実行ファイル名。
    ///
    /// Zed 拡張は WebAssembly として動くため、この Rust コードから
    /// ワークスペース内の `target/debug/ram-lsp` を直接相対パスで参照しない。
    /// 代わりに Zed が渡す worktree の環境から `PATH` 検索する。
    const SERVER_BINARY_NAME: &'static str = "ram-lsp";
}

impl zed::Extension for RamExtension {
    fn new() -> Self {
        Self
    }

    fn language_server_command(
        &mut self,
        _language_server_id: &zed::LanguageServerId,
        worktree: &zed::Worktree,
    ) -> zed::Result<zed::Command> {
        // Zed extension policy expects language servers to be discovered from
        // the user environment or downloaded, not bundled into the extension.
        // For local development, build this repository's ram-lsp binary and put
        // it on PATH before installing this directory as a dev extension.
        //
        // 事実として、Zed の公開拡張では language server を拡張に同梱しない方針が
        // ドキュメントに示されている。ここではダウンロード処理までは実装せず、
        // ローカル開発を優先して `PATH` から探す設計にしている。
        let command = worktree.which(Self::SERVER_BINARY_NAME).ok_or_else(|| {
            format!(
                "Could not find `{}` in PATH. Run `cargo build -p ram-lsp` and add \
                 `target/debug` or `target/release` to PATH before starting Zed.",
                Self::SERVER_BINARY_NAME
            )
        })?;

        Ok(zed::Command {
            command,
            // ram-lsp は stdio LSP サーバなので、追加引数は不要である。
            args: Vec::new(),
            // ユーザーの shell 環境を引き継ぐことで、PATH などの設定を反映する。
            env: worktree.shell_env(),
        })
    }
}

// Zed が WebAssembly extension としてこの型を認識できるように登録する。
zed::register_extension!(RamExtension);
