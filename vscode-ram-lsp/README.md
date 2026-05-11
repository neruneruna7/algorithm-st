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

If the extension cannot find the server automatically, set `ramLsp.serverPath`
to the absolute path of the `ram-lsp` executable.

## Packaging With Bundled Server

The extension can package a platform-specific `ram-lsp` binary under
`server/<platform>-<arch>/`.

For the current machine:

```sh
cargo build -p ram-lsp --release
cd vscode-ram-lsp
npm run bundle:server
npm run compile
npx @vscode/vsce package
```

On macOS arm64, this creates:

```text
server/darwin-arm64/ram-lsp
```

After installing the generated `.vsix`, the extension uses the bundled binary
unless `ramLsp.serverPath` is explicitly set.

To distribute to other operating systems or CPU architectures, build
`ram-lsp` for each target and place each binary under the corresponding
directory, for example:

```text
server/darwin-arm64/ram-lsp
server/darwin-x64/ram-lsp
server/linux-x64/ram-lsp
server/win32-x64/ram-lsp.exe
```
