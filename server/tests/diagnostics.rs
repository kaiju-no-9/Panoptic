use panoptic_lsp::diagnostics::compute;
use std::path::PathBuf;

fn ws() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/workspace")
}

fn messages(text: &str) -> Vec<String> {
    let root = ws();
    compute(text, &root.join("doc.md"), &[root.clone()])
        .into_iter()
        .map(|d| d.message)
        .collect()
}

#[test]
fn fixture_file_reports_expected_problems() {
    let text = std::fs::read_to_string(ws().join("broken-links.md")).unwrap();
    let m = messages(&text);
    assert_eq!(m.len(), 4, "{m:?}");
    assert!(m
        .iter()
        .any(|s| s.contains("./nope.md") || s.contains("nope.md")));
    assert!(m.iter().any(|s| s.contains("#nothing")));
    assert!(m.iter().any(|s| s.contains("alt text")));
    assert!(m.iter().any(|s| s.contains("gone.png")));
}

#[test]
fn valid_links_produce_nothing() {
    let text = "[a](./guide.md) [b](docs/architecture.md#overview) [c](./guide.md#install-1)\n\
                ![Alt](images/existing.png) [d](./guide.md?x=1#usage--tips) [e](#top)\n\n# Top\n";
    assert!(messages(text).is_empty(), "{:?}", messages(text));
}

#[test]
fn remote_mailto_and_code_ignored() {
    let text = "[x](https://example.com/a.md) [m](mailto:a@b.c) <https://x.y/z>\n\
                `[c](nope.md)`\n\n```\n[d](nope.md)\n![](nope.png)\n```\n";
    assert!(messages(text).is_empty());
}

#[test]
fn url_encoded_paths_are_decoded() {
    assert!(messages("[a](./docs/../guide%2Emd)").is_empty());
    assert_eq!(messages("[a](./my%20file.md)").len(), 1);
}

#[test]
fn outside_workspace_not_reported() {
    assert!(messages("[a](../../../../etc/passwd) ![x](../../outside.png)").is_empty());
}

#[test]
fn same_doc_anchor() {
    assert_eq!(messages("# One\n\n[a](#one) [b](#two)").len(), 1);
}

#[test]
fn ranges_use_utf16_columns_and_dest() {
    let root = ws();
    let d = compute("😀 [a](./nope.md)", &root.join("d.md"), &[root.clone()]);
    assert_eq!(d.len(), 1);
    // "😀 [a](" is 2 + 1 + 4 = 7 UTF-16 units.
    assert_eq!(d[0].range.start.character, 7);
    assert_eq!(d[0].range.end.character, 16);
}
