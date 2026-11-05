# Panoptic

A lightweight Markdown authoring extension for [Zed](https://zed.dev). It works on ordinary `.md` files and adds link completion, diagnostics, hover and snippets on top of Zed's built-in Markdown support.

## What it does

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

## What it does not do

No Typora-style inline WYSIWYG. Zed's extension API does not allow custom editor surfaces. Also no custom preview, wiki-links, backlinks, formatting or code actions.

## Rendered preview

Use Zed's built-in preview: open a `.md` file, open the Command Palette, search for the Markdown preview action (`markdown: open preview`), and keep editing the source buffer. The preview is a reading view, not a WYSIWYG editor.

## Installation

1. Command Palette → `zed: extensions` → **Install Dev Extension** → choose this repository's root (the folder with `extension.toml`). Zed needs Rust installed via `rustup` to compile the extension.
2. Panoptic uses `panoptic-lsp` from your `PATH` if present; otherwise it downloads the binary from the latest GitHub release of `kaiju-no-9/panoptic`.
3. To build the server yourself: `cargo install --path server` (or `cargo build --release --manifest-path server/Cargo.toml` and put `server/target/release/panoptic-lsp` on your `PATH`).

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

## Status and known limitations

- The server is tested standalone (unit tests and an in-process LSP lifecycle test). The extension compiles to WASM. **It has not yet been run inside Zed**, so install it as a dev extension and verify there.
- The repository URL (`kaiju-no-9/panoptic`) in `extension.toml` and `src/lib.rs` is assumed from your git remote's owner. Change both if it is wrong.
- Not published to the Zed extension registry.
- Full-document sync, and documents are re-analysed on each change; very large files may be slow.
- No inline WYSIWYG rendering.
