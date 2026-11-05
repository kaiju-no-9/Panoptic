use std::fs;
use zed_extension_api::{
    self as zed, Architecture, Command, DownloadedFileType, GithubReleaseOptions, LanguageServerId,
    LanguageServerInstallationStatus, Os, Result, Worktree,
};

/// The only repository binaries are ever downloaded from.
const REPO: &str = "kaiju-no-9/panoptic";
const BINARY: &str = "panoptic-lsp";

struct Panoptic {
    cached_binary_path: Option<String>,
}

/// Rust target triple and archive type for the current platform.
fn platform_target() -> Result<(&'static str, bool)> {
    let (os, arch) = zed::current_platform();
    match (os, arch) {
        (Os::Mac, Architecture::Aarch64) => Ok(("aarch64-apple-darwin", false)),
        (Os::Mac, Architecture::X8664) => Ok(("x86_64-apple-darwin", false)),
        (Os::Linux, Architecture::X8664) => Ok(("x86_64-unknown-linux-gnu", false)),
        (Os::Linux, Architecture::Aarch64) => Ok(("aarch64-unknown-linux-gnu", false)),
        (Os::Windows, Architecture::X8664) => Ok(("x86_64-pc-windows-msvc", true)),
        (os, arch) => Err(format!(
            "Panoptic does not provide a language server for this platform ({os:?}, {arch:?}). \
             Build `panoptic-lsp` from source and put it on your PATH."
        )),
    }
}

fn is_file(path: &str) -> bool {
    fs::metadata(path).map_or(false, |m| m.is_file())
}

impl Panoptic {
    fn binary_path(&mut self, id: &LanguageServerId, worktree: &Worktree) -> Result<String> {
        // 1. Cached path, if still present.
        if let Some(path) = &self.cached_binary_path {
            if is_file(path) {
                return Ok(path.clone());
            }
            self.cached_binary_path = None;
        }

        // 2. A user-installed binary on PATH.
        if let Some(path) = worktree.which(BINARY) {
            self.cached_binary_path = Some(path.clone());
            return Ok(path);
        }

        // 3. Latest stable release from the official repository.
        let (target, is_zip) = platform_target()?;
        zed::set_language_server_installation_status(
            id,
            &LanguageServerInstallationStatus::CheckingForUpdate,
        );
        let release = zed::latest_github_release(
            REPO,
            GithubReleaseOptions {
                require_assets: true,
                pre_release: false,
            },
        )
        .map_err(|e| {
            format!("could not fetch the latest Panoptic release from github.com/{REPO}: {e}")
        })?;

        let ext = if is_zip { "zip" } else { "tar.gz" };
        let asset_stem = format!("{BINARY}-{}-{target}", release.version);
        let asset_name = format!("{asset_stem}.{ext}");
        let asset = release
            .assets
            .iter()
            .find(|a| a.name == asset_name)
            .ok_or_else(|| {
                format!(
                    "release {} has no asset named {asset_name} for this platform",
                    release.version
                )
            })?;

        let exe = if is_zip {
            format!("{BINARY}.exe")
        } else {
            BINARY.to_string()
        };
        let binary_path = format!("{asset_stem}/{exe}");

        if !is_file(&binary_path) {
            zed::set_language_server_installation_status(
                id,
                &LanguageServerInstallationStatus::Downloading,
            );
            let result = zed::download_file(
                &asset.download_url,
                &asset_stem,
                if is_zip {
                    DownloadedFileType::Zip
                } else {
                    DownloadedFileType::GzipTar
                },
            );
            if let Err(e) = result {
                zed::set_language_server_installation_status(
                    id,
                    &LanguageServerInstallationStatus::Failed(e.clone()),
                );
                return Err(format!("failed to download {asset_name}: {e}"));
            }
            if !is_file(&binary_path) {
                return Err(format!(
                    "downloaded {asset_name} but the expected executable {binary_path} is missing"
                ));
            }
            zed::make_file_executable(&binary_path)
                .map_err(|e| format!("failed to mark {binary_path} executable: {e}"))?;

            // Remove older downloaded versions.
            if let Ok(entries) = fs::read_dir(".") {
                for entry in entries.flatten() {
                    let name = entry.file_name();
                    let name = name.to_string_lossy();
                    if name.starts_with(BINARY) && name != asset_stem {
                        fs::remove_dir_all(entry.path()).ok();
                    }
                }
            }
        }

        zed::set_language_server_installation_status(id, &LanguageServerInstallationStatus::None);
        self.cached_binary_path = Some(binary_path.clone());
        Ok(binary_path)
    }
}

impl zed::Extension for Panoptic {
    fn new() -> Self {
        Self {
            cached_binary_path: None,
        }
    }

    fn language_server_command(
        &mut self,
        id: &LanguageServerId,
        worktree: &Worktree,
    ) -> Result<Command> {
        Ok(Command {
            command: self.binary_path(id, worktree)?,
            args: Vec::new(),
            env: Vec::new(),
        })
    }
}

zed::register_extension!(Panoptic);
