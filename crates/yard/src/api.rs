//! The one API: `POST /rpc` on the daemon's unix socket with
//! `{method, params}`, answered by `{boundary, ok, result | error}`.

use http_body_util::{BodyExt, Full, Limited};
use hyper::body::{Bytes, Incoming};
use hyper::{Request, Response};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

const BODY_MAX: usize = 8 * 1024 * 1024;
pub const BOUNDARY_HEADER: &str = "yard-boundary";

/// A refused or failed call, as data. `code` is a spec token.
#[derive(Debug)]
pub struct Fail {
    pub code: &'static str,
    pub message: String,
    pub data: Value,
}

impl Fail {
    pub fn new(code: &'static str, message: impl Into<String>) -> Fail {
        Fail {
            code,
            message: message.into(),
            data: Value::Null,
        }
    }
    pub fn refused(message: impl Into<String>) -> Fail {
        Fail::new("refused", message)
    }
    pub fn invalid(message: impl Into<String>) -> Fail {
        Fail::new("invalid", message)
    }
    pub fn not_found(message: impl Into<String>) -> Fail {
        Fail::new("not_found", message)
    }
    /// A mutation named an identity that is no longer current.
    pub fn stale(message: impl Into<String>, expected: Value, current: Value) -> Fail {
        Fail {
            code: "stale",
            message: message.into(),
            data: json!({ "expected": expected, "current": current }),
        }
    }
    pub fn with(mut self, data: Value) -> Fail {
        self.data = data;
        self
    }
    pub fn to_json(&self) -> Value {
        json!({ "code": self.code, "message": self.message, "data": self.data })
    }
}

impl From<rusqlite::Error> for Fail {
    fn from(error: rusqlite::Error) -> Fail {
        Fail::new("internal", format!("store: {error}"))
    }
}

impl From<String> for Fail {
    fn from(message: String) -> Fail {
        Fail::new("internal", message)
    }
}

/// This build's identity: the digest of the running executable. Client and
/// daemon are one binary, so any rebuild is a new boundary.
pub fn boundary() -> &'static str {
    static BOUNDARY: OnceLock<String> = OnceLock::new();
    BOUNDARY.get_or_init(|| {
        use sha2::{Digest, Sha256};
        let bytes = std::env::current_exe()
            .and_then(std::fs::read)
            .unwrap_or_default();
        crate::config::hex(&Sha256::digest(&bytes))[..16].to_string()
    })
}

/// `$XDG_STATE_HOME/yard`, `~/.local/state/yard` when unset.
pub fn state_dir() -> PathBuf {
    xdg("XDG_STATE_HOME", ".local/state").join("yard")
}

pub fn config_dir() -> PathBuf {
    xdg("XDG_CONFIG_HOME", ".config").join("yard")
}

fn xdg(var: &str, fallback: &str) -> PathBuf {
    match std::env::var_os(var) {
        Some(dir) if !dir.is_empty() => PathBuf::from(dir),
        _ => PathBuf::from(std::env::var_os("HOME").unwrap_or_default()).join(fallback),
    }
}

pub fn socket_path() -> PathBuf {
    state_dir().join("yard.sock")
}

/// Answer one HTTP request with the envelope.
pub async fn serve<F, Fut>(
    request: Request<Incoming>,
    handle: F,
) -> Result<Response<Full<Bytes>>, hyper::Error>
where
    F: FnOnce(String, Value) -> Fut,
    Fut: std::future::Future<Output = Result<Value, Fail>>,
{
    let theirs = request
        .headers()
        .get(BOUNDARY_HEADER)
        .and_then(|value| value.to_str().ok())
        .map(str::to_string);
    let outcome = async {
        if request.uri().path() != "/rpc" || request.method() != hyper::Method::POST {
            return Err(Fail::invalid("the API is POST /rpc"));
        }
        if theirs.as_deref() != Some(boundary()) {
            return Err(Fail::new(
                "boundary",
                "this yard and the daemon are different builds; restart the daemon (yard daemon restart)",
            ));
        }
        let body = Limited::new(request.into_body(), BODY_MAX)
            .collect()
            .await
            .map_err(|error| Fail::invalid(format!("request body: {error}")))?
            .to_bytes();
        let call: Value = serde_json::from_slice(&body)
            .map_err(|error| Fail::invalid(format!("request is not JSON: {error}")))?;
        let method = call["method"]
            .as_str()
            .ok_or_else(|| Fail::invalid("request names no method"))?
            .to_string();
        handle(method, call["params"].clone()).await
    }
    .await;
    let envelope = match outcome {
        Ok(result) => json!({ "boundary": boundary(), "ok": true, "result": result }),
        Err(fail) => json!({ "boundary": boundary(), "ok": false, "error": fail.to_json() }),
    };
    Ok(Response::builder()
        .header("content-type", "application/json")
        .body(Full::new(Bytes::from(envelope.to_string())))
        .expect("response builds"))
}

/// One call to the daemon at `socket`.
pub async fn call(socket: &Path, method: &str, params: Value) -> Result<Value, Fail> {
    let stream = tokio::net::UnixStream::connect(socket).await.map_err(|error| {
        Fail::new(
            "daemon",
            format!(
                "no daemon answers at {}: {error}; start it with `yard daemon install` or `yard daemon run`",
                socket.display()
            ),
        )
    })?;
    let io = hyper_util::rt::TokioIo::new(stream);
    let (mut sender, connection) = hyper::client::conn::http1::handshake(io)
        .await
        .map_err(|error| Fail::new("daemon", format!("handshake: {error}")))?;
    tokio::spawn(connection);
    let body = json!({ "method": method, "params": params }).to_string();
    let request = Request::post("/rpc")
        .header("host", "yard")
        .header("content-type", "application/json")
        .header(BOUNDARY_HEADER, boundary())
        .body(Full::new(Bytes::from(body)))
        .expect("request builds");
    let response = sender
        .send_request(request)
        .await
        .map_err(|error| Fail::new("daemon", format!("the daemon hung up: {error}")))?;
    let body = Limited::new(response.into_body(), 64 * 1024 * 1024)
        .collect()
        .await
        .map_err(|error| Fail::new("daemon", format!("response body: {error}")))?
        .to_bytes();
    let envelope: Value = serde_json::from_slice(&body)
        .map_err(|error| Fail::new("daemon", format!("response is not JSON: {error}")))?;
    if envelope["ok"] == Value::Bool(true) {
        Ok(envelope["result"].clone())
    } else {
        let error = &envelope["error"];
        let code = ["refused", "invalid", "not_found", "stale", "boundary"]
            .into_iter()
            .find(|code| error["code"] == *code)
            .unwrap_or("internal");
        Err(Fail {
            code,
            message: error["message"].as_str().unwrap_or_default().to_string(),
            data: error["data"].clone(),
        })
    }
}
