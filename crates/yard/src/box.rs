//! The pinfold process interface. Every call is a child process with an
//! explicit cwd, a scrubbed environment, bounded output and a time bound. It
//! returns what pinfold said as data and never touches a store.

use std::collections::BTreeMap;
use std::os::unix::process::ExitStatusExt;
use std::path::{Path, PathBuf};
use std::process::{ExitStatus, Stdio};
use std::time::Duration;

use nix::sys::signal::{Signal, kill, killpg};
use nix::unistd::Pid;
use serde::ser::SerializeMap;
use serde::{Deserialize, Serialize, Serializer};
use serde_json::Value;
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, ChildStdout, Command};
use tokio::task::JoinHandle;

const PROGRAM: &str = "pinfold";
/// What is kept of one output stream, or of one line; the rest is drained.
const OUTPUT_CAP: usize = 1 << 20;
/// `stat`, `down`, `list` and `prune` answer from the runtime's state.
const CONTROL_TIMEOUT: Duration = Duration::from_secs(30);
/// How long a SIGTERMed `up` gets to remove its box before it is killed.
const TEARDOWN_GRACE: Duration = Duration::from_secs(30);
const ABSENT_EXIT: i32 = 3;

#[derive(Debug, Clone)]
pub struct Pinfold {
    env: Vec<(String, String)>,
    cwd: PathBuf,
}

#[derive(Debug, Clone, Serialize)]
pub struct BoxSpec {
    pub name: String,
    pub labels: BTreeMap<String, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub harness: Option<String>,
    pub image: String,
    pub mounts: Vec<Mount>,
    pub env: BTreeMap<String, EnvValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub egress: Option<Egress>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cpus: Option<f64>,
    /// A whole number and `M` or `G`, at least `256M`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Mount {
    pub host: PathBuf,
    pub guest: String,
    pub readonly: bool,
}

/// A literal value, or `{"from": NAME}`: read from `up`'s own environment,
/// so the value never reaches argv or the spec.
#[derive(Debug, Clone)]
pub enum EnvValue {
    Value(String),
    From(String),
}

impl Serialize for EnvValue {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        match self {
            EnvValue::Value(value) => s.serialize_str(value),
            EnvValue::From(from) => {
                let mut map = s.serialize_map(Some(1))?;
                map.serialize_entry("from", from)?;
                map.end()
            }
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Egress {
    pub allow: Vec<String>,
    pub routes: BTreeMap<String, Route>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum Route {
    /// A host service as `host:port`.
    Service(String),
    /// Dials `to` and sets each header on the host side.
    Inject {
        to: String,
        headers: BTreeMap<String, Header>,
    },
}

#[derive(Debug, Clone, Serialize)]
pub struct Header {
    pub from: String,
    pub prefix: String,
}

/// `reason` is pinfold's refusal (`spec`, `profile`, `runtime`,
/// `image-missing`, `name-in-use`), or `failed`, `down` (a signal ended `up`
/// before ready), `timeout`, `spawn`, `eof` or `protocol` (a line that is
/// not pinfold's).
#[derive(Debug)]
pub struct UpError {
    pub reason: String,
    pub detail: String,
}

#[derive(Debug)]
pub struct ExecOutput {
    /// 128+n for a signal death.
    pub code: i32,
    pub stdout: String,
    pub stderr: String,
}

#[derive(Debug)]
pub enum ExecError {
    Absent,
    Timeout,
    Spawn(String),
}

impl std::fmt::Display for ExecError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExecError::Absent => write!(f, "the box is absent"),
            ExecError::Timeout => write!(f, "the command timed out"),
            ExecError::Spawn(detail) => write!(f, "{detail}"),
        }
    }
}

impl std::fmt::Display for BuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BuildError::Failed(log) => write!(f, "the build failed:\n{}", log.join("\n")),
            BuildError::Refused { reason, detail } => write!(f, "refused ({reason}): {detail}"),
            BuildError::Other(detail) => write!(f, "{detail}"),
        }
    }
}

#[derive(Debug)]
pub struct Stat {
    /// Monotonic for the box's life; null where the runtime cannot answer.
    pub oom_kills: Option<u64>,
}

#[derive(Debug)]
pub struct Listed {
    pub name: String,
}

#[derive(Debug)]
pub struct Built {
    /// The build's own `ref`, unique to this build.
    pub reference: String,
}

#[derive(Debug)]
pub enum BuildError {
    /// The last lines of the runtime's build output.
    Failed(Vec<String>),
    Refused {
        reason: String,
        detail: String,
    },
    Other(String),
}

/// A box this process brought up. Dropping it closes `up`'s stdin, which is
/// pinfold's `down`; the `up` process is never killed for it.
#[derive(Debug)]
pub struct LiveBox {
    pub name: String,
    pub image_id: Option<String>,
    child: Child,
    stdin: Option<ChildStdin>,
    end: JoinHandle<Result<String, String>>,
}

/// Any line pinfold prints; each call reads the fields it needs.
#[derive(Default, Deserialize)]
struct Line {
    event: Option<String>,
    reason: Option<String>,
    detail: Option<Value>,
    image: Option<Value>,
    #[serde(rename = "ref")]
    reference: Option<String>,
    log: Option<Vec<String>>,
    #[serde(rename = "box")]
    name: Option<String>,
}

impl Line {
    fn parse(bytes: &[u8]) -> Option<Line> {
        serde_json::from_slice(bytes).ok()
    }

    fn detail(&self) -> String {
        match &self.detail {
            Some(Value::String(text)) => text.clone(),
            Some(other) => other.to_string(),
            None => String::new(),
        }
    }

    fn image_id(&self) -> Option<String> {
        let id = self.image.as_ref()?.get("id")?.as_str()?;
        Some(id.to_string())
    }
}

#[derive(Deserialize)]
struct ListLine {
    name: String,
}

#[derive(Deserialize)]
struct StatLine {
    oom_kills: Option<u64>,
}

struct Collected {
    code: i32,
    stdout: String,
    stderr: String,
}

enum RunError {
    Timeout,
    Spawn(String),
}

impl std::fmt::Display for RunError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RunError::Timeout => f.write_str("timed out"),
            RunError::Spawn(detail) => write!(f, "could not start pinfold: {detail}"),
        }
    }
}

impl Pinfold {
    /// `env` is everything a pinfold child inherits; `cwd` is where it runs.
    pub fn new(env: Vec<(String, String)>, cwd: PathBuf) -> Pinfold {
        Pinfold { env, cwd }
    }

    /// Every child is its own process group, so a signal to the daemon's
    /// group does not reach it, and a timeout kills what it started.
    fn command(&self, args: &[&str]) -> Command {
        let mut command = Command::new(PROGRAM);
        command
            .args(args)
            .current_dir(&self.cwd)
            .env_clear()
            .envs(self.env.iter().map(|(k, v)| (k, v)))
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .process_group(0);
        command
    }

    fn exec_command(&self, name: &str, workdir: Option<&str>, argv: &[String]) -> Command {
        let mut args = vec!["box", "exec", name];
        if let Some(dir) = workdir {
            args.extend(["--workdir", dir]);
        }
        args.push("--");
        args.extend(argv.iter().map(String::as_str));
        self.command(&args)
    }

    /// Writes the spec and keeps stdin open until `ready`. `secrets` reach
    /// this child's environment only, for the spec's `{"from": NAME}`.
    /// `timeout` bounds bring-up; a late `up` is SIGTERMed, which removes
    /// whatever it made.
    pub async fn up(
        &self,
        spec: &BoxSpec,
        secrets: &[(String, String)],
        timeout: Duration,
    ) -> Result<LiveBox, UpError> {
        let error = |reason: &str, detail: String| UpError {
            reason: reason.to_string(),
            detail,
        };
        let mut wire = serde_json::to_vec(spec).map_err(|e| error("spec", e.to_string()))?;
        wire.push(b'\n');
        let mut command = self.command(&["box", "up"]);
        command
            .envs(secrets.iter().map(|(k, v)| (k, v)))
            .stdin(Stdio::piped());
        let mut child = command.spawn().map_err(|e| error("spawn", e.to_string()))?;
        let mut stdin = child.stdin.take();
        let mut stdout = BufReader::new(child.stdout.take().expect("piped stdout"));
        let stderr = tokio::spawn(read_capped(child.stderr.take().expect("piped stderr")));

        let first = tokio::time::timeout(timeout, async {
            // A failed write is answered by pinfold's own line, or by EOF.
            if let Some(pipe) = stdin.as_mut() {
                let _ = pipe.write_all(&wire).await;
            }
            read_line(&mut stdout).await
        })
        .await;
        let Ok(first) = first else {
            drop(stdin);
            terminate(&mut child).await;
            return Err(error("timeout", format!("not ready in {timeout:?}")));
        };
        let Some(bytes) = first else {
            drop(stdin);
            let _ = tokio::time::timeout(TEARDOWN_GRACE, child.wait()).await;
            let detail = match tokio::time::timeout(Duration::from_secs(1), stderr).await {
                Ok(Ok(text)) => text,
                _ => String::new(),
            };
            return Err(error("eof", detail));
        };
        let line = Line::parse(&bytes).unwrap_or_default();
        match line.event.as_deref() {
            Some("ready") => Ok(LiveBox {
                name: spec.name.clone(),
                image_id: line.image_id(),
                child,
                stdin,
                end: tokio::spawn(drain_to_end(stdout)),
            }),
            // Each of these is `up`'s last line; it exits by itself.
            Some("refused") => Err(error(
                line.reason.as_deref().unwrap_or("refused"),
                line.detail(),
            )),
            Some("failed") => Err(error("failed", line.detail())),
            Some("down") => Err(error("down", line.reason.unwrap_or_default())),
            _ => {
                drop(stdin);
                terminate(&mut child).await;
                Err(error("protocol", String::from_utf8_lossy(&bytes).into()))
            }
        }
    }

    /// The caller reads and waits. Not killed on drop: cancelling work in a
    /// box is taking the box down.
    pub fn exec_streaming(
        &self,
        name: &str,
        workdir: Option<&str>,
        argv: &[String],
    ) -> std::io::Result<Child> {
        self.exec_command(name, workdir, argv).spawn()
    }

    /// An exit 3 with pinfold's own message is an absent box; a command in
    /// the box can print the same, so a caller that must know asks `stat`.
    pub async fn exec(
        &self,
        name: &str,
        workdir: Option<&str>,
        argv: &[String],
        timeout: Duration,
    ) -> Result<ExecOutput, ExecError> {
        let out = collect(self.exec_command(name, workdir, argv), timeout)
            .await
            .map_err(|e| match e {
                RunError::Timeout => ExecError::Timeout,
                RunError::Spawn(detail) => ExecError::Spawn(detail),
            })?;
        if out.code == ABSENT_EXIT && out.stderr.starts_with("pinfold box exec: no box named") {
            return Err(ExecError::Absent);
        }
        Ok(ExecOutput {
            code: out.code,
            stdout: out.stdout,
            stderr: out.stderr,
        })
    }

    pub async fn stat(&self, name: &str) -> Result<Option<Stat>, String> {
        let out = self.control(&["box", "stat", name]).await?;
        if out.code == ABSENT_EXIT {
            return Ok(None);
        }
        let line: StatLine = serde_json::from_str(checked(out)?.trim())
            .map_err(|e| format!("pinfold box stat: {e}"))?;
        Ok(Some(Stat {
            oom_kills: line.oom_kills,
        }))
    }

    /// Signals the box's `up` to tear down. An absent box is already down.
    pub async fn down(&self, name: &str) -> Result<(), String> {
        checked(self.control(&["box", "down", name]).await?).map(drop)
    }

    /// `label` is `KEY=VALUE`, or `KEY` for any value.
    pub async fn list(&self, label: &str) -> Result<Vec<Listed>, String> {
        let out = checked(self.control(&["box", "list", "--label", label]).await?)?;
        lines(&out)
            .map(|text| {
                let line: ListLine = serde_json::from_str(text)
                    .map_err(|e| format!("pinfold box list: {e}: {text}"))?;
                Ok(Listed { name: line.name })
            })
            .collect()
    }

    /// Removes every box on the host whose `up` is gone.
    pub async fn prune(&self) -> Result<Vec<String>, String> {
        let out = checked(self.control(&["box", "prune"]).await?)?;
        lines(&out)
            .map(|text| {
                Line::parse(text.as_bytes())
                    .and_then(|line| line.name)
                    .ok_or_else(|| format!("pinfold box prune: {text}"))
            })
            .collect()
    }

    pub async fn image_build(
        &self,
        name: &str,
        containerfile: &Path,
        context: &Path,
        timeout: Duration,
    ) -> Result<Built, BuildError> {
        let containerfile = path_arg(containerfile)?;
        let context = path_arg(context)?;
        let args = [
            "image",
            "build",
            name,
            "--containerfile",
            containerfile,
            "--context",
            context,
        ];
        let out = collect(self.command(&args), timeout)
            .await
            .map_err(|e| BuildError::Other(format!("pinfold image build: {e}")))?;
        let line = lines(&out.stdout)
            .next()
            .and_then(|text| Line::parse(text.as_bytes()));
        match line {
            Some(line) if line.event.as_deref() == Some("built") && out.code == 0 => Ok(Built {
                reference: line
                    .reference
                    .ok_or_else(|| BuildError::Other(format!("no ref: {}", out.stdout)))?,
            }),
            Some(line) if line.event.as_deref() == Some("failed") => {
                Err(BuildError::Failed(line.log.unwrap_or_default()))
            }
            Some(line) if line.event.as_deref() == Some("refused") => Err(BuildError::Refused {
                detail: line.detail(),
                reason: line.reason.unwrap_or_default(),
            }),
            _ => Err(BuildError::Other(format!(
                "pinfold image build exited {}: {}{}",
                out.code, out.stdout, out.stderr
            ))),
        }
    }

    async fn control(&self, args: &[&str]) -> Result<Collected, String> {
        collect(self.command(args), CONTROL_TIMEOUT)
            .await
            .map_err(|e| format!("pinfold {}: {e}", args.join(" ")))
    }
}

impl LiveBox {
    /// Closes stdin and waits for teardown. The reason is `down`'s token:
    /// `stdin-closed`, `signal` (someone ran `box down`), or `exited` (the
    /// box ended on its own). A hung `up` is killed and its box left to
    /// `prune`.
    pub async fn down(mut self, timeout: Duration) -> Result<String, String> {
        drop(self.stdin.take());
        let ended = tokio::time::timeout(timeout, async {
            let end = (&mut self.end).await;
            let _ = self.child.wait().await;
            end
        })
        .await;
        match ended {
            Ok(Ok(end)) => end,
            Ok(Err(e)) => Err(format!("box {}: {e}", self.name)),
            Err(_) => {
                kill_group(&self.child);
                let _ = self.child.wait().await;
                Err(format!("box {} not down in {timeout:?}", self.name))
            }
        }
    }
}

/// After `ready`, the rest of `up`'s stdout, drained so the pipe never
/// fills, down to its last line.
async fn drain_to_end(mut stdout: BufReader<ChildStdout>) -> Result<String, String> {
    let mut end = Err("box up ended without a down line".to_string());
    while let Some(bytes) = read_line(&mut stdout).await {
        let Some(line) = Line::parse(&bytes) else {
            continue;
        };
        match line.event.as_deref() {
            Some("down") => end = Ok(line.reason.unwrap_or_default()),
            Some("failed") => end = Err(format!("box up failed: {}", line.detail())),
            _ => {}
        }
    }
    end
}

/// One line, at most `OUTPUT_CAP` bytes of it; `None` at EOF or on a read
/// error.
async fn read_line(reader: &mut BufReader<ChildStdout>) -> Option<Vec<u8>> {
    let mut buf = Vec::new();
    match reader
        .take(OUTPUT_CAP as u64)
        .read_until(b'\n', &mut buf)
        .await
    {
        Ok(0) | Err(_) => None,
        Ok(_) => Some(buf),
    }
}

/// The first `OUTPUT_CAP` bytes; the rest is read and dropped so the child
/// never blocks on a full pipe.
async fn read_capped(mut reader: impl AsyncRead + Unpin) -> String {
    let mut kept = Vec::new();
    let mut chunk = [0u8; 8192];
    while let Ok(n @ 1..) = reader.read(&mut chunk).await {
        let room = OUTPUT_CAP - kept.len();
        kept.extend_from_slice(&chunk[..n.min(room)]);
    }
    String::from_utf8_lossy(&kept).into_owned()
}

async fn collect(mut command: Command, timeout: Duration) -> Result<Collected, RunError> {
    let mut child = command
        .kill_on_drop(true)
        .spawn()
        .map_err(|e| RunError::Spawn(e.to_string()))?;
    let stdout = child.stdout.take().expect("piped stdout");
    let stderr = child.stderr.take().expect("piped stderr");
    let work = async {
        let (stdout, stderr) = tokio::join!(read_capped(stdout), read_capped(stderr));
        (stdout, stderr, child.wait().await)
    };
    match tokio::time::timeout(timeout, work).await {
        Ok((stdout, stderr, Ok(status))) => Ok(Collected {
            code: exit_code(status),
            stdout,
            stderr,
        }),
        Ok((_, _, Err(e))) => Err(RunError::Spawn(e.to_string())),
        Err(_) => {
            kill_group(&child);
            let _ = child.wait().await;
            Err(RunError::Timeout)
        }
    }
}

fn exit_code(status: ExitStatus) -> i32 {
    status
        .code()
        .or_else(|| status.signal().map(|n| 128 + n))
        .unwrap_or(-1)
}

fn checked(out: Collected) -> Result<String, String> {
    if out.code == 0 {
        Ok(out.stdout)
    } else {
        Err(format!(
            "pinfold exited {}: {}",
            out.code,
            out.stderr.trim()
        ))
    }
}

fn lines(text: &str) -> impl Iterator<Item = &str> {
    text.lines().filter(|line| !line.trim().is_empty())
}

fn path_arg(path: &Path) -> Result<&str, BuildError> {
    path.to_str()
        .ok_or_else(|| BuildError::Other(format!("not UTF-8: {}", path.display())))
}

/// `id()` is `None` once the child is reaped, so a reused pid is never hit.
fn kill_group(child: &Child) {
    if let Some(pid) = child.id() {
        let _ = killpg(Pid::from_raw(pid as i32), Signal::SIGKILL);
    }
}

/// SIGTERM is pinfold's teardown, before or after ready; a hung `up` is
/// killed after the grace.
async fn terminate(child: &mut Child) {
    if let Some(pid) = child.id() {
        let _ = kill(Pid::from_raw(pid as i32), Signal::SIGTERM);
    }
    if tokio::time::timeout(TEARDOWN_GRACE, child.wait())
        .await
        .is_err()
    {
        kill_group(child);
        let _ = child.wait().await;
    }
}
