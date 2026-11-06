use crate::document::LineIndex;
use crate::links::{self, Dest, Resolution, Workspace};
use crate::markdown;
use lsp_types::Url;
use lsp_types::{
    CodeAction, CodeActionKind, Diagnostic, DocumentSymbol, Location, Position, Range, SymbolKind,
    TextEdit, WorkspaceSymbol,
};
use std::path::{Path, PathBuf};

const MAX_FILES: usize = 5000;

fn markdown_files(roots: &[PathBuf]) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut stack = roots.to_vec();
    while let Some(dir) = stack.pop() {
        if out.len() >= MAX_FILES {
            break;
        }
        let Ok(entries) = std::fs::read_dir(dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if !matches!(
                    path.file_name().and_then(|n| n.to_str()),
                    Some(
                        ".git"
                            | "node_modules"
                            | "target"
                            | "dist"
                            | "build"
                            | "out"
                            | ".next"
                            | ".venv"
                    )
                ) {
                    stack.push(path);
                }
            } else if links::is_markdown(&path) {
                out.push(path);
            }
            if out.len() >= MAX_FILES {
                break;
            }
        }
    }
    out
}

fn target_for(
    link: &markdown::Link,
    current: &Path,
    roots: &[PathBuf],
) -> Option<(PathBuf, Option<String>)> {
    let ws = Workspace::new(roots, current)?;
    match links::classify(&link.dest) {
        Dest::Fragment(f) => Some((current.to_path_buf(), Some(f))),
        Dest::Local {
            path,
            fragment,
            rooted,
        } => match ws.resolve(&path, rooted) {
            Resolution::Exists(p) => Some((p, fragment)),
            _ => None,
        },
        Dest::Ignored => None,
    }
}

fn link_at(text: &str, offset: usize) -> Option<markdown::Link> {
    markdown::parse(text)
        .links
        .into_iter()
        .find(|l| l.range.contains(&offset))
}

pub fn definition(text: &str, offset: usize, path: &Path, roots: &[PathBuf]) -> Option<Location> {
    let link = link_at(text, offset)?;
    let (target, fragment) = target_for(&link, path, roots)?;
    let uri = Url::from_file_path(&target).ok()?;
    let target_text = std::fs::read_to_string(&target).ok()?;
    let idx = LineIndex::new(&target_text);
    let range = if let Some(f) = fragment {
        let h = links::find_heading(&markdown::parse(&target_text).headings, &f)?.clone();
        idx.range(&target_text, h.range.start, h.range.end)
    } else {
        Range::new(Position::new(0, 0), Position::new(0, 0))
    };
    Some(Location { uri, range })
}

pub fn references(
    text: &str,
    offset: usize,
    path: &Path,
    roots: &[PathBuf],
    include_declaration: bool,
) -> Option<Vec<Location>> {
    let link = link_at(text, offset)?;
    let (target, fragment) = target_for(&link, path, roots)?;
    let target = target.canonicalize().unwrap_or(target);
    let mut out = Vec::new();
    for file in markdown_files(roots) {
        let Ok(source) = std::fs::read_to_string(&file) else {
            continue;
        };
        let canonical = file.canonicalize().unwrap_or(file.clone());
        for candidate in markdown::parse(&source).links {
            let Some((dest, frag)) = target_for(&candidate, &file, roots) else {
                continue;
            };
            if dest.canonicalize().unwrap_or(dest) == target && frag == fragment {
                if let Ok(uri) = Url::from_file_path(&file) {
                    out.push(Location {
                        uri,
                        range: LineIndex::new(&source).range(
                            &source,
                            candidate.range.start,
                            candidate.range.end,
                        ),
                    });
                }
            }
        }
        if include_declaration && canonical == target {
            if let Some(f) = &fragment {
                if let Some(h) = links::find_heading(&markdown::parse(&source).headings, f) {
                    if let Ok(uri) = Url::from_file_path(&file) {
                        out.push(Location {
                            uri,
                            range: LineIndex::new(&source).range(
                                &source,
                                h.range.start,
                                h.range.end,
                            ),
                        });
                    }
                }
            }
        }
    }
    Some(out)
}

pub fn document_symbols(text: &str) -> Vec<DocumentSymbol> {
    let idx = LineIndex::new(text);
    markdown::parse(text)
        .headings
        .into_iter()
        .map(|h| {
            #[allow(deprecated)]
            DocumentSymbol {
                name: h.text,
                detail: Some(format!("#{}", h.slug)),
                kind: SymbolKind::STRING,
                tags: None,
                deprecated: None,
                range: idx.range(text, h.range.start, h.range.end),
                selection_range: idx.range(text, h.range.start, h.range.end),
                children: None,
            }
        })
        .collect()
}

pub fn workspace_symbols(roots: &[PathBuf], query: &str) -> Vec<WorkspaceSymbol> {
    let q = query.to_lowercase();
    let mut out = Vec::new();
    for path in markdown_files(roots) {
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        let Ok(uri) = Url::from_file_path(&path) else {
            continue;
        };
        let idx = LineIndex::new(&text);
        for h in markdown::parse(&text)
            .headings
            .into_iter()
            .filter(|h| h.text.to_lowercase().contains(&q) || h.slug.contains(&q))
        {
            #[allow(deprecated)]
            out.push(WorkspaceSymbol {
                name: h.text.clone(),
                kind: SymbolKind::STRING,
                tags: None,
                container_name: path.file_name().and_then(|x| x.to_str()).map(str::to_owned),
                location: lsp_types::OneOf::Left(Location {
                    uri: uri.clone(),
                    range: idx.range(&text, h.range.start, h.range.end),
                }),
                data: None,
            });
            if out.len() >= 500 {
                return out;
            }
        }
    }
    out
}

pub fn code_actions(text: &str, path: &Path, diagnostics: &[Diagnostic]) -> Vec<CodeAction> {
    let parsed = markdown::parse(text);
    let mut out = Vec::new();
    for d in diagnostics {
        if d.code.as_ref().and_then(|c| match c {
            lsp_types::NumberOrString::String(s) => Some(s.as_str()),
            _ => None,
        }) != Some("missing-alt")
        {
            continue;
        }
        let Some(link) = parsed.links.iter().find(|l| {
            l.is_image
                && l.alt.trim().is_empty()
                && LineIndex::new(text).range(text, l.range.start, l.range.end) == d.range
        }) else {
            continue;
        };
        let Some(start) = text[link.range.clone()]
            .find("![]")
            .map(|i| link.range.start + i + 2)
        else {
            continue;
        };
        let edit = TextEdit {
            range: LineIndex::new(text).range(text, start, start),
            new_text: "image description".into(),
        };
        out.push(CodeAction {
            title: "Add image alt text".into(),
            kind: Some(CodeActionKind::QUICKFIX),
            diagnostics: Some(vec![d.clone()]),
            edit: Some(lsp_types::WorkspaceEdit {
                changes: Some(
                    [(Url::from_file_path(path).unwrap(), vec![edit])]
                        .into_iter()
                        .collect(),
                ),
                ..Default::default()
            }),
            ..Default::default()
        });
    }
    out
}
