# Zed API notes

Verified against `zed_extension_api = "0.7.0"` (crates.io, current at build time) by compiling `src/lib.rs` for `wasm32-wasip2`. Has been  verified inside a running Zed (see README "Status").

- Extension crate: `crate-type = ["cdylib"]`, target `wasm32-wasip2`.
- Trait: `Extension::language_server_command(&mut self, &LanguageServerId, &Worktree) -> Result<Command>`.
- Helpers used: `Worktree::which`, `latest_github_release(repo, GithubReleaseOptions { require_assets, pre_release })`, `download_file(url, dir, DownloadedFileType::{GzipTar,Zip})`, `make_file_executable`, `set_language_server_installation_status`, `current_platform() -> (Os, Architecture)`.
- Manifest: `[language_servers.<id>] name = "...", language = "Markdown"` attaches to the built-in language (same as the Markdown Oxide extension). `snippets = ["./snippets/markdown.json"]`.
- Snippet JSON: VS Code format (`prefix`, `body`, `description`).
- Downloads extract into the extension's working directory, so the resolver uses a per-version directory and removes older ones.
- Caveat: `tests/` lives in `server/tests/` rather than the repo root, because a root `tests/` would belong to the WASM extension crate.
