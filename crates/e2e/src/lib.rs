//! End-to-end tests that drive the `yard` binary from outside.
//!
//! A test builds the real binary, starts a real daemon on a temporary
//! machine, registers a real project, and lets real Pi in real pinfold boxes
//! work against the scripted fake model in `model`. It observes only what a
//! user can: exit codes, `--json` output, the store file, canonical git
//! state, host files, and boxes through `pinfold`.

pub mod model;

pub use model::*;
use serde_json::Value;
use std::io::{BufRead, BufReader, Read};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Output, Stdio};
use std::sync::OnceLock;
use std::sync::mpsc;
use std::time::Duration;

/// The credential behind the model route. It must reach the fake model and
/// never a box.
pub const SECRET: &str = "sk-e2e-5f3a9c-the-operators-key";

/// How long a test waits for an event before it fails. A wait that runs
/// out is a failure, never a pass.
pub const DEADLINE: Duration = Duration::from_secs(600);

/// The built `yard` binary, the test executable's sibling.
pub fn yard() -> &'static Path {
    static BINARY: OnceLock<PathBuf> = OnceLock::new();
    BINARY.get_or_init(|| {
        let status = Command::new(env!("CARGO"))
            .args(["build", "-p", "yard", "--locked"])
            .status()
            .expect("run cargo build -p yard");
        assert!(status.success(), "cargo build -p yard failed");
        let exe = std::env::current_exe().expect("test executable path");
        let binary = exe
            .parent()
            .and_then(Path::parent)
            .expect("target dir")
            .join("yard");
        assert!(binary.is_file(), "{} is missing", binary.display());
        binary
    })
}

/// One pinfold state dir for every test, as on a host. pinfold's daily
/// maintenance pass stamps it, so the pass runs once rather than in every
/// test's first `box list` (about 10 s each on yard-sthlm).
const PINFOLD_STATE: &str = "/tmp/yard-e2e-state";

/// The real `program`, found on the test's own PATH.
pub fn real(program: &str) -> PathBuf {
    let path = std::env::var_os("PATH").unwrap_or_default();
    std::env::split_paths(&path)
        .map(|dir| dir.join(program))
        .find(|candidate| candidate.is_file())
        .unwrap_or_else(|| panic!("{program} on PATH"))
}

/// A temporary machine: its own state, config and wrapper bin directory,
/// one daemon, and the fake model its `operator.env` points at.
pub struct Machine {
    pub root: PathBuf,
    pub state: PathBuf,
    pub config: PathBuf,
    pub cache: PathBuf,
    /// First on the daemon's `PATH`: a test puts a git or pinfold wrapper here.
    pub bin: PathBuf,
    pub model: FakeModel,
    /// Extra variables in the daemon's environment.
    pub env: Vec<(String, String)>,
    daemon: std::sync::Mutex<Option<Child>>,
}

impl Machine {
    pub fn new(
        test: &str,
        script: impl Fn(&ModelRequest) -> Reply + Send + Sync + 'static,
    ) -> Machine {
        // The daemon's socket lives under the state dir; keep it short.
        let root = PathBuf::from("/tmp").join(format!("ye-{}-{test}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let state = root.join("s");
        let config = root.join("c");
        let bin = root.join("bin");
        // One pinfold cache for every test, so the pinned harness is fetched once.
        let cache = PathBuf::from("/tmp/yard-e2e-cache");
        for dir in [
            &state,
            &config.join("yard"),
            &bin,
            &cache,
            Path::new(PINFOLD_STATE),
        ] {
            std::fs::create_dir_all(dir).unwrap();
        }
        let model = FakeModel::start(script);
        let machine = Machine {
            root,
            state,
            config,
            cache,
            bin,
            model,
            env: Vec::new(),
            daemon: std::sync::Mutex::new(None),
        };
        machine.wrapper("pinfold", "");
        machine.write_operator_env(&format!(
            "OPENROUTER_API_KEY={SECRET}\nYARD_ORIGIN_OPENROUTER={}\n",
            machine.model.origin()
        ));
        machine
    }

    pub fn operator_env(&self) -> PathBuf {
        self.config.join("yard/operator.env")
    }

    pub fn write_operator_env(&self, text: &str) {
        use std::os::unix::fs::PermissionsExt;
        let path = self.operator_env();
        std::fs::write(&path, text).unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600)).unwrap();
    }

    /// The operator's Claude token, and the fixture behind its login route.
    pub fn write_claude_env(&self) {
        self.write_operator_env(&format!(
            "CLAUDE_CODE_OAUTH_TOKEN={SECRET}\nYARD_ORIGIN_CLAUDE={}\n",
            self.model.origin()
        ));
    }

    /// The host's Codex login: a `CODEX_HOME` holding a future-dated
    /// `auth.json`, as pinfold's own login-route test uses, and the fixture
    /// behind the route. `CODEX_HOME` is the daemon's own environment, which
    /// it forwards to `box up` for pinfold's host helper; the route origin is
    /// a machine setting.
    pub fn write_codex_env(&mut self, token: &str, account_id: &str) {
        let home = self.root.join("codex-home");
        std::fs::create_dir_all(&home).unwrap();
        let auth = serde_json::json!({
            "OPENAI_API_KEY": null,
            "tokens": {
                "id_token": token,
                "access_token": token,
                "refresh_token": "refresh-e2e",
                "account_id": account_id,
            },
            "last_refresh": "2026-09-29T00:00:00Z",
        });
        std::fs::write(home.join("auth.json"), auth.to_string()).unwrap();
        self.write_operator_env(&format!("YARD_ORIGIN_CODEX={}\n", self.model.origin()));
        self.env
            .push(("CODEX_HOME".into(), home.display().to_string()));
    }

    fn command(&self) -> Command {
        let mut command = Command::new(yard());
        let path = std::env::var_os("PATH").unwrap_or_default();
        let mut dirs = vec![self.bin.clone()];
        dirs.extend(std::env::split_paths(&path));
        command
            .env("XDG_STATE_HOME", &self.state)
            .env("XDG_CONFIG_HOME", &self.config)
            .env("XDG_CACHE_HOME", &self.cache)
            .env("PATH", std::env::join_paths(dirs).unwrap());
        for (name, value) in &self.env {
            command.env(name, value);
        }
        command
    }

    /// Start the daemon and wait for its `serving` line.
    pub fn start(&self) {
        let lines = self.spawn();
        Machine::serving(&lines);
    }

    /// Start the daemon without waiting for it to serve: its stdout lines.
    pub fn spawn(&self) -> mpsc::Receiver<String> {
        assert!(
            self.daemon.lock().unwrap().is_none(),
            "the daemon is already running"
        );
        let mut child = self
            .command()
            .args(["daemon", "run"])
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .expect("spawn yard daemon run");
        let lines = lines(child.stdout.take().unwrap());
        *self.daemon.lock().unwrap() = Some(child);
        lines
    }

    /// Wait for the daemon's `serving` line.
    #[track_caller]
    pub fn serving(lines: &mpsc::Receiver<String>) {
        let line = lines
            .recv_timeout(DEADLINE)
            .expect("the daemon never said it was serving");
        let line: Value = serde_json::from_str(&line).expect("the serving line is JSON");
        assert_eq!(line["event"], "serving", "{line}");
    }

    /// SIGKILL the daemon, or reap it if something already killed it.
    pub fn kill(&self) {
        if let Some(mut child) = self.daemon.lock().unwrap().take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }

    /// Wait for a daemon that something else killed, such as a git wrapper.
    pub fn wait_dead(&self) -> std::process::ExitStatus {
        let mut child = self
            .daemon
            .lock()
            .unwrap()
            .take()
            .expect("a daemon was started");
        child.wait().expect("wait for the daemon")
    }

    pub fn daemon_pid(&self) -> u32 {
        self.daemon
            .lock()
            .unwrap()
            .as_ref()
            .expect("a daemon was started")
            .id()
    }

    /// Run `yard ARGS` in `dir`.
    pub fn yard(&self, dir: &Path, args: &[&str]) -> Output {
        self.command()
            .current_dir(dir)
            .args(args)
            .stdin(Stdio::null())
            .output()
            .expect("run yard")
    }

    /// Run `yard ARGS --json` in `dir`, assert it succeeded and parse it.
    #[track_caller]
    pub fn json(&self, dir: &Path, args: &[&str]) -> Value {
        let out = self.yard(dir, &[args, &["--json"]].concat());
        assert!(
            out.status.success(),
            "yard {args:?} failed: {}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
        serde_json::from_slice(&out.stdout).expect("yard --json prints JSON")
    }

    /// Run `yard ARGS --json` and return its error object; assert it failed.
    #[track_caller]
    pub fn refused(&self, dir: &Path, args: &[&str]) -> Value {
        let out = self.yard(dir, &[args, &["--json"]].concat());
        assert!(
            !out.status.success(),
            "yard {args:?} succeeded: {}",
            String::from_utf8_lossy(&out.stdout)
        );
        let value: Value = serde_json::from_slice(&out.stdout).expect("yard --json prints JSON");
        value["error"].clone()
    }

    /// `pinfold` with this machine's state.
    pub fn pinfold(&self, args: &[&str]) -> Output {
        Command::new("pinfold")
            .env("XDG_STATE_HOME", PINFOLD_STATE)
            .env("XDG_CONFIG_HOME", &self.config)
            .env("XDG_CACHE_HOME", &self.cache)
            .args(args)
            .output()
            .expect("run pinfold")
    }

    /// Boxes carrying `label`, as `pinfold box list` reports them.
    pub fn boxes(&self, label: &str) -> Vec<Value> {
        let out = self.pinfold(&["box", "list", "--label", label]);
        assert!(out.status.success(), "pinfold box list failed");
        String::from_utf8_lossy(&out.stdout)
            .lines()
            .map(|line| serde_json::from_str(line).expect("box list line is JSON"))
            .collect()
    }

    /// Put a wrapper for `program` first on the daemon's PATH. `script` is
    /// the body of a POSIX shell script; `$REAL` is the real program. Every
    /// pinfold wrapper first moves pinfold to the shared state dir and
    /// records the names the daemon builds, for Drop.
    pub fn wrapper(&self, program: &str, script: &str) {
        use std::os::unix::fs::PermissionsExt;
        let path = self.bin.join(program);
        let record = if program == "pinfold" {
            format!(
                "export XDG_STATE_HOME='{PINFOLD_STATE}'\n\
                 if [ \"$1 $2\" = 'image build' ]; then echo \"$3\" >> '{}'; fi\n",
                self.images().display()
            )
        } else {
            String::new()
        };
        let text = format!(
            "#!/bin/sh\nREAL='{}'\n{record}{script}\nexec \"$REAL\" \"$@\"\n",
            real(program).display()
        );
        std::fs::write(&path, text).unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
    }

    fn images(&self) -> PathBuf {
        self.root.join("images")
    }
}

impl Drop for Machine {
    fn drop(&mut self) {
        // Each box's `up` tears it down when the daemon's end of its stdin
        // closes; prune takes any whose `up` died too. Never down by label:
        // that reaches other tests' boxes. Best effort: a Drop during
        // unwinding must not panic.
        self.kill();
        let _ = self.pinfold(&["box", "prune"]);
        // pinfold keeps a name's last images until it is retired. Retire
        // the names this machine's daemon built, now that none of its boxes
        // remain. An image id can carry other tests' names too.
        let names: std::collections::BTreeSet<String> = std::fs::read_to_string(self.images())
            .unwrap_or_default()
            .lines()
            .map(str::to_string)
            .collect();
        for name in names {
            let _ = self.pinfold(&["image", "rm", &name]);
        }
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

/// A registered project: a git checkout with `.yard` committed and synced.
pub struct Project<'a> {
    pub machine: &'a Machine,
    pub path: PathBuf,
}

/// A configuration naming one Pi agent on the fake model.
pub fn config(extra: &str) -> String {
    agent_config(
        r#"harness = "pi"
provider = "openrouter"
model = "fake-model""#,
        extra,
    )
}

fn agent_config(agent: &str, extra: &str) -> String {
    format!(
        r#"max_lanes = 2
approve = "manual"

[target]
ref = "main"
protected_paths = ["AGENTS.md", "CLAUDE.md", ".agents/", ".pi/"]

[agents.worker]
{agent}

[workflows.default]
implementer = "worker"
review = ["correctness"]

[workflows.plan]
access = "read-only"
review = "none"

[review]
max_rounds = 3
blocking = "P1"

[review.seats.correctness]
agent = "worker"
instructions = "Review for correctness."
{extra}"#
    )
}

impl<'a> Project<'a> {
    /// `git init`, one commit, `yard init`, the test's configuration
    /// committed, and `yard sync`.
    pub fn new(machine: &'a Machine, name: &str, config: &str) -> Project<'a> {
        let path = machine.root.join(name);
        std::fs::create_dir_all(&path).unwrap();
        let project = Project { machine, path };
        project.git(&["init", "--quiet", "--initial-branch=main"]);
        project.write("README.md", "# fixture\n");
        project.write("AGENTS.md", "Rule: every file ends with a newline.\n");
        project.git(&["add", "-A"]);
        project.git(&["commit", "--quiet", "-m", "fixture"]);
        machine.json(&project.path, &["init"]);
        project.write(".yard/config.toml", config);
        project.git(&["add", "-A"]);
        project.git(&["commit", "--quiet", "-m", "yard"]);
        project.json(&["sync"]);
        project
    }

    /// The operator's configuration change: consume canonical, commit
    /// `config` in the checkout, and sync it in.
    pub fn reconfigure(&self, config: &str) {
        self.json(&["sync"]);
        self.write(".yard/config.toml", config);
        self.git(&["commit", "--quiet", "-am", "Configure"]);
        self.json(&["sync"]);
    }

    pub fn write(&self, path: &str, text: &str) {
        let path = self.path.join(path);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, text).unwrap();
    }

    /// Host git in the checkout, asserted to succeed.
    #[track_caller]
    pub fn git(&self, args: &[&str]) -> String {
        git(&self.path, args)
    }

    pub fn canonical(&self) -> PathBuf {
        self.path.join(".yard/local/canonical.git")
    }

    /// Canonical's `main`, read by git.
    pub fn canonical_head(&self) -> String {
        git(&self.canonical(), &["rev-parse", "refs/heads/main"])
            .trim()
            .to_string()
    }

    pub fn yard(&self, args: &[&str]) -> Output {
        self.machine.yard(&self.path, args)
    }

    #[track_caller]
    pub fn json(&self, args: &[&str]) -> Value {
        self.machine.json(&self.path, args)
    }

    #[track_caller]
    pub fn refused(&self, args: &[&str]) -> Value {
        self.machine.refused(&self.path, args)
    }

    /// The store, opened read-only.
    pub fn store(&self) -> rusqlite::Connection {
        store(&self.path).expect("open the store read-only")
    }

    /// Rows of one query against the store, as JSON objects.
    pub fn rows(&self, sql: &str) -> Vec<Value> {
        let conn = self.store();
        let mut statement = conn.prepare(sql).expect("prepare");
        let names: Vec<String> = statement
            .column_names()
            .iter()
            .map(|name| name.to_string())
            .collect();
        let rows = statement
            .query_map([], |row| {
                let mut object = serde_json::Map::new();
                for (index, name) in names.iter().enumerate() {
                    let value = match row.get_ref(index)? {
                        rusqlite::types::ValueRef::Null => Value::Null,
                        rusqlite::types::ValueRef::Integer(n) => Value::from(n),
                        rusqlite::types::ValueRef::Real(f) => Value::from(f),
                        rusqlite::types::ValueRef::Text(t) => {
                            Value::from(String::from_utf8_lossy(t).into_owned())
                        }
                        rusqlite::types::ValueRef::Blob(_) => Value::Null,
                    };
                    object.insert(name.clone(), value);
                }
                Ok(Value::Object(object))
            })
            .expect("query");
        rows.map(Result::unwrap).collect()
    }

    /// Follow the audit stream from `since` with `status --history`.
    pub fn watch(&self, since: i64) -> Watch {
        let mut child = self
            .machine
            .command()
            .current_dir(&self.path)
            .args([
                "status",
                "--history",
                "--since",
                &since.to_string(),
                "--json",
            ])
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .expect("spawn yard status --history");
        let (send, heard) = mpsc::channel();
        let stdout = child.stdout.take().unwrap();
        let events = send.clone();
        std::thread::spawn(move || {
            for line in BufReader::new(stdout).lines().map_while(Result::ok) {
                if events.send(Heard::Event(line)).is_err() {
                    break;
                }
            }
        });
        Watch {
            child,
            send,
            heard,
            seen: Vec::new(),
        }
    }
}

/// `yard status --history`: every event since a sequence, in order.
pub struct Watch {
    child: Child,
    /// A FIFO's text joins the events, so one wait covers both.
    send: mpsc::Sender<Heard>,
    heard: mpsc::Receiver<Heard>,
    /// Every event read so far.
    pub seen: Vec<Value>,
}

impl Watch {
    /// Read events until one matches, and return it. Panics on the deadline.
    #[track_caller]
    pub fn until(&mut self, what: &str, matches: impl Fn(&Value) -> bool) -> Value {
        let deadline = std::time::Instant::now() + DEADLINE;
        loop {
            let left = deadline.saturating_duration_since(std::time::Instant::now());
            let line = match self.heard.recv_timeout(left) {
                Ok(Heard::Event(line)) => line,
                Ok(Heard::Said(_)) => continue,
                Err(_) => panic!(
                    "seen:\n{}\nno event {what} within {DEADLINE:?}",
                    self.seen
                        .iter()
                        .map(Value::to_string)
                        .collect::<Vec<_>>()
                        .join("\n")
                ),
            };
            let event: Value = serde_json::from_str(&line).expect("watch prints JSON lines");
            self.seen.push(event.clone());
            if matches(&event) {
                return event;
            }
            // An execution that could not record its own end: no scenario
            // waits past one, so fail now rather than at the deadline.
            assert!(
                !(event["event"] == "attention.raised" && event["data"]["reason"] == "error"),
                "waiting for {what}, the daemon raised an error: {event}"
            );
        }
    }

    /// The first event already read that matches, or the next one to.
    #[track_caller]
    pub fn find(&mut self, what: &str, matches: impl Fn(&Value) -> bool) -> Value {
        match self.seen.iter().find(|event| matches(event)) {
            Some(event) => event.clone(),
            None => self.until(what, matches),
        }
    }

    /// The next attention item raised, whatever its kind: a scenario that
    /// expects one kind fails fast on another.
    #[track_caller]
    pub fn attention(&mut self) -> Value {
        self.until("attention raised", |event| {
            event["event"] == "attention.raised"
        })
    }

    /// The text a wrapper or gate writes to the FIFO `path` once it holds a
    /// call, reading events meanwhile. In every scenario that holds a call,
    /// the call comes before any attention, so an attention first fails now:
    /// the held call will never come, as when a landing fails before it.
    #[track_caller]
    pub fn said(&mut self, path: &Path) -> String {
        let send = self.send.clone();
        let fifo = path.to_owned();
        std::thread::spawn(move || {
            let text = std::fs::read_to_string(&fifo).unwrap_or_default();
            let _ = send.send(Heard::Said(text));
        });
        let deadline = std::time::Instant::now() + DEADLINE;
        loop {
            let left = deadline.saturating_duration_since(std::time::Instant::now());
            match self.heard.recv_timeout(left) {
                Ok(Heard::Said(text)) => return text,
                Ok(Heard::Event(line)) => {
                    let event: Value =
                        serde_json::from_str(&line).expect("watch prints JSON lines");
                    self.seen.push(event.clone());
                    assert!(
                        event["event"] != "attention.raised",
                        "waiting for a call held at {}, the daemon raised: {event}",
                        path.display()
                    );
                }
                Err(_) => panic!("no call held at {} within {DEADLINE:?}", path.display()),
            }
        }
    }

    /// Wait for an event by name, optionally matching `data` fields.
    #[track_caller]
    pub fn event(&mut self, name: &str, data: &[(&str, &str)]) -> Value {
        self.until(name, |event| {
            event["event"] == name
                && data
                    .iter()
                    .all(|(key, value)| event["data"][key].as_str() == Some(value))
        })
    }
}

enum Heard {
    Event(String),
    Said(String),
}

/// Spawn `yard status --watch --json`, with `--since` when given. It exits
/// once an item is open.
pub fn watch_attention(project: &Project<'_>, since: Option<i64>) -> Child {
    let mut command = project.machine.command();
    command
        .current_dir(&project.path)
        .args(["status", "--watch", "--json"]);
    if let Some(since) = since {
        command.arg("--since").arg(since.to_string());
    }
    command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("spawn yard status --watch")
}

/// Wait for a `status --watch` child to exit and parse what it printed:
/// `{"seq", "attention"}`.
#[track_caller]
pub fn watch_answer(mut child: Child) -> Value {
    let stdout = child.stdout.take().expect("piped stdout");
    let (send, heard) = mpsc::channel();
    std::thread::spawn(move || {
        let mut text = String::new();
        let _ = BufReader::new(stdout).read_to_string(&mut text);
        let _ = send.send(text);
    });
    let text = heard.recv_timeout(DEADLINE).unwrap_or_else(|_| {
        let _ = child.kill();
        let _ = child.wait();
        panic!("yard status --watch did not exit within {DEADLINE:?}");
    });
    let status = child.wait().expect("wait for yard status --watch");
    assert!(status.success(), "yard status --watch failed");
    serde_json::from_str(&text).expect("status --watch prints JSON")
}

/// The open items a `status --watch` child printed.
#[track_caller]
pub fn attention_items(child: Child) -> Vec<Value> {
    watch_answer(child)["attention"]
        .as_array()
        .cloned()
        .unwrap_or_default()
}

/// `Watch::said` where no daemon serves a watch.
#[track_caller]
pub fn said(path: &Path) -> String {
    let (send, heard) = mpsc::channel();
    let fifo = path.to_owned();
    std::thread::spawn(move || {
        let _ = send.send(std::fs::read_to_string(&fifo).unwrap_or_default());
    });
    heard
        .recv_timeout(DEADLINE)
        .unwrap_or_else(|_| panic!("no call held at {} within {DEADLINE:?}", path.display()))
}

impl Drop for Watch {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// Host `git -C dir ARGS`, asserted to succeed; its stdout.
#[track_caller]
pub fn git(dir: &Path, args: &[&str]) -> String {
    let out = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args([
            "-c",
            "user.name=Operator",
            "-c",
            "user.email=op@example.invalid",
        ])
        .args(args)
        .output()
        .expect("run git");
    assert!(
        out.status.success(),
        "git {args:?} failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).into_owned()
}

/// The implementer's usual script: one commit, then done.
pub fn commit_file(path: &str, text: &str, message: &str) -> ToolCall {
    bash(&format!(
        "cd /workspace && mkdir -p \"$(dirname {path})\" && printf '%s' '{text}' > {path} && git add -A && git commit -q -m '{message}' && echo committed"
    ))
}

/// Single-quote `text` for a POSIX shell.
pub fn shell_quote(text: &str) -> String {
    format!("'{}'", text.replace('\'', "'\\''"))
}

/// Claude's `Bash` tool, whose input field is `command`.
pub fn claude_bash(command: &str) -> ToolCall {
    tool("Bash", serde_json::json!({ "command": command }))
}

/// Write `files` and commit them in one Claude Bash call.
pub fn claude_files(files: &[(&str, &str)], message: &str) -> ToolCall {
    claude_bash(&files_command(files, message))
}

fn files_command(files: &[(&str, &str)], message: &str) -> String {
    let mut command = String::from("cd /workspace");
    for (path, text) in files {
        command.push_str(&format!(
            " && mkdir -p \"$(dirname {path})\" && printf '%s' {} > {path}",
            shell_quote(text)
        ));
    }
    command.push_str(&format!(
        " && git add -A && git commit -q -m {} && echo committed",
        shell_quote(message)
    ));
    command
}

/// One file, one commit, through Claude's `Bash` tool.
pub fn claude_commit_file(path: &str, text: &str, message: &str) -> ToolCall {
    claude_files(&[(path, text)], message)
}

/// A review publication through Claude's MCP tool name.
pub fn claude_publish(findings: Value) -> ToolCall {
    tool(
        "mcp__yard__yard_publish_review",
        serde_json::json!({ "findings": findings }),
    )
}

/// A configuration naming one Claude login agent on the fake model.
pub fn claude_config(extra: &str) -> String {
    agent_config(
        r#"harness = "claude"
login = true
model = "claude-sonnet-4-5""#,
        extra,
    )
}

/// One execution's script: `calls` when it opens, then done.
pub fn act(request: &ModelRequest, calls: Vec<ToolCall>) -> Reply {
    if request.opens() {
        Reply::Tools(calls)
    } else {
        Reply::Text("done".into())
    }
}

/// A configuration naming one Codex login agent on the fake model.
pub fn codex_config(extra: &str) -> String {
    agent_config(
        r#"harness = "codex"
login = true
model = "gpt-5-codex""#,
        extra,
    )
}

/// Codex's shell tool, whose input field is `cmd`.
pub fn codex_shell(command: &str) -> ToolCall {
    tool("exec_command", serde_json::json!({ "cmd": command }))
}

/// Write `files` and commit them in one Codex shell call.
pub fn codex_files(files: &[(&str, &str)], message: &str) -> ToolCall {
    codex_shell(&files_command(files, message))
}

/// One file, one commit, through Codex's shell tool.
pub fn codex_commit_file(path: &str, text: &str, message: &str) -> ToolCall {
    codex_files(&[(path, text)], message)
}

/// Yard's publish tool as Codex's Responses request advertises it: a tool
/// inside the `mcp__yard` namespace, or at the top level. Returns the
/// namespace when there is one and the bare tool name.
fn codex_publish_tool(request: &ModelRequest) -> Option<(Option<String>, String)> {
    for tool in request.body["tools"].as_array().into_iter().flatten() {
        if let Some(name) = tool["name"].as_str()
            && name.ends_with("yard_publish_review")
        {
            return Some((None, name.to_string()));
        }
        for inner in tool["tools"].as_array().into_iter().flatten() {
            if let (Some(namespace), Some(name)) = (tool["name"].as_str(), inner["name"].as_str())
                && name.ends_with("yard_publish_review")
            {
                return Some((Some(namespace.to_string()), name.to_string()));
            }
        }
    }
    None
}

/// Whether the request is a Codex review seat: Yard's publish tool is
/// offered inside the namespace Codex wraps an MCP server's tools in.
pub fn codex_seat(request: &ModelRequest) -> bool {
    codex_publish_tool(request).is_some()
}

/// A review publication through the tool Codex advertises for Yard's MCP
/// server, carrying the namespace Codex wraps it in.
pub fn codex_publish(request: &ModelRequest, findings: Value) -> ToolCall {
    let (namespace, name) =
        codex_publish_tool(request).unwrap_or((None, "yard_publish_review".to_string()));
    ToolCall {
        name,
        namespace,
        arguments: serde_json::json!({ "findings": findings }),
    }
}

/// A JWT pinfold's codex login route accepts: `exp` in the future and an
/// `https://api.openai.com/auth` claim holding `chatgpt_account_id`.
pub fn codex_jwt(account_id: &str, exp_in_seconds: u64) -> String {
    let exp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
        + exp_in_seconds;
    let header = serde_json::json!({ "alg": "none", "typ": "JWT" });
    let payload = serde_json::json!({
        "exp": exp,
        "https://api.openai.com/auth": { "chatgpt_account_id": account_id },
    });
    format!(
        "{}.{}.{}",
        base64url(header.to_string().as_bytes()),
        base64url(payload.to_string().as_bytes()),
        base64url(b"signature")
    )
}

/// base64url without padding, as a JWT uses.
fn base64url(bytes: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
    let mut out = String::new();
    for chunk in bytes.chunks(3) {
        let b = [
            chunk[0],
            *chunk.get(1).unwrap_or(&0),
            *chunk.get(2).unwrap_or(&0),
        ];
        let n = (u32::from(b[0]) << 16) | (u32::from(b[1]) << 8) | u32::from(b[2]);
        out.push(ALPHABET[(n >> 18) as usize & 63] as char);
        out.push(ALPHABET[(n >> 12) as usize & 63] as char);
        if chunk.len() > 1 {
            out.push(ALPHABET[(n >> 6) as usize & 63] as char);
        }
        if chunk.len() > 2 {
            out.push(ALPHABET[n as usize & 63] as char);
        }
    }
    out
}

/// A review publication with these findings.
pub fn publish(findings: Value) -> ToolCall {
    tool(
        "yard_publish_review",
        serde_json::json!({ "findings": findings }),
    )
}

/// A project's store, opened read-only.
fn store(project: &Path) -> rusqlite::Result<rusqlite::Connection> {
    rusqlite::Connection::open_with_flags(
        project.join(".yard/local/store.sqlite"),
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
    )
}

/// A child's stdout, one line per message, read on its own thread.
fn lines(stdout: std::process::ChildStdout) -> mpsc::Receiver<String> {
    let (send, receive) = mpsc::channel();
    std::thread::spawn(move || {
        for line in BufReader::new(stdout).lines().map_while(Result::ok) {
            if send.send(line).is_err() {
                break;
            }
        }
    });
    receive
}
