//! The tools a worker calls: JSON-RPC over HTTP on loopback, reached
//! through a pinfold route, authenticated by a per-execution bearer.

use crate::api::Fail;
use crate::daemon::{Daemon, Project};
use http_body_util::{BodyExt, Full, Limited};
use hyper::body::{Bytes, Incoming};
use hyper::{Request, Response, StatusCode};
use serde_json::{Value, json};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

const BODY_MAX: usize = 1024 * 1024;
pub const NOTE_MAX: usize = 4000;

/// One bearer's grant.
#[derive(Clone)]
pub struct Grant {
    pub project: Arc<Project>,
    pub execution: i64,
    pub kind: Kind,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Implementation,
    Review,
}

#[derive(Default)]
pub struct Grants {
    bearers: Mutex<HashMap<String, Grant>>,
    /// `(project key, execution)` whose grant a harness has listed tools for.
    /// A required MCP server's connection is the registration proof, and this
    /// is the daemon's own positive signal that the harness reached it.
    connected: Mutex<HashSet<(String, i64)>>,
    /// The one MCP session each bearer may use, opened by its first
    /// `initialize`. Every process in a box inherits the bearer, so the
    /// session id, held only in the harness's memory, is what authorises.
    sessions: Mutex<HashMap<String, String>>,
}

impl Grants {
    pub fn issue(&self, grant: Grant) -> String {
        let mut bytes = [0u8; 32];
        getrandom::fill(&mut bytes).expect("the OS gives randomness");
        let bearer = format!("yard_{}", crate::config::hex(&bytes));
        self.bearers
            .lock()
            .expect("grants lock")
            .insert(bearer.clone(), grant);
        bearer
    }

    /// Execution ids are per project, so a grant is named by both.
    pub fn revoke(&self, project: &Project, execution: i64) {
        self.bearers
            .lock()
            .expect("grants lock")
            .retain(|_, grant| grant.project.key != project.key || grant.execution != execution);
        let bearers = self.bearers.lock().expect("grants lock");
        self.sessions
            .lock()
            .expect("grants lock")
            .retain(|bearer, _| bearers.contains_key(bearer));
        self.connected
            .lock()
            .expect("grants lock")
            .remove(&(project.key.clone(), execution));
    }

    /// Whether the harness fetched this execution's tools from the server.
    pub fn connected(&self, project: &Project, execution: i64) -> bool {
        self.connected
            .lock()
            .expect("grants lock")
            .contains(&(project.key.clone(), execution))
    }

    /// The harness reached the server. Called in the same task that answers
    /// `initialize`, before the harness's first frame, so a harness that
    /// connected has already left the mark.
    fn mark_connected(&self, project: &Project, execution: i64) {
        self.connected
            .lock()
            .expect("grants lock")
            .insert((project.key.clone(), execution));
    }

    /// Opens the bearer's session, once. `None` if it already has one.
    fn open_session(&self, bearer: &str) -> Option<String> {
        let mut sessions = self.sessions.lock().expect("grants lock");
        if sessions.contains_key(bearer) {
            return None;
        }
        let mut bytes = [0u8; 16];
        getrandom::fill(&mut bytes).expect("the OS gives randomness");
        let id = crate::config::hex(&bytes);
        sessions.insert(bearer.to_string(), id.clone());
        Some(id)
    }

    fn in_session(&self, bearer: &str, id: Option<&str>) -> bool {
        id.is_some_and(|id| {
            self.sessions
                .lock()
                .expect("grants lock")
                .get(bearer)
                .is_some_and(|bound| bound == id)
        })
    }

    fn get(&self, bearer: &str) -> Option<Grant> {
        self.bearers
            .lock()
            .expect("grants lock")
            .get(bearer)
            .cloned()
    }
}

/// The names of the tools a kind is granted.
pub fn tool_names(kind: Kind) -> Vec<String> {
    tools(kind)
        .iter()
        .filter_map(|tool| tool["name"].as_str().map(str::to_string))
        .collect()
}

fn tools(kind: Kind) -> Vec<Value> {
    let context = json!({
        "name": "yard_context",
        "description": "Your ticket, the brief, and the candidate's base and head.",
        "inputSchema": { "type": "object", "properties": {}, "additionalProperties": false },
    });
    let progress = json!({
        "name": "yard_progress",
        "description": "Record one short progress note for the operator and for a later fresh session.",
        "inputSchema": {
            "type": "object",
            "properties": { "note": { "type": "string", "maxLength": NOTE_MAX } },
            "required": ["note"],
            "additionalProperties": false,
        },
    });
    let propose = json!({
        "name": "yard_propose",
        "description": "Propose a new ticket, an edit to your own ticket, or a dependency link. The operator decides. `key` names a proposed ticket so a later proposal of yours can depend on it.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "kind": { "type": "string", "enum": ["ticket", "edit", "link"] },
                "key": { "type": "string", "maxLength": 64 },
                "title": { "type": "string", "maxLength": 200 },
                "body": { "type": "string", "maxLength": 65536 },
                "priority": { "type": "string", "enum": ["P0", "P1", "P2", "P3"] },
                "workflow": { "type": "string", "maxLength": 64 },
                "parked": { "type": "boolean" },
                "depends_on": { "type": "array", "items": { "type": "string", "maxLength": 64 }, "maxItems": 32 },
                "ticket": { "type": "string", "maxLength": 64 },
                "reason": { "type": "string", "maxLength": 4000 },
            },
            "required": ["kind"],
            "additionalProperties": false,
        },
    });
    let publish = json!({
        "name": "yard_publish_review",
        "description": "Publish your review once. Every finding has a priority P0 (worst) to P3.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "findings": {
                    "type": "array",
                    "maxItems": 200,
                    "items": {
                        "type": "object",
                        "properties": {
                            "priority": { "type": "string", "enum": ["P0", "P1", "P2", "P3"] },
                            "file": { "type": "string", "maxLength": 1024 },
                            "line": { "type": "integer", "minimum": 1 },
                            "category": { "type": "string", "maxLength": 64 },
                            "body": { "type": "string", "maxLength": 8000 },
                        },
                        "required": ["priority", "body"],
                        "additionalProperties": false,
                    },
                },
            },
            "required": ["findings"],
            "additionalProperties": false,
        },
    });
    match kind {
        Kind::Implementation => vec![context, progress, propose],
        Kind::Review => vec![context, progress, publish],
    }
}

pub async fn serve(daemon: Arc<Daemon>, listener: tokio::net::TcpListener) {
    loop {
        let Ok((stream, _)) = listener.accept().await else {
            continue;
        };
        let daemon = daemon.clone();
        tokio::spawn(async move {
            let service = hyper::service::service_fn(move |request| {
                let daemon = daemon.clone();
                async move { Ok::<_, hyper::Error>(answer(&daemon, request).await) }
            });
            let _ = hyper::server::conn::http1::Builder::new()
                .serve_connection(hyper_util::rt::TokioIo::new(stream), service)
                .await;
        });
    }
}

fn reply(status: StatusCode, body: Option<Value>) -> Response<Full<Bytes>> {
    Response::builder()
        .status(status)
        .header("content-type", "application/json")
        .body(Full::new(Bytes::from(
            body.map(|value| value.to_string()).unwrap_or_default(),
        )))
        .expect("response builds")
}

async fn answer(daemon: &Arc<Daemon>, request: Request<Incoming>) -> Response<Full<Bytes>> {
    if request.uri().path() != "/mcp" || request.method() != hyper::Method::POST {
        return reply(StatusCode::NOT_FOUND, None);
    }
    let bearer = request
        .headers()
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .map(str::to_string);
    let Some((bearer, grant)) = bearer.and_then(|b| daemon.grants.get(&b).map(|g| (b, g))) else {
        return reply(StatusCode::UNAUTHORIZED, None);
    };
    let session = request
        .headers()
        .get("mcp-session-id")
        .and_then(|value| value.to_str().ok())
        .map(str::to_string);
    let Ok(body) = Limited::new(request.into_body(), BODY_MAX).collect().await else {
        return reply(StatusCode::PAYLOAD_TOO_LARGE, None);
    };
    let message = serde_json::from_slice::<Value>(&body.to_bytes());
    let opens = message
        .as_ref()
        .is_ok_and(|m| m["method"] == "initialize" && m.get("id").is_some());
    let mut opened = None;
    if opens {
        opened = daemon.grants.open_session(&bearer);
        if opened.is_none() {
            return reply(StatusCode::NOT_FOUND, None);
        }
    } else if !daemon.grants.in_session(&bearer, session.as_deref()) {
        return reply(StatusCode::NOT_FOUND, None);
    }
    let Ok(message) = message else {
        return reply(
            StatusCode::OK,
            Some(json!({ "jsonrpc": "2.0", "id": null,
                         "error": { "code": -32700, "message": "not JSON" } })),
        );
    };
    let id = message.get("id").cloned();
    let method = message["method"].as_str().unwrap_or_default();
    let result = match (method, &id) {
        ("notifications/initialized", None) => return reply(StatusCode::ACCEPTED, None),
        ("initialize", Some(_)) => {
            // The handshake is the connection; a harness reaches it before
            // its first frame, and `thread.started` may come before the
            // tools are listed.
            daemon
                .grants
                .mark_connected(&grant.project, grant.execution);
            Ok(json!({
                "protocolVersion": message["params"]["protocolVersion"].as_str().unwrap_or("2025-06-18"),
                "capabilities": { "tools": {} },
                "serverInfo": { "name": "yard", "version": env!("CARGO_PKG_VERSION") },
            }))
        }
        ("tools/list", Some(_)) => {
            daemon
                .grants
                .mark_connected(&grant.project, grant.execution);
            Ok(json!({ "tools": tools(grant.kind) }))
        }
        ("tools/call", Some(_)) => Ok(call(daemon, &grant, &message["params"]).await),
        _ => Err(json!({ "code": -32601, "message": format!("method {method:?} is not served") })),
    };
    let envelope = match result {
        Ok(result) => json!({ "jsonrpc": "2.0", "id": id, "result": result }),
        Err(error) => json!({ "jsonrpc": "2.0", "id": id, "error": error }),
    };
    let mut response = reply(StatusCode::OK, Some(envelope));
    if let Some(id) = opened {
        response
            .headers_mut()
            .insert("mcp-session-id", id.parse().expect("hex is a header value"));
    }
    response
}

async fn call(daemon: &Arc<Daemon>, grant: &Grant, params: &Value) -> Value {
    let name = params["name"].as_str().unwrap_or_default();
    let arguments = params.get("arguments").cloned().unwrap_or(json!({}));
    let granted = tools(grant.kind)
        .iter()
        .any(|tool| tool["name"].as_str() == Some(name));
    let outcome = if !granted {
        Err(Fail::refused(format!(
            "{name} is not granted to this execution"
        )))
    } else {
        match name {
            "yard_context" => crate::jobs::supervise::context(daemon, grant).await,
            "yard_progress" => progress(grant, &arguments),
            "yard_propose" => crate::jobs::propose(grant, &arguments),
            "yard_publish_review" => crate::jobs::review::publish(grant, &arguments),
            _ => unreachable!("granted tools are the four above"),
        }
    };
    daemon.wake.notify_one();
    match outcome {
        Ok(value) => json!({
            "content": [{ "type": "text", "text": match value {
                Value::String(text) => text,
                other => other.to_string(),
            } }],
        }),
        Err(fail) => {
            let _ = grant.project.tx(|tx| {
                crate::store::audit(
                    tx,
                    "tool.refused",
                    crate::store::Target {
                        execution: Some(grant.execution),
                        ..Default::default()
                    },
                    Some(&fail.message),
                    json!({ "tool": name, "code": fail.code }),
                )
            });
            json!({
                "content": [{ "type": "text", "text": format!("{}: {}", fail.code, fail.message) }],
                "isError": true,
            })
        }
    }
}

fn progress(grant: &Grant, arguments: &Value) -> Result<Value, Fail> {
    let object = strict(arguments, &["note"])?;
    let note = object["note"]
        .as_str()
        .ok_or_else(|| Fail::invalid("note must be a string"))?;
    if note.is_empty() || note.len() > NOTE_MAX {
        return Err(Fail::invalid(format!("note must be 1 to {NOTE_MAX} bytes")));
    }
    grant
        .project
        .read(|conn| crate::store::executions::set_progress(conn, grant.execution, note))?;
    Ok(json!("recorded"))
}

/// The object `value` holds, refusing any key not in `allowed`.
pub fn strict<'a>(
    value: &'a Value,
    allowed: &[&str],
) -> Result<&'a serde_json::Map<String, Value>, Fail> {
    let object = value
        .as_object()
        .ok_or_else(|| Fail::invalid("arguments must be an object"))?;
    if let Some(key) = object.keys().find(|key| !allowed.contains(&key.as_str())) {
        return Err(Fail::invalid(format!("unknown argument {key:?}")));
    }
    Ok(object)
}
