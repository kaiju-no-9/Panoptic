# Panoptic

Panoptic adds practical Markdown authoring features to [Zed](https://zed.dev): local link and heading completion, diagnostics for broken references, link hover details, and snippets. It works with ordinary `.md` files and Zed's built-in Markdown language.

## Features

- **Completion** of relative file links `[text](./docs/…)` and heading anchors `[text](./file.md#…)` / `[text](#…)`.
- **Diagnostics** (warnings only):
  - `missing-alt`: image has empty alt text.
  - `broken-link`: relative link target does not exist.
  - `broken-image`: local image file does not exist.
  - `broken-anchor`: `#fragment` matches no heading (in this file or the target Markdown file).
- **Hover** on a link: resolved path, whether it exists, and the target heading. Remote URLs are shown as text; nothing is fetched.
- **Snippets**: `h1`, `h2`, `link`, `image`, `task`, `quote`, `code`, `table`.

Anchors use GitHub-style slugs: lowercase, spaces to `-`, punctuation dropped, Unicode letters kept, duplicates get `-1`, `-2`, …. Empty headings produce no anchor.

Remote URLs, `mailto:` links, links inside code, and targets outside the workspace are never reported. No network access, telemetry or file modification.

## Scope

Panoptic does not provide Typora-style inline WYSIWYG editing. Zed's extension API does not expose a custom editor surface. It also does not add a custom preview, wiki-links, backlinks, formatting, or code actions.

## Rendered preview

Open a `.md` file, open Zed's Command Palette, and search for **Markdown: Open Preview**. The rendered preview is a reading view; edit the original Markdown source buffer as usual.

## Installation

1. In Zed's Command Palette, run **zed: extensions**, choose **Install Dev Extension**, and select this repository (the directory containing `extension.toml`). Zed compiles the extension; install Rust with `rustup` if needed.
2. Panoptic uses `panoptic-lsp` from `PATH` when available. Otherwise, it downloads a matching binary from the latest stable GitHub release of `kaiju-no-9/panoptic`.
3. To build the server locally, run `cargo build --release --manifest-path server/Cargo.toml`, then add `server/target/release` to `PATH`. Alternatively, run `cargo install --path server`.

## Supported platforms

macOS (Apple Silicon, Intel), Linux (x86-64, ARM64), Windows (x86-64). Other platforms get a readable error; build the server from source instead.

## Troubleshooting

- Run `zed: open log` and look for `Panoptic Markdown LSP` or extension errors. `zed --foreground` gives more output.
- "no asset" / download errors: no release exists yet or no network. Build the server from source and put it on `PATH`.
- No diagnostics for `../` links that leave the workspace: this is intentional.

## Development

```sh
cargo test --manifest-path server/Cargo.toml     # server unit + integration tests
cargo build --release --target wasm32-wasip2     # extension (needs: rustup target add wasm32-wasip2)
```

## Current status and remaining work

- Server unit and integration tests, including an in-process LSP lifecycle test, are present. The extension has been built for WASM. **It still needs validation inside Zed** as a development extension.
- The release workflow defines builds for macOS (Apple Silicon and Intel), Linux (x86-64 and ARM64), and Windows (x86-64). These release artifacts still need a tagged release and platform validation.
- Panoptic has not been published to the Zed extension registry.
- The server uses full-document synchronization and re-analyzes a document after changes; very large files may be slower.
- Inline WYSIWYG rendering is outside the current Zed extension API and is not part of this project.
