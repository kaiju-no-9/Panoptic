use crate::markdown::{self, Heading};
use percent_encoding::percent_decode_str;
use std::path::{Component, Path, PathBuf};

const MAX_READ_BYTES: u64 = 2 * 1024 * 1024;

/// What a link destination refers to.
#[derive(Debug, PartialEq, Eq)]
pub enum Dest {
    /// Remote URL, `mailto:`, other scheme, empty, or otherwise not a local reference.
    Ignored,
    /// `#fragment` within the current document.
    Fragment(String),
    Local {
        /// Percent-decoded path without query or fragment.
        path: String,
        fragment: Option<String>,
        /// Starts with `/` (relative to the workspace root).
        rooted: bool,
    },
}

pub fn has_scheme(s: &str) -> bool {
    let end = s.find([':', '/', '?', '#']).unwrap_or(s.len());
    s[end..].starts_with(':')
        && end > 0
        && s.starts_with(|c: char| c.is_ascii_alphabetic())
        && s[..end]
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '-' | '.'))
}

pub fn decode(s: &str) -> String {
    percent_decode_str(s).decode_utf8_lossy().into_owned()
}

pub fn classify(raw: &str) -> Dest {
    let raw = raw.trim();
    if raw.is_empty() || has_scheme(raw) || raw.starts_with("//") {
        return Dest::Ignored;
    }
    let (before, fragment) = match raw.split_once('#') {
        Some((b, f)) => (b, Some(decode(f)).filter(|f| !f.is_empty())),
        None => (raw, None),
    };
    let path = before.split('?').next().unwrap_or("");
    if path.is_empty() {
        return match fragment {
            Some(f) => Dest::Fragment(f),
            None => Dest::Ignored,
        };
    }
    Dest::Local {
        path: decode(path),
        fragment,
        rooted: path.starts_with('/'),
    }
}

/// Directories that links may point into. Targets outside are never read.
pub struct Workspace {
    roots: Vec<PathBuf>,
    pub doc_dir: PathBuf,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Resolution {
    Exists(PathBuf),
    Missing(PathBuf),
    /// Outside the workspace boundary: neither checked nor reported.
    Outside,
}

impl Workspace {
    /// `roots` are the workspace folders; if empty, the document's directory is the boundary.
    pub fn new(roots: &[PathBuf], doc_path: &Path) -> Option<Self> {
        let doc_dir = canon(doc_path.parent()?);
        let mut roots: Vec<PathBuf> = roots.iter().map(|r| canon(r)).collect();
        if roots.is_empty() {
            roots.push(doc_dir.clone());
        }
        Some(Self { roots, doc_dir })
    }

    fn inside(&self, p: &Path) -> bool {
        self.roots.iter().any(|r| p.starts_with(r))
    }

    pub fn resolve(&self, path: &str, rooted: bool) -> Resolution {
        let base = if rooted {
            self.roots
                .iter()
                .find(|r| self.doc_dir.starts_with(r))
                .unwrap_or(&self.roots[0])
                .clone()
        } else {
            self.doc_dir.clone()
        };
        let candidate = normalize(&base.join(path.trim_start_matches('/')));
        match candidate.canonicalize() {
            Ok(real) if self.inside(&real) => Resolution::Exists(real),
            Ok(_) => Resolution::Outside,
            Err(_) if self.inside(&candidate) => Resolution::Missing(candidate),
            Err(_) => Resolution::Outside,
        }
    }
}

fn canon(p: &Path) -> PathBuf {
    p.canonicalize().unwrap_or_else(|_| normalize(p))
}

/// Lexically remove `.` and `..` components.
pub fn normalize(p: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for c in p.components() {
        match c {
            Component::CurDir => {}
            Component::ParentDir => {
                out.pop();
            }
            other => out.push(other.as_os_str()),
        }
    }
    out
}

pub fn is_markdown(p: &Path) -> bool {
    matches!(
        p.extension()
            .and_then(|e| e.to_str())
            .map(str::to_ascii_lowercase)
            .as_deref(),
        Some("md" | "markdown")
    )
}

/// Headings of a Markdown file on disk (bounded size), or `None` if unreadable.
pub fn headings_of_file(p: &Path) -> Option<Vec<Heading>> {
    if !is_markdown(p) || std::fs::metadata(p).ok()?.len() > MAX_READ_BYTES {
        return None;
    }
    let text = std::fs::read_to_string(p).ok()?;
    Some(markdown::parse(&text).headings)
}

pub fn find_heading<'a>(headings: &'a [Heading], fragment: &str) -> Option<&'a Heading> {
    let f = fragment.to_lowercase();
    headings.iter().find(|h| h.slug == f)
}
