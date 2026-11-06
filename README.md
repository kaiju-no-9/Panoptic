<div align="center">

# Panoptic

### Markdown authoring support for Zed

Link completion, diagnostics, hover details, and snippets for ordinary Markdown files.

[![Release](https://img.shields.io/github/v/release/kaiju-no-9/panoptic?label=release)](https://github.com/kaiju-no-9/panoptic/releases)
[![Release workflow](https://github.com/kaiju-no-9/panoptic/actions/workflows/release.yml/badge.svg)](https://github.com/kaiju-no-9/panoptic/actions/workflows/release.yml)
![Rust](https://img.shields.io/badge/Rust-extension%20%2B%20LSP-000000?logo=rust)

</div>
<img width="1469" height="700" alt="22" src="https://github.com/user-attachments/assets/22728823-3f55-457e-8288-161564e95e15" />

---
## Demo

See Panoptic in action:

[▶ Watch the demo on X](https://x.com/kumar_nish98725/status/2118458330440200654)

The demo walks through Panoptic's Markdown authoring experience in Zed, including its editor assistance and navigation features.

## What is Panoptic?

Panoptic is a Zed extension that makes writing Markdown more reliable. It attaches to Zed's built-in **Markdown** language and works with regular `.md` files. The extension provides editor assistance through a small Rust language server and Zed's native snippet support.

## Features

| Capability | What Panoptic does |
|---|---|
| **Link completion** | Suggests relative files and folders inside Markdown link destinations. |
| **Heading completion** | Suggests same-file or target-file anchors after `#` in a link. |
| **Diagnostics** | Warns about missing image alt text, broken local links and images, and missing heading anchors. |
| **Hover** | Shows local target details and heading information; remote URLs are shown as text only. |
| **Quick fixes** | Offers a code action to add placeholder alt text to an image flagged for missing alt text. |
| **Navigation** | Go to the local file or matching heading from a link; find references to the same target. |
| **Project navigation** | Lists document headings in file symbols and workspace symbol search. |
| **Snippets** | Provides `h1`, `h2`, `link`, `image`, `task`, `quote`, `code`, and `table`. |
| **Rendered reading** | Works alongside Zed's built-in Markdown preview. |

Heading anchors use GitHub-style slugs: lowercase, spaces become hyphens, punctuation is dropped, Unicode letters are kept, and repeated headings receive suffixes such as `-1` and `-2`.

Remote URLs, `mailto:` links, links inside code, and targets outside the workspace are excluded from local-reference diagnostics. Reference-style Markdown links participate in parsing and navigation. Panoptic does not fetch URLs, send telemetry, or modify files automatically.

## Install for development

Panoptic is not published in Zed's extension registry yet. To install it locally:

1. Clone this repository and open it in Zed.
2. Open the Command Palette with **Cmd+Shift+P** and run **zed: install dev extension**.
3. Select the repository directory containing `extension.toml`.
4. Open a `.md` file. Panoptic uses `panoptic-lsp` on your `PATH` if available; otherwise, it downloads the matching binary from the [latest GitHub release](https://github.com/kaiju-no-9/panoptic/releases/latest).

Zed builds the extension for `wasm32-wasip2`. When developing, install Rust with `rustup`; Zed can install the target automatically.

To build the language server locally and avoid downloading a release binary:

```sh
cargo build --release --manifest-path server/Cargo.toml
```

Add `server/target/release` to `PATH` so Zed can find `panoptic-lsp`.

## Try it

In a Markdown source file inside the project, add a link to a file that does not exist:

```md
[missing](./does-not-exist.md)
```

Panoptic should underline the destination. Hover over it to read the warning, or press **Cmd+Shift+M** to open project diagnostics. You can try the other features with these examples:

```md
![](./missing-image.png)
[section](./README.md#)
```

Type `h1` or `link` in a Markdown buffer to try snippets. For rendered reading, use **Cmd+Shift+P → `markdown: open preview`**. The preview is separate from the source editor; Panoptic does not provide inline WYSIWYG editing.

## How it works

```text
Zed Markdown buffer
        |
        +-- Panoptic extension (Rust compiled to WASM)
        |       finds or downloads the native server
        |
        +-- panoptic-lsp (native Rust process)
                link and heading analysis
                diagnostics, completions, and hover
```

Zed remains responsible for editing, syntax highlighting, file buffers, and the rendered preview. The Markdown file stays the source of truth.

## Supported release targets

The GitHub release workflow builds the native language server for:

- macOS Apple Silicon and Intel
- Linux x86-64 and ARM64
- Windows x86-64

Panoptic has been manually verified in Zed on macOS Apple Silicon, including server startup and a broken-link warning. Other platforms are release build targets and still need hands-on validation.

## Develop and test

Run the language-server tests and build the extension with:

```sh
cargo test --manifest-path server/Cargo.toml
rustup target add wasm32-wasip2
cargo build --release --target wasm32-wasip2
```

To build only the native server, run `cargo build --release --manifest-path server/Cargo.toml`.

## Limitations

- No Typora-style inline WYSIWYG editing; Zed's public extension API does not provide a custom editor surface.
- No custom Markdown preview, wiki-link syntax, formatting, or automatic edits. The missing-alt quick fix inserts editable placeholder text.
- Documents use full-text synchronization and are re-analyzed after changes, so very large Markdown files may take longer.
- The extension has not been published to the Zed registry.

## Troubleshooting

- Use **Cmd+Shift+P → `zed: open log`** and search for `Panoptic` if the language server does not start.
- If Zed reports a missing release asset, check the [GitHub releases](https://github.com/kaiju-no-9/panoptic/releases) for an asset matching your platform, or build the server locally and add it to `PATH`.
- Links that leave the workspace are intentionally not reported.
