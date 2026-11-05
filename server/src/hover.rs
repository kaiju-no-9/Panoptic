use crate::document::LineIndex;
use crate::links::{self, Dest, Resolution, Workspace};
use crate::markdown;
use lsp_types::{Hover, HoverContents, MarkupContent, MarkupKind};
use std::path::{Path, PathBuf};

pub fn hover(text: &str, offset: usize, doc_path: &Path, roots: &[PathBuf]) -> Option<Hover> {
    let parsed = markdown::parse(text);
    let link = parsed
        .links
        .iter()
        .filter(|l| l.range.contains(&offset))
        .min_by_key(|l| l.range.len())?;

    let body = match links::classify(&link.dest) {
        Dest::Ignored => {
            if link.dest.trim().is_empty() {
                return None;
            }
            format!("`{}`", link.dest.replace('`', "'"))
        }
        Dest::Fragment(f) => match links::find_heading(&parsed.headings, &f) {
            Some(h) => format!("Heading in this document: **{}** (`#{}`)", h.text, h.slug),
            None => format!("No heading matches `#{f}` in this document"),
        },
        Dest::Local {
            path,
            fragment,
            rooted,
        } => {
            let ws = Workspace::new(roots, doc_path)?;
            match ws.resolve(&path, rooted) {
                Resolution::Outside => return None,
                Resolution::Missing(p) => format!("`{}`\n\nTarget does not exist", p.display()),
                Resolution::Exists(p) => {
                    let mut s = format!("`{}`\n\nTarget exists", p.display());
                    if let Some(f) = fragment {
                        s.push_str(&format!("\n\nFragment: `#{f}`"));
                        match links::headings_of_file(&p) {
                            Some(hs) => match links::find_heading(&hs, &f) {
                                Some(h) => s.push_str(&format!("\n\nHeading: **{}**", h.text)),
                                None => s.push_str("\n\nNo matching heading found"),
                            },
                            None => {}
                        }
                    }
                    s
                }
            }
        }
    };

    let index = LineIndex::new(text);
    Some(Hover {
        contents: HoverContents::Markup(MarkupContent {
            kind: MarkupKind::Markdown,
            value: body,
        }),
        range: Some(index.range(text, link.range.start, link.range.end)),
    })
}
