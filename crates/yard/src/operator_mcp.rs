//! Trusted stdio client. Worker HTTP grants never enter this dispatch.

use crate::api::{self, Fail};
use serde_json::{Value, json};
use std::collections::HashMap;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::task::JoinSet;

const FRAME_MAX: u64 = 1024 * 1024;
const IN_FLIGHT_MAX: usize = 64;

// A trailing ! marks a required field. Daemon operations remain authoritative.
const OPERATIONS: &[(&str, &str)] = &[
    ("daemon.status", ""),
    ("project.list", ""),
    ("project.forget", "path!"),
    ("init", "path!"),
    ("doctor", "project!"),
    (
        "plan.resolve",
        "project! target preset workflow graph:o overrides:o",
    ),
    (
        "plan.validate",
        "project! target preset workflow graph:o overrides:o",
    ),
    ("sync", "project! branch default:b"),
    ("status", "project!"),
    ("events", "project! since:i"),
    ("attention", "project! since:i"),
    (
        "ticket.new",
        "project! title! body priority workflow target depends_on:a parked:b",
    ),
    ("ticket.show", "project! ticket!"),
    ("ticket.list", "project!"),
    (
        "ticket.edit",
        "project! ticket! revision:i! title body priority workflow target default_target:b",
    ),
    ("ticket.park", "project! ticket!"),
    ("ticket.unpark", "project! ticket!"),
    ("ticket.depend", "project! ticket! on!"),
    ("ticket.done", "project! ticket! reason!"),
    ("ticket.abandon", "project! ticket! reason"),
    (
        "attempt.start",
        "project! ticket! attention:i preset workflow graph:o overrides:o",
    ),
    ("attempt.stop", "project! ticket!"),
    (
        "attempt.accept",
        "project! ticket! attention:i! check:i! plan_revision:i! target! head! proof text!",
    ),
    (
        "attempt.repair",
        "project! ticket! attention:i! check:i! plan_revision:i! target! head! proof text!",
    ),
    (
        "attempt.approve",
        "project! ticket! head! target! proof text",
    ),
    (
        "attempt.reject",
        "project! ticket! head! target! proof text!",
    ),
    ("attempt.abandon", "project! ticket! reason"),
    ("attempt.show", "project! ticket!"),
    ("attempt.diff", "project! ticket!"),
    ("attempt.tail", "project! ticket!"),
    ("attempt.stats", "project! ticket!"),
    ("proposal.accept", "project! attention:i! text"),
    ("proposal.reject", "project! attention:i! text"),
];

fn name(method: &str) -> String {
    format!("yard_{}", method.replace('.', "_"))
}

fn schema(fields: &str) -> Value {
    let mut properties = serde_json::Map::new();
    let mut required = Vec::new();
    for field in fields.split_whitespace() {
        let key = field.trim_end_matches('!');
        let (key, kind) = key.split_once(':').unwrap_or((key, "s"));
        let value = match key {
            "graph" => json!({"type": "object", "properties": {
                "inputs": {"type": "object", "additionalProperties": output_schema()},
                "nodes": {"type": "object", "additionalProperties": node_schema()}
            }, "required": ["nodes"], "additionalProperties": false}),
            "overrides" => json!({"type": "object", "additionalProperties": node_schema()}),
            _ => match kind {
                "o" => json!({"type": "object"}),
                "i" => json!({"type": "integer"}),
                "b" => json!({"type": "boolean"}),
                "a" => json!({"type": "array", "items": {"type": "string"}}),
                _ => json!({"type": "string"}),
            },
        };
        properties.insert(key.into(), value);
        if field.ends_with('!') {
            required.push(key);
        }
    }
    json!({"type": "object", "properties": properties, "required": required,
           "additionalProperties": false})
}

fn output_schema() -> Value {
    json!({"type": "string", "enum": ["candidate", "report", "verdict", "selection"]})
}

fn node_schema() -> Value {
    json!({"type": "object", "additionalProperties": false, "properties": {
        "profile": {"type": "string"},
        "operator_failure": {"type": "boolean"},
        "runner": {"oneOf": [
            {"type": "object", "properties": {"kind": {"const": "agent"},
                "agent": {"type": "string"}, "instructions": {"type": "string"}},
                "required": ["kind", "agent", "instructions"], "additionalProperties": false},
            {"type": "object", "properties": {"kind": {"const": "command"},
                "gate": {"type": "string"}}, "required": ["kind", "gate"], "additionalProperties": false}
        ]},
        "access": {"type": "string", "enum": ["read-only", "write-allowed"]},
        "outputs": {"type": "object", "additionalProperties": output_schema()},
        "outcomes": {"type": "array", "items": {"type": "string"}},
        "inputs": {"type": "object", "additionalProperties": {
            "type": "object", "properties": {"node": {"type": ["string", "null"]},
                "output": {"type": "string"}, "kind": output_schema()},
            "required": ["output", "kind"], "additionalProperties": false}},
        "requires": {"type": "array", "items": {"type": "string"}},
        "routes": {"type": "object", "additionalProperties": {"type": "array", "items": {
            "type": "object", "properties": {"to": {"type": "string"},
                "repeat": {"type": ["integer", "null"], "minimum": 1}},
            "required": ["to"], "additionalProperties": false}}}
    }})
}

fn validate(arguments: &Value, schema: &Value) -> Result<(), Fail> {
    let properties = schema["properties"].as_object().expect("schema properties");
    let keys: Vec<_> = properties.keys().map(String::as_str).collect();
    let object = crate::mcp::strict(arguments, &keys)?;
    for key in schema["required"].as_array().expect("schema required") {
        if !object.contains_key(key.as_str().expect("field name")) {
            return Err(Fail::invalid(format!("missing argument {key}")));
        }
    }
    for (key, value) in object {
        let valid = match properties[key]["type"].as_str() {
            Some("object") => value.is_object(),
            Some("integer") => value.as_i64().is_some(),
            Some("boolean") => value.is_boolean(),
            Some("array") => value
                .as_array()
                .is_some_and(|a| a.iter().all(Value::is_string)),
            _ => value.is_string(),
        };
        if !valid {
            return Err(Fail::invalid(format!("invalid argument {key}")));
        }
        if matches!(key.as_str(), "project" | "path")
            && !std::path::Path::new(value.as_str().expect("validated path")).is_absolute()
        {
            return Err(Fail::invalid(format!("{key} must be an absolute path")));
        }
    }
    Ok(())
}

fn tool_result(outcome: Result<Value, Fail>) -> Value {
    let (value, failed) = match outcome {
        Ok(value) => (json!({"result": value}), false),
        Err(fail) => (json!({"error": fail.to_json()}), true),
    };
    json!({"content": [{"type": "text", "text": value.to_string()}],
           "structuredContent": value, "isError": failed})
}

async fn forward(method: &str, fields: &str, mut args: Value) -> Result<Value, Fail> {
    validate(&args, &schema(fields))?;
    if matches!(method, "init" | "project.forget") {
        let path = std::path::Path::new(args["path"].as_str().expect("validated path"));
        let canonical = match std::fs::canonicalize(path) {
            Ok(path) => path,
            Err(_) if method == "project.forget" => path.to_path_buf(),
            Err(error) => return Err(Fail::invalid(error.to_string())),
        };
        args["path"] = json!(canonical);
    }
    let mut value = api::call(&api::socket_path(), method, args).await?;
    // Ticket bodies have an explicit show drilldown.
    let tickets = if method == "status" {
        value.get_mut("tickets").and_then(Value::as_array_mut)
    } else if method == "ticket.list" {
        value.as_array_mut()
    } else {
        None
    };
    for ticket in tickets.into_iter().flatten() {
        if let Some(object) = ticket.as_object_mut() {
            object.remove("body");
        }
    }
    Ok(value)
}

async fn answer(message: &Value) -> Value {
    let id = &message["id"];
    let result = match message["method"].as_str().unwrap_or_default() {
        "initialize" => json!({
            "protocolVersion": "2025-06-18", "capabilities": {"tools": {}},
            "serverInfo": {"name": "yard-operator", "version": env!("CARGO_PKG_VERSION")},
        }),
        "ping" => json!({}),
        "tools/list" => json!({"tools": OPERATIONS.iter().map(|(method, fields)| {
            json!({"name": name(method), "description": match *method {
                "events" => "One history page, at most 500 events; waits at most 25 seconds.",
                "attention" => "Wait at most 25 seconds for attention or final drain; cancellation releases only this wait.",
                "attempt.start" => "Start or retry through the existing attempt lifecycle.",
                "attempt.tail" => "Read the live worker transcript explicitly.",
                "attempt.stats" => "Read recorded execution statistics; missing values remain null.",
                _ => method,
            }, "inputSchema": schema(fields)})
        }).collect::<Vec<_>>()}),
        "tools/call" => {
            let params = &message["params"];
            let operation = OPERATIONS
                .iter()
                .find(|(method, _)| params["name"] == name(method));
            let outcome = match operation {
                None => Err(Fail::invalid("unknown operator tool")),
                Some((method, fields)) => {
                    forward(
                        method,
                        fields,
                        params.get("arguments").cloned().unwrap_or(json!({})),
                    )
                    .await
                }
            };
            tool_result(outcome)
        }
        _ => {
            return json!({"jsonrpc": "2.0", "id": id,
                          "error": {"code": -32601, "message": "unknown method"}});
        }
    };
    json!({"jsonrpc": "2.0", "id": id, "result": result})
}

async fn write(stdout: &mut tokio::io::Stdout, response: &Value) -> std::io::Result<()> {
    stdout.write_all(format!("{response}\n").as_bytes()).await?;
    stdout.flush().await
}

/// Newline-delimited JSON-RPC. Only read waits can be cancelled; mutations
/// finish even if stdin closes. Dropping a wait never sends a stop command.
pub async fn serve() -> std::io::Result<()> {
    let mut tasks = JoinSet::new();
    let mut pending = HashMap::new();
    let result = receive(&mut tasks, &mut pending).await;
    for (handle, wait) in pending.values() {
        if *wait {
            handle.abort();
        }
    }
    // Transport failure cannot retract a mutation we already accepted.
    let mut stdout = tokio::io::stdout();
    let mut writable = true;
    while let Some(completed) = tasks.join_next_with_id().await {
        if let Ok((task, (key, response))) = completed
            && pending
                .get(&key)
                .is_some_and(|(handle, _)| handle.id() == task)
        {
            pending.remove(&key);
            if writable {
                writable = write(&mut stdout, &response).await.is_ok();
            }
        }
    }
    result
}

async fn receive(
    tasks: &mut JoinSet<(String, Value)>,
    pending: &mut HashMap<String, (tokio::task::AbortHandle, bool)>,
) -> std::io::Result<()> {
    let mut input = BufReader::new(tokio::io::stdin());
    let mut stdout = tokio::io::stdout();
    let mut frame = Vec::new();
    let mut eof = false;
    loop {
        let mut bounded = (&mut input).take(FRAME_MAX + 1 - frame.len() as u64);
        tokio::select! {
            result = tasks.join_next_with_id(), if !tasks.is_empty() => {
                if let Some(Ok((task, (key, response)))) = result
                    && pending.get(&key).is_some_and(|(handle, _)| handle.id() == task) {
                    pending.remove(&key);
                    write(&mut stdout, &response).await?;
                }
            }
            read = bounded.read_until(b'\n', &mut frame), if !eof => {
                let count = read?;
                if count == 0 {
                    eof = true;
                    for (handle, wait) in pending.values() {
                        if *wait { handle.abort(); }
                    }
                } else {
                    if frame.len() as u64 > FRAME_MAX {
                        write(&mut stdout, &json!({"jsonrpc": "2.0", "id": null,
                            "error": {"code": -32600, "message": "MCP frame too large"}})).await?;
                        return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "MCP frame too large"));
                    }
                    let message = serde_json::from_slice::<Value>(&frame);
                    frame.clear();
                    let message = match message {
                        Ok(m) if m.is_object() && m["jsonrpc"] == "2.0" && m["method"].is_string() => m,
                        other => {
                            write(&mut stdout, &json!({"jsonrpc": "2.0", "id": null,
                                "error": {"code": if other.is_err() { -32700 } else { -32600 }, "message": "invalid frame"}})).await?;
                            continue;
                        }
                    };
                    if message.get("id").is_none() {
                        if message["method"] == "notifications/cancelled" {
                            let key = message["params"]["requestId"].to_string();
                            if pending.get(&key).is_some_and(|(_, wait)| *wait) {
                                let (handle, _) = pending.remove(&key).expect("pending wait");
                                handle.abort();
                                write(&mut stdout, &json!({"jsonrpc": "2.0", "id": message["params"]["requestId"],
                                    "error": {"code": -32800, "message": "cancelled"}})).await?;
                            }
                        }
                        continue;
                    }
                    if !message["id"].is_string() && message["id"].as_i64().is_none() {
                        write(&mut stdout, &json!({"jsonrpc": "2.0", "id": null,
                            "error": {"code": -32600, "message": "invalid request id"}})).await?;
                        continue;
                    }
                    let key = message["id"].to_string();
                    if pending.contains_key(&key) || tasks.len() >= IN_FLIGHT_MAX {
                        write(&mut stdout, &json!({"jsonrpc": "2.0", "id": message["id"],
                            "error": {"code": -32600, "message": "duplicate id or too many requests"}})).await?;
                        continue;
                    }
                    let wait = message["method"] == "tools/call"
                        && matches!(message["params"]["name"].as_str(), Some("yard_attention" | "yard_events"));
                    let task_key = key.clone();
                    let handle = tasks.spawn(async move { (task_key, answer(&message).await) });
                    pending.insert(key, (handle, wait));
                }
            }
            else => break,
        }
    }
    Ok(())
}
