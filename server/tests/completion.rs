use lsp_types::CompletionTextEdit;
use panoptic_lsp::completion::complete;
use panoptic_lsp::document::LineIndex;
use std::path::PathBuf;

fn ws() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/workspace")
}

/// Returns (label, new_text, start_col, end_col) per item; `|` marks the cursor.
fn run(src: &str) -> Vec<(String, String, u32, u32)> {
    let off = src.find('|').unwrap();
    let text = src.replacen('|', "", 1);
    let root = ws();
    let _ = LineIndex::new(&text);
    complete(&text, off, &root.join("doc.md"), &[root.clone()])
        .into_iter()
        .map(|i| {
            let Some(CompletionTextEdit::Edit(e)) = i.text_edit else {
                panic!()
            };
            (
                i.label,
                e.new_text,
                e.range.start.character,
                e.range.end.character,
            )
        })
        .collect()
}

#[test]
fn lists_current_dir_without_noise() {
    let r = run("[a](|)");
    let labels: Vec<_> = r.iter().map(|x| x.0.as_str()).collect();
    assert!(labels.contains(&"guide.md") && labels.contains(&"docs/"));
    assert!(!labels.iter().any(|l| l.starts_with('.')));
}

#[test]
fn preserves_prefix_and_filters() {
    let r = run("[a](./docs/ar|)");
    assert_eq!(r.len(), 1);
    assert_eq!(r[0].1, "architecture.md");
    // Replaces only the typed name "ar" (cols 11..13).
    assert_eq!((r[0].2, r[0].3), (11, 13));
}

#[test]
fn heading_completion_other_file() {
    let r = run("[a](./docs/architecture.md#|)");
    let slugs: Vec<_> = r.iter().map(|x| x.1.as_str()).collect();
    assert_eq!(slugs, ["architecture", "overview", "components"]);
}

#[test]
fn heading_completion_same_file() {
    let r = run("# Alpha\n\n[a](#|)");
    assert_eq!(r.len(), 1);
    assert_eq!(r[0].1, "alpha");
}

#[test]
fn no_suggestions_outside_link_destination() {
    assert!(run("plain text|").is_empty());
    assert!(run("[a|](x)").is_empty());
    assert!(run("[a](https://exa|)").is_empty());
    assert!(run("```\n[a](|)\n```").is_empty());
    assert!(run("[a](x |)").is_empty());
}
