use crate::document::LineIndex;
use crate::links::{self, Dest, Resolution, Workspace};
use crate::markdown;
use lsp_types::{Diagnostic, DiagnosticSeverity};
use std::path::{Path, PathBuf};

pub const SOURCE: &str = "panoptic";

/// Analyze a document. Pure: never modifies anything.
pub fn compute(text: &str, doc_path: &Path, roots: &[PathBuf]) -> Vec<Diagnostic> {
    let Some(ws) = Workspace::new(roots, doc_path) else {
        return Vec::new();
    };
    let index = LineIndex::new(text);
    let parsed = markdown::parse(text);
    let mut out = Vec::new();
    let mut warn = |start: usize, end: usize, code: &str, message: String| {
        out.push(Diagnostic {
            range: index.range(text, start, end),
            severity: Some(DiagnosticSeverity::WARNING),
            code: Some(lsp_types::NumberOrString::String(code.into())),
            source: Some(SOURCE.into()),
            message,
            ..Default::default()
        });
    };

    for link in &parsed.links {
        if link.is_image && link.alt.trim().is_empty() {
            warn(
                link.range.start,
                link.range.end,
                "missing-alt",
                "Image is missing alt text".into(),
            );
        }
        let r = link.dest_range.clone().unwrap_or(link.range.clone());
        match links::classify(&link.dest) {
            Dest::Ignored => {}
            Dest::Fragment(f) => {
                if !link.is_image && links::find_heading(&parsed.headings, &f).is_none() {
                    warn(
                        r.start,
                        r.end,
                        "broken-anchor",
                        format!("No heading matches '#{f}' in this document"),
                    );
                }
            }
            Dest::Local {
                path,
                fragment,
                rooted,
            } => match ws.resolve(&path, rooted) {
                Resolution::Outside => {}
                Resolution::Missing(_) => {
                    let (code, what) = if link.is_image {
                        ("broken-image", "Image file")
                    } else {
                        ("broken-link", "Link target")
                    };
                    warn(r.start, r.end, code, format!("{what} not found: {path}"));
                }
                Resolution::Exists(target) => {
                    if let (Some(f), false) = (fragment, link.is_image) {
                        if let Some(hs) = links::headings_of_file(&target) {
                            if links::find_heading(&hs, &f).is_none() {
                                warn(
                                    r.start,
                                    r.end,
                                    "broken-anchor",
                                    format!("No heading matches '#{f}' in {path}"),
                                );
                            }
                        }
                    }
                }
            },
        }
    }
    out
}
