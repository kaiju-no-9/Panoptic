use panoptic_lsp::hover::hover;
use panoptic_lsp::links::{classify, Dest, Resolution, Workspace};
use std::path::PathBuf;

fn ws_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/workspace")
}

#[test]
fn classification() {
    assert_eq!(classify("https://x.y"), Dest::Ignored);
    assert_eq!(classify("mailto:a@b.c"), Dest::Ignored);
    assert_eq!(classify("//cdn.x/y"), Dest::Ignored);
    assert_eq!(classify("#frag"), Dest::Fragment("frag".into()));
    assert_eq!(
        classify("./a%20b.md?q=1#Top"),
        Dest::Local {
            path: "./a b.md".into(),
            fragment: Some("Top".into()),
            rooted: false
        }
    );
    assert_eq!(
        classify("/docs/x.md"),
        Dest::Local {
            path: "/docs/x.md".into(),
            fragment: None,
            rooted: true
        }
    );
}

#[test]
fn resolution() {
    let root = ws_dir();
    let ws = Workspace::new(&[root.clone()], &root.join("docs/a.md")).unwrap();
    assert!(matches!(
        ws.resolve("architecture.md", false),
        Resolution::Exists(_)
    ));
    assert!(matches!(
        ws.resolve("../guide.md", false),
        Resolution::Exists(_)
    ));
    assert!(matches!(
        ws.resolve("/guide.md", true),
        Resolution::Exists(_)
    ));
    assert!(matches!(
        ws.resolve("nope.md", false),
        Resolution::Missing(_)
    ));
    assert_eq!(ws.resolve("../../x.md", false), Resolution::Outside);
}

#[test]
fn hover_local_link() {
    let root = ws_dir();
    let text = "[a](./docs/architecture.md#overview)";
    let h = hover(text, 2, &root.join("d.md"), &[root.clone()]).unwrap();
    let lsp_types::HoverContents::Markup(m) = h.contents else {
        panic!()
    };
    assert!(m.value.contains("Target exists") && m.value.contains("Heading: **Overview**"));
}

#[test]
fn hover_remote_is_text_only_and_missing_reported() {
    let root = ws_dir();
    let h = hover(
        "[a](https://example.com)",
        1,
        &root.join("d.md"),
        &[root.clone()],
    )
    .unwrap();
    let lsp_types::HoverContents::Markup(m) = h.contents else {
        panic!()
    };
    assert_eq!(m.value, "`https://example.com`");
    let h = hover("[a](./zzz.md)", 1, &root.join("d.md"), &[root.clone()]).unwrap();
    let lsp_types::HoverContents::Markup(m) = h.contents else {
        panic!()
    };
    assert!(m.value.contains("does not exist"));
    assert!(hover("no links", 2, &root.join("d.md"), &[root]).is_none());
}
