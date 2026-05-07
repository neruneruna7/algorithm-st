# RAM for Zed

Zed dev extension for RAM files.

## What It Provides

- Associates `*.ram` files with the `RAM` language.
- Starts the existing `ram-lsp` language server.
- Provides basic Tree-sitter highlighting through the local RAM grammar.

## Development

Build the language server first:

```sh
cargo build -p ram-lsp
```

Start Zed with `ram-lsp` on `PATH`:

```sh
PATH="/Users/kino/workspace/univ/m1/algorithm-st/target/debug:$PATH" zed --foreground
```

Then run `zed: install dev extension` and select this directory:

```text
/Users/kino/workspace/univ/m1/algorithm-st/zed-ram-lsp
```

If the language server cannot start, open `zed: open log` and check whether
`ram-lsp` was found on `PATH`.

## Publishing Note

This extension currently references the Tree-sitter grammar through a local
`file://` URL for development. Before publishing, move the grammar to a public
repository and replace the grammar entry in `extension.toml` with a pinned
repository URL and revision.
