# Panoptic
## Step-by-Step Build Guide for a Markdown Extension for Zed

**Project:** Panoptic  
**Product:** A lightweight Markdown authoring extension for Zed  
**Target editor:** Zed  
**Implementation:** Zed extension (Rust/WASM) + a small native Rust language server  
**Primary goal:** Make working with Markdown in Zed more pleasant while staying inside Zed's supported extension system.  
**Status:** Implementation brief for an AI coding agent or developer

---

## 1. Build this product—not a mockup

Create a real, installable Zed extension named **Panoptic**.

Panoptic should improve Markdown authoring with a small set of dependable features:

1. Completion for relative Markdown file links and heading anchors.
2. Diagnostics for broken local links, broken local image paths, and images without alt text.
3. Hover information for local Markdown links and heading targets.
4. A small set of useful Markdown snippets.
5. Clear instructions for using Zed's existing Markdown preview to read the rendered document.

The extension must work with ordinary `.md` files and must not require users to rename documents, migrate a vault, or use a proprietary file format.

### Important product distinction

The original inspiration is Typora: formatted Markdown that can be edited directly in place.

**Zed's current public extension API does not let a third-party extension replace the editor's text renderer, create a custom editor surface, or implement Typora-style cursor-aware inline rendering.** Zed extensions currently provide capabilities such as language support, language servers, themes, icon themes, snippets, debuggers, and MCP servers. A visual-extension proposal is not a released custom-editor API, and its stated MVP excludes custom editor implementations.

Therefore, this project must not pretend that it can deliver full Typora WYSIWYG through the current extension API. Panoptic's first release is a genuine Zed extension that enhances Markdown authoring and works alongside Zed's built-in rendered Markdown preview.

Do not silently change this project into a Zed fork, a companion desktop app, a browser app, or a VS Code extension. Those are different products.

If native inline WYSIWYG becomes a non-negotiable requirement, stop and propose a separate upstream Zed feature or native Zed contribution. Do not fake it with invisible text colors, redactions, inlay hints, or a side-by-side preview.

---

## 2. Product scope

### 2.1 MVP features

| Feature | Expected behavior | Implementation |
|---|---|---|
| Relative link completion | Suggest files when typing `[text](./...)` | Language Server Protocol (LSP) completion |
| Heading anchor completion | Suggest anchors when typing `#` inside a Markdown link | LSP completion |
| Broken local-link diagnostics | Warn when a relative Markdown link points to a missing file or heading | LSP diagnostics |
| Broken image diagnostics | Warn when a local image target does not exist | LSP diagnostics |
| Missing image alt text | Warn when an image has no meaningful alt text | LSP diagnostics |
| Link hover | Show the resolved local target and whether it exists | LSP hover |
| Markdown snippets | Insert common Markdown structures quickly | Zed snippets |
| Rendered reading | Use Zed's existing Markdown preview | Built-in Zed feature; document the workflow |

### 2.2 Explicitly out of scope for v0.1

Do not build these for the first release:

- Typora-style inline WYSIWYG editing.
- A custom editor, custom panel, webview, or preview renderer.
- A Zed fork or companion desktop application.
- A custom Markdown parser written from scratch.
- Mermaid rendering, LaTeX, math, or interactive diagram editing.
- Wiki-link databases, backlinks, graph views, daily notes, or vault management.
- Cloud sync, accounts, telemetry, collaboration, or databases.
- AI writing features.
- Remote URL crawling or network-based link checking.
- A large settings UI.

Keep the project small. Add a feature only when it solves a real Markdown workflow problem and can be implemented through a documented Zed API.

---

## 3. Technical architecture

Panoptic consists of two small parts:

1. **Zed extension package:** a Rust library compiled to WebAssembly. It tells Zed how to start the language server and downloads the correct native server binary when needed.
2. **Panoptic language server:** a native Rust executable that implements a small subset of LSP for Markdown documents.

```text
                         Zed
                          |
                  Markdown language
                          |
                          v
                 Panoptic extension
                    (Rust / WASM)
                          |
             finds or downloads binary
                          |
                          v
                 panoptic-lsp process
                    (native Rust)
                          |
          +---------------+----------------+
          |               |                |
          v               v                v
      Completion      Diagnostics         Hover
          |               |                |
          +---------------+----------------+
                          |
                          v
                 Normal Markdown file
```

Zed remains responsible for text editing, syntax highlighting, file buffers, undo/redo, and its native Markdown preview. Panoptic does not duplicate those systems.

### 3.1 Source of truth

The `.md` file is always the source of truth.

- Never rewrite a document just to display it.
- Never maintain a second editable HTML copy.
- Never store Markdown content in a database.
- LSP features must not modify a file unless the user explicitly applies a code action or formatting action.
- Diagnostics and completions must not change document contents.

### 3.2 Keep the components separate

**Zed extension layer**

- Implements the Zed `Extension` trait.
- Resolves the platform-specific `panoptic-lsp` binary.
- Checks for a user-installed binary first.
- Downloads an official release binary when necessary.
- Returns the command Zed should execute.

**Language server layer**

- Reads and analyzes Markdown documents.
- Tracks open document text and version.
- Publishes diagnostics.
- Provides link and anchor completions.
- Provides hover information.
- Uses standard input/output for LSP messages.

Do not put Markdown parsing or link-analysis logic in the WebAssembly extension. Keep it in the native language server, where normal Rust libraries and filesystem access are available.

---

## 4. Repository structure

Create one repository with the following minimal structure:

```text
panoptic/
├── extension.toml
├── Cargo.toml                 # Zed extension crate
├── src/
│   └── lib.rs                 # Zed Extension implementation
│
├── snippets/
│   └── markdown.json          # Markdown snippets
│
├── server/
│   ├── Cargo.toml             # Native LSP server crate
│   └── src/
│       ├── main.rs            # LSP process entry point
│       ├── server.rs          # LSP capabilities and request handling
│       ├── document.rs        # Open-document state
│       ├── markdown.rs        # Markdown parsing helpers
│       ├── links.rs           # Link and image target resolution
│       ├── diagnostics.rs     # Diagnostics
│       └── completion.rs      # Link and anchor completion
│
├── tests/
│   ├── fixtures/
│   ├── diagnostics.rs
│   ├── completion.rs
│   └── links.rs
│
├── .github/
│   └── workflows/
│       └── release.yml        # Cross-platform server builds
│
├── README.md
├── LICENSE
└── CHANGELOG.md
```

Do not create extra crates or packages unless the implementation demonstrates a clear need.

---

## 5. Step 1 — Inspect Zed's extension contract

Before writing code, inspect the current official Zed extension documentation and a working language-server extension.

Read:

- https://zed.dev/docs/extensions/developing-extensions
- https://zed.dev/docs/extensions/languages
- https://zed.dev/docs/extensions/snippets
- https://zed.dev/docs/configuring-languages
- https://github.com/Feel-ix-343/markdown-oxide-zed

Use the Markdown Oxide Zed extension as a packaging reference only. Do not copy its product behavior or implementation wholesale. Its manifest demonstrates how an extension can register a language server for Zed's existing `Markdown` language.

### Required feasibility checks

1. Confirm the current Zed extension SDK and compatible version of `zed_extension_api`.
2. Confirm the exact manifest syntax for a language server attached to `Markdown`.
3. Confirm the current `Extension` trait method signature for starting the server.
4. Confirm the platform and architecture enums exposed by the SDK.
5. Confirm the download and release helper APIs available in the SDK.
6. Confirm how Zed loads extension-provided snippets.

Do not rely on old blog posts or obsolete examples when the current official API documentation differs.

**Deliverable:** A short `docs/zed-api-notes.md` recording the verified API signatures, tested Zed version, and any compatibility caveats. Keep it concise.

---

## 6. Step 2 — Create the Zed extension scaffold

Create `extension.toml` at the repository root.

Use this as the starting shape and verify field names against the current Zed documentation:

```toml
id = "panoptic"
name = "Panoptic"
version = "0.1.0"
schema_version = 1
authors = ["YOUR NAME <YOUR EMAIL>"]
description = "Enhanced Markdown authoring support for Zed"
repository = "https://github.com/YOUR-USERNAME/panoptic"
snippets = ["./snippets/markdown.json"]

[language_servers.panoptic]
name = "Panoptic Markdown LSP"
language = "Markdown"
```

Replace the author, repository, and other placeholders with real project details. Do not invent personal information.

Important:

- Attach to Zed's existing `Markdown` language.
- Do not create a competing `Panoptic Markdown` language for ordinary `.md` files unless the current API makes that unavoidable and the trade-off is documented.
- Do not bundle or compile the native language server as the extension's WASM module. They are separate binaries with separate build targets.
- Do not add a custom grammar unless testing proves it is necessary.

Create the extension crate's `Cargo.toml`:

```toml
[package]
name = "panoptic-zed"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
zed_extension_api = "USE_CURRENT_COMPATIBLE_VERSION"
```

Use the latest compatible `zed_extension_api` version confirmed in Step 1. Do not leave the placeholder in the final repository.

---

## 7. Step 3 — Implement the extension's binary resolver

Implement `src/lib.rs`.

The extension should follow this launch order:

1. Reuse a cached binary path if it still exists.
2. Check whether `panoptic-lsp` is already available on the user's `PATH` through Zed's `Worktree::which` API.
3. If not installed, find the latest stable Panoptic GitHub release using the Zed extension API.
4. Select the correct release asset for the current operating system and CPU architecture.
5. Download and extract the asset using the extension SDK's supported download helpers.
6. Verify that the expected executable exists.
7. Cache the resolved path for the lifetime of the extension instance.
8. Return a `zed::Command` that starts `panoptic-lsp` with no shell wrapper.

### Platform targets

The first release should build these targets if the Rust toolchain and dependencies support them:

- macOS Apple Silicon: `aarch64-apple-darwin`
- macOS Intel: `x86_64-apple-darwin`
- Linux x86-64: `x86_64-unknown-linux-gnu`
- Linux ARM64: `aarch64-unknown-linux-gnu`
- Windows x86-64: `x86_64-pc-windows-msvc` or another explicitly tested supported target

If a platform is unsupported, return a readable error instead of selecting an incorrect binary.

### Release asset naming

Use predictable, versioned filenames, for example:

```text
panoptic-lsp-v0.1.0-aarch64-apple-darwin.tar.gz
panoptic-lsp-v0.1.0-x86_64-apple-darwin.tar.gz
panoptic-lsp-v0.1.0-x86_64-unknown-linux-gnu.tar.gz
panoptic-lsp-v0.1.0-aarch64-unknown-linux-gnu.tar.gz
panoptic-lsp-v0.1.0-x86_64-pc-windows-msvc.zip
```

The extension's resolver and release workflow must agree exactly on asset names and internal paths.

### Failure behavior

Handle these cases explicitly:

- No network connection.
- GitHub release unavailable.
- No asset for the current platform.
- Download or extraction failure.
- Missing executable after extraction.
- Unsupported operating system or architecture.
- A cached binary path that no longer exists.

Show a useful error through the extension's returned error. Never silently launch a different executable.

---

## 8. Step 4 — Build the native Markdown language server

Create `server/Cargo.toml` and a native Rust executable called `panoptic-lsp`.

Use a small, maintained LSP implementation such as `tower-lsp` or `lsp-server` with `lsp-types`. Choose one approach; do not combine multiple LSP frameworks.

Use a Markdown parser that provides source ranges, such as `pulldown-cmark`. Enable only the parser extensions required for the initial link and heading analysis. Do not write a Markdown parser from scratch.

### LSP transport

- Communicate over standard input and standard output using the LSP protocol.
- Write logs to standard error only.
- Never print debug messages to standard output.
- Handle shutdown and exit requests cleanly.
- Advertise only the capabilities that are actually implemented.

### Minimum LSP capabilities

Implement only:

- `textDocument/didOpen`
- `textDocument/didChange`
- `textDocument/didClose`
- `textDocument/didSave` (optional if diagnostics are already updated on change)
- `textDocument/publishDiagnostics`
- `textDocument/completion`
- `textDocument/hover`

Do not advertise formatting, rename, code actions, semantic tokens, or workspace symbols until those capabilities are implemented and tested.

### Document state

Track each open document by URI:

```rust
struct OpenDocument {
    uri: Url,
    version: i32,
    text: String,
}
```

Use the document version when publishing diagnostics. Discard analysis results for an older version if a newer change has already arrived.

For the MVP, full-document synchronization is acceptable if it is supported correctly by the chosen LSP framework and Zed. Keep document size reasonable and avoid retaining closed documents indefinitely.

---

## 9. Step 5 — Implement Markdown diagnostics

Diagnostics are the first user-visible feature. Keep them useful and conservative; false positives will make the extension frustrating.

### 9.1 Missing image alt text

Warn for images with empty alt text, for example:

```markdown
![](./images/architecture.png)
```

Do not warn when meaningful alt text is present:

```markdown
![System architecture](./images/architecture.png)
```

Do not automatically rewrite the document.

### 9.2 Broken local links

Check relative file links such as:

```markdown
[Architecture](./architecture.md)
[API reference](../docs/api.md)
```

Resolve them relative to the document containing the link. Report a warning only when the target should be local and does not exist.

### 9.3 Broken local images

Check local image targets:

```markdown
![Diagram](./images/system.png)
```

Report a warning when the referenced local image is missing.

### 9.4 Heading anchors

Support checking fragment-only links and same-document heading references:

```markdown
[Installation](#installation)
```

Generate GitHub-style heading slugs for the initial implementation. Handle duplicate headings with deterministic suffixes such as `-1`, `-2`, and so on. Document the exact slug behavior and test punctuation, Unicode, duplicate headings, and empty headings.

### 9.5 Do not report false positives

The analyzer must:

- Ignore remote `http` and `https` URLs.
- Ignore `mailto:` links.
- Ignore content inside fenced code blocks and inline code.
- Correctly handle reference-style links if supported by the parser.
- Decode URL-encoded local paths before checking the filesystem.
- Treat query strings and fragments separately from the local path.
- Avoid network requests entirely.
- Avoid reading files outside the permitted workspace scope.
- Avoid following symlinks outside the workspace when resolving local targets.
- Avoid marking an external or unsupported URL as a missing local file.

### Diagnostic severity

Use warnings for missing alt text and broken local references. Do not make normal Markdown writing fail or block saving.

---

## 10. Step 6 — Implement link and heading completion

Completion must be contextual. Do not display a large list of unrelated files in every Markdown buffer.

### 10.1 Relative file completion

When the cursor is inside a Markdown link destination:

```markdown
[Architecture](./docs/)
```

Suggest matching files and folders relative to the current document.

Requirements:

- Preserve the user's typed path prefix.
- Handle spaces and URL-encoded filenames.
- Exclude common noisy directories such as `.git`, `node_modules`, `target`, and build output.
- Do not crawl the entire disk.
- Limit results to the current directory or the explicitly typed path prefix.
- Keep completion responsive in large repositories.

### 10.2 Heading anchor completion

When the cursor is inside a link fragment:

```markdown
[Architecture](./architecture.md#)
```

Suggest heading anchors from the target Markdown file.

For same-file links:

```markdown
[Install](#)
```

Suggest anchors from the current document.

### 10.3 Completion correctness

- Return proper LSP completion ranges.
- Do not duplicate file extensions.
- Do not insert malformed parentheses or brackets.
- Preserve URL escaping.
- Do not overwrite text outside the active link destination.
- Return no suggestions when the cursor is not in a supported link context.

---

## 11. Step 7 — Implement hover information

When the user hovers over a local Markdown link, show a small Markdown-formatted hover result containing:

- The resolved target path.
- Whether the target exists.
- The heading fragment, if present.
- The target heading text when it can be resolved.

For remote URLs, show the URL as text only. Do not fetch websites, execute scripts, or download remote content.

Keep the hover concise. Do not turn it into a full document preview.

---

## 12. Step 8 — Add a small snippet set

Create `snippets/markdown.json` and register it in `extension.toml`.

Include only useful, predictable snippets:

| Trigger | Expansion |
|---|---|
| `h1` | Level-one heading |
| `h2` | Level-two heading |
| `link` | Markdown link |
| `image` | Image with alt text and path placeholders |
| `task` | Task-list item |
| `quote` | Blockquote |
| `code` | Fenced code block with language placeholder |
| `table` | Small two-column Markdown table |

Snippet requirements:

- Use tab stops and sensible placeholders.
- Do not insert unnecessary blank lines.
- Do not add a giant template when a short snippet is enough.
- Follow Zed's current snippet JSON schema exactly.
- Verify every snippet in a real `.md` buffer.

Do not create a custom command or toolbar for snippets. Use Zed's native snippet completion behavior.

---

## 13. Step 9 — Use Zed's built-in Markdown preview

Panoptic should complement the built-in preview rather than reimplement it.

The README must explain:

1. Open any ordinary `.md` file in Zed.
2. Use Zed's Command Palette.
3. Search for the built-in Markdown preview action.
4. Open the rendered preview.
5. Continue editing the original source buffer in Zed.

Do not claim the preview is a WYSIWYG editor. It is a rendered reading view, separate from source editing.

Panoptic's role is to improve the source-authoring workflow through completions, diagnostics, hover information, and snippets.

---

## 14. Step 10 — Test the language server independently

Before testing inside Zed, test `panoptic-lsp` as a standalone LSP process.

### Unit tests

Test at least:

- Relative path resolution.
- URL-encoded paths.
- Local links with fragments.
- Remote URL exclusion.
- `mailto:` exclusion.
- Missing image alt text.
- Existing and missing image files.
- Heading slug generation.
- Duplicate heading slugs.
- Link completion path filtering.
- Heading completion.
- Code fences and inline code not producing diagnostics.
- Unicode text and source ranges.

### Integration tests

Use an LSP test client or protocol harness to verify:

1. Initialize handshake succeeds.
2. Server advertises only working capabilities.
3. Opening a Markdown document publishes diagnostics.
4. Changing a document updates diagnostics with the correct version.
5. Closing a document releases its state.
6. Completion results contain correct ranges and insertion text.
7. Hover results are valid LSP markup.
8. Shutdown and exit terminate cleanly.

### Test fixtures

Create a small fixture workspace:

```text
fixtures/workspace/
├── README.md
├── guide.md
├── docs/
│   └── architecture.md
├── images/
│   └── existing.png
└── broken-links.md
```

Keep fixtures deterministic. Do not rely on external websites or network access in tests.

---

## 15. Step 11 — Test the extension inside Zed

Install Panoptic locally as a development extension.

1. Open Zed.
2. Open the Extensions view from the Command Palette.
3. Choose **Install Dev Extension**.
4. Select the root `panoptic/` directory containing `extension.toml`.
5. Open a normal `.md` file.
6. Confirm Zed starts **Panoptic Markdown LSP** for the `Markdown` language.
7. Test completion, diagnostics, hover, and snippets.
8. Open Zed's log if the server does not start.

Test at minimum:

- macOS Apple Silicon, because it is a primary target.
- One Linux target.
- Windows if included in the first release matrix.

Verify that Panoptic does not break the built-in Markdown language, existing Markdown preview, other installed language servers, or ordinary text editing.

### Logs and debugging

- Use Zed's `zed: open log` action to inspect extension and language-server startup errors.
- For more verbose output, launch Zed from a terminal with `zed --foreground` where supported.
- LSP protocol output must remain on stdout; diagnostic logs go to stderr.

---

## 16. Step 12 — Package and release native server binaries

The Zed extension package is compiled to WebAssembly, but `panoptic-lsp` is a native executable. The release workflow must build and publish the native executable for each supported platform.

Create `.github/workflows/release.yml`.

The release workflow should:

1. Run tests before building release assets.
2. Build the server for each supported OS/architecture target.
3. Package the executable in the agreed archive format.
4. Use the exact asset names expected by `src/lib.rs`.
5. Publish assets to a GitHub Release tagged with the same version as `extension.toml`.
6. Fail if any required asset is missing.

Do not publish a release whose extension manifest version and binary release version disagree.

The extension must use the Zed Extension API's supported release/download helpers. Do not use arbitrary shell commands from the WASM extension to download or install binaries.

---

## 17. Step 13 — Documentation and publishing

Create a concise but complete `README.md` containing:

- What Panoptic does.
- What Panoptic does not do.
- Supported Zed version, if a minimum version is needed.
- Installation instructions.
- How to use link and heading completions.
- What each diagnostic means.
- How to use snippets.
- How to open Zed's built-in Markdown preview.
- Supported operating systems and architectures.
- Troubleshooting steps.
- Known limitations, especially the lack of inline WYSIWYG rendering through the current extension API.

Before publishing, follow the current Zed extension publishing requirements:

https://zed.dev/docs/extensions/publishing

Do not claim registry publication until the extension repository has actually been accepted and published.

---

## 18. Suggested implementation order

Follow this order. Do not build everything in parallel.

### Milestone 1 — Extension loads

- Repository scaffold exists.
- `extension.toml` is valid.
- Rust extension compiles for the current Zed extension target.
- Dev extension installs in Zed.
- Zed recognizes the extension.

**Exit condition:** Panoptic appears as an installed development extension without manifest or loading errors.

### Milestone 2 — Language server starts

- Native `panoptic-lsp` binary runs over stdio.
- The extension finds or downloads the correct binary.
- Zed starts the server for the built-in Markdown language.
- Initialize/shutdown handshake works.

**Exit condition:** Zed's log confirms that Panoptic's server starts successfully for a normal `.md` file.

### Milestone 3 — Diagnostics work

- Missing alt text warning.
- Broken local link warning.
- Broken local image warning.
- No false positives for remote URLs or code examples.

**Exit condition:** Diagnostics appear at the correct ranges and update after edits.

### Milestone 4 — Completions and hover work

- Relative file link completion.
- Heading anchor completion.
- Local-link hover.

**Exit condition:** Completions insert valid Markdown link destinations and hover output resolves correctly.

### Milestone 5 — Snippets and documentation

- Snippets install and expand.
- README explains the built-in preview workflow.
- Known limitations are explicit.

**Exit condition:** A new user can install Panoptic and use its MVP without reading the source code.

### Milestone 6 — Release

- CI tests pass.
- Native binaries exist for every declared platform.
- Manifest and release versions match.
- Publishing requirements are checked.

**Exit condition:** A tagged release can be installed without manually building the native language server.

---

## 19. Quality requirements

### Performance

- Do not scan the entire filesystem on every keystroke.
- Keep completion scoped to the active relative path.
- Cache parsed headings for open documents.
- Avoid re-analyzing unchanged documents.
- Avoid expensive synchronous work in the LSP request path.
- Handle large Markdown documents without blocking the server indefinitely.

### Reliability

- Unknown Markdown syntax must not crash the server.
- Missing files should produce diagnostics, not panics.
- Malformed URLs should be ignored safely or reported as unsupported.
- One bad document must not terminate the LSP process.
- Old document versions must not overwrite newer diagnostics.

### Privacy and security

- No analytics or telemetry in v0.1.
- No account system.
- No external network requests for link checking.
- No execution of HTML, JavaScript, shell commands, or embedded Markdown content.
- Resolve local paths relative to the current document and enforce workspace boundaries.
- Do not read arbitrary files outside the workspace for hover or completion.
- Only download the Panoptic server binary from the project's official release repository.

---

## 20. Definition of done

Panoptic v0.1 is complete when:

- It is a real Zed extension with a valid `extension.toml`.
- It installs through Zed's **Install Dev Extension** workflow.
- It attaches to Zed's existing `Markdown` language without requiring `.md` files to be renamed.
- It starts the correct native language-server binary on supported platforms.
- It provides relative link completion.
- It provides heading-anchor completion.
- It reports broken local links and images.
- It reports images without alt text.
- It provides useful local-link hover information.
- Its Markdown snippets work.
- It does not break Zed's normal source editing or built-in Markdown preview.
- Tests cover path resolution, diagnostics, completions, and LSP lifecycle.
- README documents installation, use, limitations, and troubleshooting.

**A preview pane alone is not a Panoptic feature. Zed already provides Markdown preview. Panoptic's value must come from its own working extension features.**

---

## 21. Non-negotiable instructions for the coding agent

When implementing this document:

1. Inspect the existing repository before creating files. Do not overwrite unrelated work.
2. Verify current Zed APIs and toolchain requirements before coding.
3. Build a real extension and a real LSP server; do not stop at a plan or mockup.
4. Complete one milestone at a time and run its tests before proceeding.
5. Do not create a fork, companion app, webview, or custom renderer.
6. Do not claim Typora-style WYSIWYG. The current public extension API does not expose the necessary editor surface.
7. If a required API is missing, document the blocker and use only the supported fallback described here.
8. Do not invent Zed commands, manifest fields, extension hooks, or SDK methods. Verify them in current documentation and examples.
9. Keep dependencies and architecture minimal.
10. At the end, report exactly what works, what was tested, supported platforms, and remaining limitations.

---

## 22. Official references

- [Zed — Developing Extensions](https://zed.dev/docs/extensions/developing-extensions)
- [Zed — Language Extensions](https://zed.dev/docs/extensions/languages)
- [Zed — Snippets](https://zed.dev/docs/extensions/snippets)
- [Zed — Configuring Languages](https://zed.dev/docs/configuring-languages)
- [Zed — Extension Publishing](https://zed.dev/docs/extensions/publishing)
- [Zed — Markdown Language](https://zed.dev/docs/languages/markdown)
- [Markdown Oxide Zed extension](https://github.com/Feel-ix-343/markdown-oxide-zed) — packaging and LSP-launch reference
- [Zed Visual Extension API discussion](https://github.com/zed-industries/zed/discussions/53403) — proposal, not a custom-editor API
- [Zed Markdown renderer discussion](https://github.com/zed-industries/zed/discussions/27163) — extension access limitations

---

**Panoptic's v0.1 principle:** Ship a small, reliable, installable Markdown extension that works with Zed today. Do not promise a Typora clone until Zed provides an extension API capable of building one.
