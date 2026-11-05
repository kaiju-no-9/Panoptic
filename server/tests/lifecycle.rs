use lsp_server::{Connection, Message, Notification, Request, RequestId};
use lsp_types::{PublishDiagnosticsParams, Url};
use serde_json::{json, Value};
use std::path::PathBuf;

fn ws() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/workspace")
}

fn recv(c: &Connection) -> Message {
    c.receiver
        .recv_timeout(std::time::Duration::from_secs(5))
        .expect("server timed out")
}

fn notify(c: &Connection, method: &str, params: Value) {
    c.sender
        .send(Message::Notification(Notification {
            method: method.into(),
            params,
        }))
        .unwrap();
}

fn request(c: &Connection, id: i32, method: &str, params: Value) -> Value {
    c.sender
        .send(Message::Request(Request {
            id: RequestId::from(id),
            method: method.into(),
            params,
        }))
        .unwrap();
    loop {
        if let Message::Response(r) = recv(c) {
            assert!(r.error.is_none(), "{:?}", r.error);
            return r.result.unwrap_or(Value::Null);
        }
    }
}

fn diagnostics(c: &Connection) -> PublishDiagnosticsParams {
    match recv(c) {
        Message::Notification(n) if n.method == "textDocument/publishDiagnostics" => {
            serde_json::from_value(n.params).unwrap()
        }
        other => panic!("unexpected {other:?}"),
    }
}

#[test]
fn full_lifecycle() {
    let (server, client) = Connection::memory();
    let handle = std::thread::spawn(move || panoptic_lsp::server::run(server).unwrap());

    // Initialize handshake.
    let root = Url::from_file_path(ws()).unwrap();
    let init = request(
        &client,
        1,
        "initialize",
        json!({
            "processId": null, "capabilities": {}, "rootUri": root,
            "workspaceFolders": [{"uri": root, "name": "w"}]
        }),
    );
    let caps = &init["capabilities"];
    assert!(caps["completionProvider"].is_object() && caps["hoverProvider"] == true);
    for unsupported in [
        "documentFormattingProvider",
        "renameProvider",
        "codeActionProvider",
        "semanticTokensProvider",
        "workspaceSymbolProvider",
    ] {
        assert!(
            caps.get(unsupported).is_none(),
            "{unsupported} must not be advertised"
        );
    }
    notify(&client, "initialized", json!({}));

    // Open.
    let uri = Url::from_file_path(ws().join("live.md")).unwrap();
    notify(
        &client,
        "textDocument/didOpen",
        json!({"textDocument": {
            "uri": uri, "languageId": "markdown", "version": 1, "text": "[a](./nope.md)\n"
        }}),
    );
    let d = diagnostics(&client);
    assert_eq!((d.version, d.diagnostics.len()), (Some(1), 1));

    // Change: version carried through, diagnostics updated.
    notify(
        &client,
        "textDocument/didChange",
        json!({
            "textDocument": {"uri": uri, "version": 2},
            "contentChanges": [{"text": "[a](./guide.md) [b](./guide.md#"}]
        }),
    );
    let d = diagnostics(&client);
    assert_eq!(d.version, Some(2));

    // Completion and hover.
    let items = request(
        &client,
        2,
        "textDocument/completion",
        json!({
            "textDocument": {"uri": uri}, "position": {"line": 0, "character": 31}
        }),
    );
    let labels: Vec<_> = items
        .as_array()
        .unwrap()
        .iter()
        .map(|i| i["label"].as_str().unwrap().to_string())
        .collect();
    assert!(labels.contains(&"install".to_string()), "{labels:?}");
    let hv = request(
        &client,
        3,
        "textDocument/hover",
        json!({
            "textDocument": {"uri": uri}, "position": {"line": 0, "character": 2}
        }),
    );
    assert_eq!(hv["contents"]["kind"], "markdown");

    // Close clears diagnostics and state.
    notify(
        &client,
        "textDocument/didClose",
        json!({"textDocument": {"uri": uri}}),
    );
    let d = diagnostics(&client);
    assert!(d.diagnostics.is_empty());
    let none = request(
        &client,
        4,
        "textDocument/hover",
        json!({
            "textDocument": {"uri": uri}, "position": {"line": 0, "character": 2}
        }),
    );
    assert!(none.is_null());

    // Shutdown / exit.
    request(&client, 5, "shutdown", Value::Null);
    notify(&client, "exit", Value::Null);
    handle.join().unwrap();
}
