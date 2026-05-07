# RAM LSP Client

VS Code client extension for `ram-lsp`.

## Development

Build the language server first:

```sh
cargo build -p ram-lsp
```

Then install and compile the extension:

```sh
cd vscode-ram-lsp
npm install
npm run compile
```

Open this directory in VS Code and press `F5` to launch an Extension Development Host.

If the extension cannot find the server automatically, set `ramLsp.serverPath` to the absolute path of the `ram-lsp` executable.
