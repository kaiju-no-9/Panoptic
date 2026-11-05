use pulldown_cmark::{CowStr, Event, LinkType, Options, Parser, Tag, TagEnd};
use std::collections::HashMap;
use std::ops::Range;

/// A link or image found in a document (never inside code).
#[derive(Debug, Clone)]
pub struct Link {
    pub is_image: bool,
    /// Destination as resolved by the parser (escapes processed, not percent-decoded).
    pub dest: String,
    /// Byte range of the raw destination text, for inline links only.
    pub dest_range: Option<Range<usize>>,
    /// Byte range of the whole link/image.
    pub range: Range<usize>,
    pub alt: String,
}

#[derive(Debug, Clone)]
pub struct Heading {
    pub text: String,
    pub slug: String,
}

#[derive(Debug, Default)]
pub struct Parsed {
    pub links: Vec<Link>,
    pub headings: Vec<Heading>,
}

struct Open {
    is_image: bool,
    dest: String,
    inline: bool,
    alt: String,
}

pub fn parse(text: &str) -> Parsed {
    let opts = Options::ENABLE_TABLES | Options::ENABLE_STRIKETHROUGH | Options::ENABLE_TASKLISTS;
    let mut out = Parsed::default();
    let mut open: Vec<Open> = Vec::new();
    let mut heading: Option<String> = None;
    let mut seen: HashMap<String, usize> = HashMap::new();

    for (event, range) in Parser::new_ext(text, opts).into_offset_iter() {
        match event {
            Event::Start(Tag::Link {
                link_type,
                dest_url,
                ..
            }) => {
                open.push(open_item(false, link_type, &dest_url));
            }
            Event::Start(Tag::Image {
                link_type,
                dest_url,
                ..
            }) => {
                open.push(open_item(true, link_type, &dest_url));
            }
            Event::End(TagEnd::Link) | Event::End(TagEnd::Image) => {
                if let Some(o) = open.pop() {
                    if o.dest.is_empty() && !o.is_image {
                        continue;
                    }
                    let dest_range = if o.inline {
                        inline_dest_range(text, range.clone())
                    } else {
                        None
                    };
                    out.links.push(Link {
                        is_image: o.is_image,
                        dest: o.dest,
                        dest_range,
                        range,
                        alt: o.alt,
                    });
                }
            }
            Event::Start(Tag::Heading { .. }) => heading = Some(String::new()),
            Event::End(TagEnd::Heading(_)) => {
                if let Some(h) = heading.take() {
                    let h = h.trim().to_string();
                    let base = slugify(&h);
                    if !base.is_empty() {
                        let n = seen.entry(base.clone()).or_insert(0);
                        let slug = if *n == 0 {
                            base.clone()
                        } else {
                            format!("{base}-{n}")
                        };
                        *n += 1;
                        out.headings.push(Heading { text: h, slug });
                    }
                }
            }
            Event::Text(t) | Event::Code(t) => push_text(&mut open, &mut heading, &t),
            Event::SoftBreak | Event::HardBreak => push_text(&mut open, &mut heading, " "),
            _ => {}
        }
    }
    out
}

fn push_text(open: &mut [Open], heading: &mut Option<String>, t: &str) {
    for o in open.iter_mut().filter(|o| o.is_image) {
        o.alt.push_str(t);
    }
    if let Some(h) = heading {
        h.push_str(t);
    }
}

fn open_item(is_image: bool, link_type: LinkType, dest: &CowStr) -> Open {
    // Autolinks (<https://...>) and emails are always external.
    let skip = matches!(link_type, LinkType::Autolink | LinkType::Email);
    Open {
        is_image,
        dest: if skip {
            String::new()
        } else {
            dest.to_string()
        },
        inline: link_type == LinkType::Inline,
        alt: String::new(),
    }
}

/// Locate the raw destination inside `[text](dest "title")`.
fn inline_dest_range(text: &str, range: Range<usize>) -> Option<Range<usize>> {
    let slice = text.get(range.clone())?;
    let open = slice.rfind("](")? + 2;
    let rest = &slice[open..];
    let lead = rest.len() - rest.trim_start().len();
    let rest_trim = &rest[lead..];
    let start = range.start + open + lead;
    if let Some(inner) = rest_trim.strip_prefix('<') {
        let end = inner.find('>')?;
        return Some(start + 1..start + 1 + end);
    }
    let end = rest_trim
        .find(char::is_whitespace)
        .unwrap_or_else(|| rest_trim.len().saturating_sub(1));
    Some(start..start + end)
}

/// GitHub-style heading slug: lowercase, spaces to hyphens, drop punctuation
/// (keeping letters, digits, `_` and `-`, including Unicode letters).
pub fn slugify(text: &str) -> String {
    let mut s = String::new();
    for c in text.trim().chars() {
        if c.is_alphanumeric() || c == '_' || c == '-' {
            s.extend(c.to_lowercase());
        } else if c.is_whitespace() {
            s.push('-');
        }
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slugs() {
        assert_eq!(slugify("Hello, World!"), "hello-world");
        assert_eq!(slugify("  Über Café  "), "über-café");
        assert_eq!(slugify("snake_case & more"), "snake_case--more");
        assert_eq!(slugify("!!!"), "");
    }

    #[test]
    fn duplicate_and_empty_headings() {
        let p = parse("# A\n\n## A\n\n### A\n\n#\n\n## !!!\n");
        let slugs: Vec<_> = p.headings.iter().map(|h| h.slug.as_str()).collect();
        assert_eq!(slugs, ["a", "a-1", "a-2"]);
    }

    #[test]
    fn dest_ranges() {
        let text = "x [a](./b%20c.md \"t\") <y> ![](i.png) [d](<e f.md>)";
        let p = parse(text);
        let dests: Vec<_> = p
            .links
            .iter()
            .map(|l| &text[l.dest_range.clone().unwrap()])
            .collect();
        assert_eq!(dests, ["./b%20c.md", "i.png", "e f.md"]);
        assert!(p.links[1].is_image && p.links[1].alt.is_empty());
    }

    #[test]
    fn ignores_code() {
        let p = parse("`[a](x.md)`\n\n```\n[b](y.md)\n```\n");
        assert!(p.links.is_empty());
    }
}
