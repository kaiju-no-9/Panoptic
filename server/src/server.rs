use crate::document::{LineIndex, OpenDocument};
use crate::{completion, diagnostics, hover};
use lsp_server::{Connection, Message, Notification, Request, RequestId, Response};
use lsp_types::notification::{
    DidChangeTextDocument, DidCloseTextDocument, DidOpenTextDocument, Notification as _,
    PublishDiagnostics,
};
use lsp_types::{
    CompletionOptions, CompletionParams, CompletionResponse, DidChangeTextDocumentParams,
    DidCloseTextDocumentParams, DidOpenTextDocumentParams, HoverParams, HoverProviderCapability,
    InitializeParams, PublishDiagnosticsParams, ServerCapabilities, TextDocumentSyncCapability,
    TextDocumentSyncKind, Url,
};
use std::collections::HashMap;
use std::error::Error;
use std::path::PathBuf;

type Res<T> = Result<T, Box<dyn Error + Sync + Send>>;

pub fn capabilities() -> ServerCapabilities {
    ServerCapabilities {
        text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL)),
        completion_provider: Some(CompletionOptions {
            trigger_characters: Some(vec!["(".into(), "/".into(), "#".into()]),
            ..Default::default()
        }),
        hover_provider: Some(HoverProviderCapability::Simple(true)),
        ..Default::default()
    }
}

struct State {
    docs: HashMap<Url, OpenDocument>,
    roots: Vec<PathBuf>,
}

pub fn run(connection: Connection) -> Res<()> {
    let caps = serde_json::to_value(capabilities())?;
    let init = connection.initialize(caps)?;
    let params: InitializeParams = serde_json::from_value(init)?;

    #[allow(deprecated)]
    let mut roots: Vec<PathBuf> = params
        .workspace_folders
        .iter()
        .flatten()
        .filter_map(|f| f.uri.to_file_path().ok())
        .collect();
    #[allow(deprecated)]
    if roots.is_empty() {
        roots.extend(params.root_uri.and_then(|u| u.to_file_path().ok()));
    }

    let mut state = State {
        docs: HashMap::new(),
        roots,
    };

    for msg in &connection.receiver {
        match msg {
            Message::Request(req) => {
                if connection.handle_shutdown(&req)? {
                    return Ok(());
                }
                let resp = handle_request(&state, req);
                connection.sender.send(Message::Response(resp))?;
            }
            Message::Notification(n) => handle_notification(&connection, &mut state, n)?,
            Message::Response(_) => {}
        }
    }
    Ok(())
}

fn handle_request(state: &State, req: Request) -> Response {
    let id = req.id.clone();
    let result = match req.method.as_str() {
        "textDocument/completion" => {
            serde_json::from_value::<CompletionParams>(req.params).map(|p| {
                let pos = p.text_document_position;
                let items = with_doc(
                    state,
                    &pos.text_document.uri,
                    pos.position,
                    |text, off, path| completion::complete(text, off, path, &state.roots),
                );
                serde_json::to_value(CompletionResponse::Array(items.unwrap_or_default()))
            })
        }
        "textDocument/hover" => serde_json::from_value::<HoverParams>(req.params).map(|p| {
            let pos = p.text_document_position_params;
            let h = with_doc(
                state,
                &pos.text_document.uri,
                pos.position,
                |text, off, path| hover::hover(text, off, path, &state.roots),
            );
            serde_json::to_value(h.flatten())
        }),
        m => return error(id, -32601, format!("method not found: {m}")),
    };
    match result {
        Ok(Ok(v)) => Response {
            id,
            result: Some(v),
            error: None,
        },
        Ok(Err(e)) | Err(e) => error(id, -32603, e.to_string()),
    }
}

fn error(id: RequestId, code: i32, message: String) -> Response {
    Response::new_err(id, code, message)
}

fn with_doc<T>(
    state: &State,
    uri: &Url,
    pos: lsp_types::Position,
    f: impl FnOnce(&str, usize, &std::path::Path) -> T,
) -> Option<T> {
    let doc = state.docs.get(uri)?;
    let path = uri.to_file_path().ok()?;
    let off = LineIndex::new(&doc.text).offset(&doc.text, pos);
    Some(f(&doc.text, off, &path))
}

fn handle_notification(conn: &Connection, state: &mut State, n: Notification) -> Res<()> {
    match n.method.as_str() {
        DidOpenTextDocument::METHOD => {
            let p: DidOpenTextDocumentParams = serde_json::from_value(n.params)?;
            let d = p.text_document;
            let uri = d.uri.clone();
            state.docs.insert(
                d.uri.clone(),
                OpenDocument {
                    uri: d.uri,
                    version: d.version,
                    text: d.text,
                },
            );
            publish(conn, state, &uri)?;
        }
        DidChangeTextDocument::METHOD => {
            let p: DidChangeTextDocumentParams = serde_json::from_value(n.params)?;
            let uri = p.text_document.uri;
            if let Some(doc) = state.docs.get_mut(&uri) {
                // Ignore stale updates; full sync means the last change is the whole text.
                if p.text_document.version >= doc.version {
                    if let Some(c) = p.content_changes.into_iter().last() {
                        doc.text = c.text;
                        doc.version = p.text_document.version;
                    }
                    publish(conn, state, &uri)?;
                }
            }
        }
        DidCloseTextDocument::METHOD => {
            let p: DidCloseTextDocumentParams = serde_json::from_value(n.params)?;
            let uri = p.text_document.uri;
            if state.docs.remove(&uri).is_some() {
                send_diagnostics(conn, uri, Vec::new(), None)?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn publish(conn: &Connection, state: &State, uri: &Url) -> Res<()> {
    let Some(doc) = state.docs.get(uri) else {
        return Ok(());
    };
    let diags = match uri.to_file_path() {
        Ok(path) => diagnostics::compute(&doc.text, &path, &state.roots),
        Err(_) => Vec::new(),
    };
    send_diagnostics(conn, uri.clone(), diags, Some(doc.version))
}

fn send_diagnostics(
    conn: &Connection,
    uri: Url,
    diagnostics: Vec<lsp_types::Diagnostic>,
    version: Option<i32>,
) -> Res<()> {
    let params = PublishDiagnosticsParams {
        uri,
        diagnostics,
        version,
    };
    conn.sender.send(Message::Notification(Notification::new(
        PublishDiagnostics::METHOD.to_string(),
        params,
    )))?;
    Ok(())
}
