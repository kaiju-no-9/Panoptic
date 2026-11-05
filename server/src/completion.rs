use crate::document::LineIndex;
use crate::links::{self, Resolution, Workspace};
use crate::markdown;
use lsp_types::{CompletionItem, CompletionItemKind, CompletionTextEdit, TextEdit};
use percent_encoding::{utf8_percent_encode, AsciiSet, CONTROLS};
use std::path::{Path, PathBuf};

const ENCODE: &AsciiSet = &CONTROLS
    .add(b' ')
    .add(b'"')
    .add(b'<')
    .add(b'>')
    .add(b'`')
    .add(b'#')
    .add(b'?')
    .add(b'(')
    .add(b')')
    .add(b'[')
    .add(b']')
    .add(b'%')
    .add(b'\\');

const EXCLUDED: &[&str] = &[
    ".git",
    "node_modules",
    "target",
    "dist",
    "build",
    "out",
    ".next",
    ".venv",
    "__pycache__",
];
const MAX_SCAN: usize = 5000;
const MAX_ITEMS: usize = 200;

/// Byte offset just after `](` if `offset` is inside a link destination being typed.
fn dest_start(text: &str, offset: usize) -> Option<usize> {
    let line_start = text[..offset].rfind('\n').map_or(0, |i| i + 1);
    let line = &text[line_start..offset];
    let idx = line.rfind("](")?;
    if !line[..idx].contains('[') {
        return None;
    }
    let start = line_start + idx + 2;
    let seg = &text[start..offset];
    if seg.contains(|c: char| c.is_whitespace() || c == ')' || c == '<') || links::has_scheme(seg) {
        return None;
    }
    // Inside a fenced code block? Count fence lines before this one.
    let fences = text[..line_start]
        .lines()
        .filter(|l| {
            let t = l.trim_start();
            t.starts_with("```") || t.starts_with("~~~")
        })
        .count();
    (fences % 2 == 0).then_some(start)
}

pub fn complete(
    text: &str,
    offset: usize,
    doc_path: &Path,
    roots: &[PathBuf],
) -> Vec<CompletionItem> {
    let Some(start) = dest_start(text, offset) else {
        return Vec::new();
    };
    let Some(ws) = Workspace::new(roots, doc_path) else {
        return Vec::new();
    };
    let index = LineIndex::new(text);
    let seg = &text[start..offset];

    if let Some(h) = seg.find('#') {
        let path_part = &seg[..h];
        let frag_start = start + h + 1;
        let headings = if path_part.is_empty() {
            markdown::parse(text).headings
        } else {
            match path_part
                .split('?')
                .next()
                .map(|p| ws.resolve(&links::decode(p), path_part.starts_with('/')))
            {
                Some(Resolution::Exists(p)) => links::headings_of_file(&p).unwrap_or_default(),
                _ => Vec::new(),
            }
        };
        let range = index.range(text, frag_start, offset);
        return headings
            .into_iter()
            .map(|h| CompletionItem {
                label: h.slug.clone(),
                detail: Some(h.text),
                kind: Some(CompletionItemKind::REFERENCE),
                text_edit: Some(CompletionTextEdit::Edit(TextEdit::new(range, h.slug))),
                ..Default::default()
            })
            .collect();
    }

    let name_off = seg.rfind('/').map_or(0, |i| i + 1);
    let (dir_part, typed) = seg.split_at(name_off);
    if dir_part.contains('?') || typed.contains('?') {
        return Vec::new();
    }
    let dir_decoded = links::decode(dir_part);
    let dir = match ws.resolve(
        if dir_decoded.is_empty() {
            "."
        } else {
            &dir_decoded
        },
        dir_part.starts_with('/'),
    ) {
        Resolution::Exists(p) if p.is_dir() => p,
        _ => return Vec::new(),
    };
    let typed_lc = links::decode(typed).to_lowercase();
    let range = index.range(text, start + name_off, offset);
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return Vec::new();
    };

    let mut items = Vec::new();
    for entry in entries.take(MAX_SCAN).flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        if EXCLUDED.contains(&name.as_str())
            || (name.starts_with('.') && !typed_lc.starts_with('.'))
            || !name.to_lowercase().starts_with(&typed_lc)
        {
            continue;
        }
        let is_dir = entry.path().is_dir();
        let mut insert = utf8_percent_encode(&name, ENCODE).to_string();
        let mut label = name.clone();
        if is_dir {
            insert.push('/');
            label.push('/');
        }
        items.push(CompletionItem {
            sort_text: Some(format!(
                "{}{}",
                if is_dir { 0 } else { 1 },
                name.to_lowercase()
            )),
            label,
            kind: Some(if is_dir {
                CompletionItemKind::FOLDER
            } else {
                CompletionItemKind::FILE
            }),
            text_edit: Some(CompletionTextEdit::Edit(TextEdit::new(range, insert))),
            ..Default::default()
        });
        if items.len() >= MAX_ITEMS {
            break;
        }
    }
    items
}
