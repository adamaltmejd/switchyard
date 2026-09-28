//! The scripted fake model: an OpenAI chat-completions endpoint on the host.
//! Real Pi in a real box reaches it through a pinfold route, and the test's
//! script decides every reply.

use serde_json::{Value, json};
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::time::Duration;

/// Prompt tokens every reply reports.
pub const PROMPT_TOKENS: u64 = 1000;
/// Completion tokens every reply reports.
pub const COMPLETION_TOKENS: u64 = 100;

/// How long `Latch::wait_held` waits before it fails the test.
const HOLD_DEADLINE: Duration = Duration::from_secs(300);
const READ_TIMEOUT: Duration = Duration::from_secs(60);
const MAX_BODY: u64 = 64 << 20;

type Script = dyn Fn(&ModelRequest) -> Reply + Send + Sync;

pub struct FakeModel {
    port: u16,
    requests: Arc<Mutex<Vec<ModelRequest>>>,
}

impl FakeModel {
    /// Bind 127.0.0.1:0 and serve every request with `script` on its own
    /// thread until the process exits. A request whose body is not a JSON
    /// object is recorded and answered 400 without running the script. A
    /// script that panics answers 500.
    pub fn start(script: impl Fn(&ModelRequest) -> Reply + Send + Sync + 'static) -> FakeModel {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind the fake model");
        let port = listener.local_addr().expect("fake model address").port();
        let requests = Arc::new(Mutex::new(Vec::new()));
        let script: Arc<Script> = Arc::new(script);
        let log = requests.clone();
        thread::spawn(move || {
            for stream in listener.incoming().flatten() {
                let (script, log) = (script.clone(), log.clone());
                thread::spawn(move || serve(stream, &*script, &log));
            }
        });
        FakeModel { port, requests }
    }

    /// "http://127.0.0.1:PORT"
    pub fn origin(&self) -> String {
        format!("http://127.0.0.1:{}", self.port)
    }

    /// Every request received so far, in arrival order. A request is recorded
    /// before the script runs, so a held one is listed.
    pub fn requests(&self) -> Vec<ModelRequest> {
        self.requests.lock().unwrap().clone()
    }
}

#[derive(Clone, Debug)]
pub struct ModelRequest {
    pub path: String,
    pub headers: Vec<(String, String)>,
    /// The JSON body. A body that is not JSON is kept as a string.
    pub body: Value,
}

impl ModelRequest {
    /// The first header named `name`, case-insensitively.
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(n, _)| n.eq_ignore_ascii_case(name))
            .map(|(_, v)| v.as_str())
    }

    /// The text of every system and developer message, joined by newlines.
    /// The Anthropic API puts the system prompt in a top-level `system` field.
    pub fn system(&self) -> String {
        let mut texts: Vec<String> = self
            .messages()
            .iter()
            .filter(|m| m["role"] == "system" || m["role"] == "developer")
            .map(|m| text(&m["content"]))
            .collect();
        if let Some(system) = self.body.get("system") {
            texts.push(text(system));
        }
        texts.join("\n")
    }

    /// Everything the model was told, system prompt and messages together.
    pub fn context(&self) -> String {
        let mut texts = vec![self.system()];
        for message in self.messages() {
            texts.push(text(&message["content"]));
        }
        texts.join("\n")
    }

    /// The text of the first user message.
    pub fn prompt(&self) -> String {
        self.user_texts().next().unwrap_or_default()
    }

    /// The text of the last user message.
    pub fn last_user(&self) -> String {
        self.user_texts().last().unwrap_or_default()
    }

    /// Whether the request offers the tool `name`. OpenAI names it under
    /// `function.name`; Anthropic names it directly, and prefixes an MCP
    /// server's tools with `mcp__<server>__`.
    pub fn has_tool(&self, name: &str) -> bool {
        self.body["tools"]
            .as_array()
            .into_iter()
            .flatten()
            .any(|t| {
                let named = t["function"]["name"]
                    .as_str()
                    .or_else(|| t["name"].as_str())
                    .unwrap_or("");
                named == name || named.ends_with(&format!("__{name}"))
            })
    }

    /// Whether the conversation ends with a user message that is not a tool
    /// result: the first request of an execution, resumed session or not.
    pub fn opens(&self) -> bool {
        self.messages()
            .last()
            .is_some_and(|message| message["role"] == "user" && !is_tool_result(message))
    }

    /// The number of assistant messages already in the conversation.
    pub fn turn(&self) -> usize {
        self.messages()
            .iter()
            .filter(|m| m["role"] == "assistant")
            .count()
    }

    /// Every tool result as (tool name, result text), in order. The name comes
    /// from the tool call the result answers, in either wire shape.
    pub fn tool_results(&self) -> Vec<(String, String)> {
        let names = self.tool_names();
        let mut results = Vec::new();
        for message in self.messages() {
            if message["role"] == "tool" {
                let name = message["tool_call_id"]
                    .as_str()
                    .and_then(|id| names.get(id).map(String::as_str))
                    .or(message["name"].as_str())
                    .unwrap_or_default();
                results.push((name.to_owned(), text(&message["content"])));
            }
            for block in message["content"].as_array().into_iter().flatten() {
                if block["type"] == "tool_result" {
                    let name = block["tool_use_id"]
                        .as_str()
                        .and_then(|id| names.get(id).map(String::as_str))
                        .unwrap_or_default();
                    results.push((name.to_owned(), text(&block["content"])));
                }
            }
        }
        results
    }

    fn tool_names(&self) -> HashMap<String, String> {
        let mut names = HashMap::new();
        for message in self.messages() {
            for call in message["tool_calls"].as_array().into_iter().flatten() {
                if let (Some(id), Some(name)) =
                    (call["id"].as_str(), call["function"]["name"].as_str())
                {
                    names.insert(id.to_string(), name.to_string());
                }
            }
            for block in message["content"].as_array().into_iter().flatten() {
                if block["type"] == "tool_use"
                    && let (Some(id), Some(name)) = (block["id"].as_str(), block["name"].as_str())
                {
                    names.insert(id.to_string(), name.to_string());
                }
            }
        }
        names
    }

    pub fn last_tool_result(&self) -> Option<(String, String)> {
        self.tool_results().pop()
    }

    fn messages(&self) -> &[Value] {
        self.body["messages"].as_array().map_or(&[], Vec::as_slice)
    }

    fn user_texts(&self) -> impl DoubleEndedIterator<Item = String> + '_ {
        self.messages()
            .iter()
            .filter(|m| m["role"] == "user")
            .map(|m| text(&m["content"]))
    }
}

/// Message content is a string or an array of parts; only text parts count.
fn text(content: &Value) -> String {
    match content {
        Value::String(s) => s.clone(),
        Value::Array(parts) => parts.iter().filter_map(|p| p["text"].as_str()).collect(),
        _ => String::new(),
    }
}

/// An Anthropic user message that carries only tool results.
fn is_tool_result(message: &Value) -> bool {
    message["content"].as_array().is_some_and(|blocks| {
        !blocks.is_empty() && blocks.iter().all(|block| block["type"] == "tool_result")
    })
}

pub enum Reply {
    /// Assistant text, finish_reason "stop".
    Text(String),
    /// One assistant turn of tool calls, finish_reason "tool_calls".
    Tools(Vec<ToolCall>),
    /// Mark the latch held, block until it is released, then send the inner reply.
    Hold(Latch, Box<Reply>),
}

pub struct ToolCall {
    pub name: String,
    pub arguments: Value,
}

/// A call to Pi's built-in bash tool.
pub fn bash(command: &str) -> ToolCall {
    tool("bash", json!({ "command": command }))
}

pub fn tool(name: &str, arguments: Value) -> ToolCall {
    ToolCall {
        name: name.to_owned(),
        arguments,
    }
}

/// A point a request is held at until the test releases it. Once released it
/// stays released: a later hold on the same latch passes straight through.
#[derive(Clone, Default)]
pub struct Latch(Arc<(Mutex<LatchState>, Condvar)>);

#[derive(Default)]
struct LatchState {
    held: bool,
    released: bool,
}

impl Latch {
    pub fn new() -> Latch {
        Latch::default()
    }

    /// Block until a request is held. Panics after a generous deadline: a
    /// hold that never comes is a failure, never a pass.
    pub fn wait_held(&self) {
        let (lock, cvar) = &*self.0;
        let state = lock.lock().unwrap();
        let (state, _) = cvar
            .wait_timeout_while(state, HOLD_DEADLINE, |s| !s.held)
            .unwrap();
        let held = state.held;
        // Unlock before panicking so the server thread's mutex isn't poisoned.
        drop(state);
        assert!(
            held,
            "fake model: no request was held within {HOLD_DEADLINE:?}"
        );
    }

    pub fn release(&self) {
        let (lock, cvar) = &*self.0;
        lock.lock().unwrap().released = true;
        cvar.notify_all();
    }

    /// Whether a request is held now.
    pub fn is_held(&self) -> bool {
        let state = self.0.0.lock().unwrap();
        state.held && !state.released
    }

    fn hold(&self) {
        let (lock, cvar) = &*self.0;
        let mut state = lock.lock().unwrap();
        state.held = true;
        cvar.notify_all();
        drop(cvar.wait_while(state, |s| !s.released).unwrap());
    }
}

fn serve(mut stream: TcpStream, script: &Script, log: &Mutex<Vec<ModelRequest>>) {
    let _ = stream.set_read_timeout(Some(READ_TIMEOUT));
    let response = match read_request(&stream) {
        None => respond(400, "application/json", error_body(400)),
        Some(request) => {
            log.lock().unwrap().push(request.clone());
            if !request.body.is_object() {
                respond(400, "application/json", error_body(400))
            } else {
                // A panicking script answers 500; the default hook has printed it.
                match catch_unwind(AssertUnwindSafe(|| script(&request))) {
                    Ok(reply) => render(reply, &request),
                    Err(_) => respond(500, "application/json", error_body(500)),
                }
            }
        }
    };
    let _ = stream.write_all(&response);
}

/// Reads one HTTP/1.1 request framed by Content-Length. `None` when the head
/// is not HTTP; a body that can't be read or parsed is kept as a string.
fn read_request(stream: &TcpStream) -> Option<ModelRequest> {
    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    reader.read_line(&mut line).ok()?;
    let mut words = line.split_whitespace();
    let (_method, path) = (words.next()?, words.next()?);
    let path = path.to_owned();
    let mut headers = Vec::new();
    loop {
        line.clear();
        if reader.read_line(&mut line).ok()? == 0 {
            return None;
        }
        let line = line.trim_end();
        if line.is_empty() {
            break;
        }
        let (name, value) = line.split_once(':')?;
        headers.push((name.trim().to_owned(), value.trim().to_owned()));
    }
    let length: u64 = headers
        .iter()
        .find(|(n, _)| n.eq_ignore_ascii_case("content-length"))
        .map_or(Some(0), |(_, v)| v.parse().ok())?;
    let mut raw = Vec::new();
    let _ = reader.take(length.min(MAX_BODY)).read_to_end(&mut raw);
    let body = serde_json::from_slice(&raw)
        .unwrap_or_else(|_| Value::String(String::from_utf8_lossy(&raw).into_owned()));
    Some(ModelRequest {
        path,
        headers,
        body,
    })
}

fn render(reply: Reply, request: &ModelRequest) -> Vec<u8> {
    match reply {
        Reply::Hold(latch, inner) => {
            latch.hold();
            render(*inner, request)
        }
        Reply::Text(text) if anthropic(request) => anthropic_text(&text, request),
        Reply::Tools(calls) if anthropic(request) => anthropic_tools(calls, request),
        reply => openai(reply, request),
    }
}

/// The Anthropic Messages API, streamed as SSE.
fn anthropic(request: &ModelRequest) -> bool {
    request.path == "/v1/messages" || request.path.starts_with("/v1/messages?")
}

fn anthropic_text(text: &str, request: &ModelRequest) -> Vec<u8> {
    let event = |name: &str, data: Value| format!("event: {name}\ndata: {data}\n\n");
    let body = [
        event(
            "message_start",
            json!({
                "type": "message_start",
                "message": anthropic_message(request),
            }),
        ),
        event(
            "content_block_start",
            json!({ "type": "content_block_start", "index": 0,
                    "content_block": { "type": "text", "text": "" } }),
        ),
        event(
            "content_block_delta",
            json!({ "type": "content_block_delta", "index": 0,
                    "delta": { "type": "text_delta", "text": text } }),
        ),
        event(
            "content_block_stop",
            json!({ "type": "content_block_stop", "index": 0 }),
        ),
        anthropic_delta("end_turn"),
        event("message_stop", json!({ "type": "message_stop" })),
    ]
    .concat();
    respond(200, "text/event-stream", body)
}

fn anthropic_tools(calls: Vec<ToolCall>, request: &ModelRequest) -> Vec<u8> {
    let event = |name: &str, data: Value| format!("event: {name}\ndata: {data}\n\n");
    let turn = request.turn();
    let mut body = event(
        "message_start",
        json!({
            "type": "message_start",
            "message": anthropic_message(request),
        }),
    );
    for (index, call) in calls.into_iter().enumerate() {
        body.push_str(&event(
            "content_block_start",
            json!({ "type": "content_block_start", "index": index,
                    "content_block": { "type": "tool_use", "id": format!("toolu_{turn}_{index}"),
                                      "name": call.name, "input": {} } }),
        ));
        body.push_str(&event(
            "content_block_delta",
            json!({ "type": "content_block_delta", "index": index,
                    "delta": { "type": "input_json_delta", "partial_json": call.arguments.to_string() } }),
        ));
        body.push_str(&event(
            "content_block_stop",
            json!({ "type": "content_block_stop", "index": index }),
        ));
    }
    body.push_str(&anthropic_delta("tool_use"));
    body.push_str(&event("message_stop", json!({ "type": "message_stop" })));
    respond(200, "text/event-stream", body)
}

fn anthropic_message(request: &ModelRequest) -> Value {
    json!({
        "id": format!("msg_fake_{}", request.turn()),
        "type": "message",
        "role": "assistant",
        "model": request.body["model"].as_str().unwrap_or("fake-model"),
        "content": [],
        "stop_reason": null,
        "stop_sequence": null,
        "usage": { "input_tokens": PROMPT_TOKENS, "output_tokens": 0 },
    })
}

fn anthropic_delta(stop_reason: &str) -> String {
    format!(
        "event: message_delta\ndata: {}\n\n",
        json!({
            "type": "message_delta",
            "delta": { "stop_reason": stop_reason, "stop_sequence": null },
            "usage": { "output_tokens": COMPLETION_TOKENS },
        })
    )
}

/// The OpenAI chat-completions API, streamed as SSE.
fn openai(reply: Reply, request: &ModelRequest) -> Vec<u8> {
    let turn = request.turn();
    let (delta, finish) = match reply {
        Reply::Text(text) => (json!({ "role": "assistant", "content": text }), "stop"),
        Reply::Tools(calls) => {
            // Ids carry the turn so they stay unique across the conversation.
            let calls: Vec<Value> = calls
                .into_iter()
                .enumerate()
                .map(|(i, call)| {
                    json!({
                        "index": i,
                        "id": format!("call_{turn}_{i}"),
                        "type": "function",
                        "function": { "name": call.name, "arguments": call.arguments.to_string() },
                    })
                })
                .collect();
            (
                json!({ "role": "assistant", "tool_calls": calls }),
                "tool_calls",
            )
        }
        Reply::Hold(_, _) => unreachable!("the hold is released before rendering"),
    };
    let model = request.body["model"].as_str().unwrap_or("fake-model");
    let chunk = |choices: Value, usage: Value| {
        let chunk = json!({
            "id": format!("chatcmpl-fake-{turn}"),
            "object": "chat.completion.chunk",
            "created": 0,
            "model": model,
            "choices": choices,
            "usage": usage,
        });
        format!("data: {chunk}\n\n")
    };
    let usage = json!({
        "prompt_tokens": PROMPT_TOKENS,
        "completion_tokens": COMPLETION_TOKENS,
        "total_tokens": PROMPT_TOKENS + COMPLETION_TOKENS,
    });
    let body = [
        chunk(
            json!([{ "index": 0, "delta": delta, "finish_reason": null }]),
            Value::Null,
        ),
        chunk(
            json!([{ "index": 0, "delta": {}, "finish_reason": finish }]),
            Value::Null,
        ),
        chunk(json!([]), usage),
        "data: [DONE]\n\n".to_owned(),
    ]
    .concat();
    respond(200, "text/event-stream", body)
}

fn error_body(code: u16) -> String {
    json!({ "error": { "message": format!("fake model status {code}"), "code": code } }).to_string()
}

fn respond(code: u16, content_type: &str, body: String) -> Vec<u8> {
    format!(
        "HTTP/1.1 {code} Fake\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nCache-Control: no-cache\r\nConnection: close\r\n\r\n{body}",
        body.len()
    )
    .into_bytes()
}
