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
use std::io::{BufRead, BufReader, Write};
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
    projects: std::sync::Mutex<Vec<PathBuf>>,
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
        for dir in [&state, &config.join("yard"), &bin, &cache] {
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
            projects: std::sync::Mutex::new(Vec::new()),
        };
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
        let stdout = child.stdout.take().unwrap();
        let (send, receive) = mpsc::channel();
        std::thread::spawn(move || {
            for line in BufReader::new(stdout).lines().map_while(Result::ok) {
                let _ = send.send(line);
            }
        });
        *self.daemon.lock().unwrap() = Some(child);
        receive
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
            .env("XDG_STATE_HOME", &self.state)
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
    /// the body of a POSIX shell script; `$REAL` is the real program.
    pub fn wrapper(&self, program: &str, script: &str) {
        use std::os::unix::fs::PermissionsExt;
        let path = self.bin.join(program);
        let text = format!(
            "#!/bin/sh\nREAL='{}'\n{script}\nexec \"$REAL\" \"$@\"\n",
            real(program).display()
        );
        std::fs::write(&path, text).unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
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
        // pinfold keeps a name's last images until it is retired: retire
        // the names of the images this machine's boxes reported, now that
        // none of its boxes remain.
        let projects =
            std::mem::take(&mut *self.projects.lock().unwrap_or_else(|e| e.into_inner()));
        let images: Vec<String> = projects
            .iter()
            .filter_map(|path| {
                rusqlite::Connection::open_with_flags(
                    path.join(".yard/local/store.sqlite"),
                    rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
                )
                .ok()
            })
            .flat_map(|store| {
                store
                    .prepare("SELECT DISTINCT image_id FROM execution WHERE image_id IS NOT NULL")
                    .and_then(|mut query| {
                        query
                            .query_map([], |row| row.get::<_, String>(0))?
                            .collect::<Result<Vec<_>, _>>()
                    })
                    .unwrap_or_default()
            })
            .collect();
        if !images.is_empty()
            && let Ok(out) = Command::new("podman")
                .args([
                    "image",
                    "inspect",
                    "--format",
                    "{{index .Labels \"dev.pinfold.image\"}}",
                ])
                .args(&images)
                .output()
        {
            let names: std::collections::BTreeSet<String> = String::from_utf8_lossy(&out.stdout)
                .lines()
                .filter(|name| !name.is_empty())
                .map(str::to_string)
                .collect();
            for name in names {
                let _ = self.pinfold(&["image", "rm", &name]);
            }
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
    format!(
        r#"max_lanes = 2
approve = "manual"

[target]
ref = "main"
protected_paths = ["AGENTS.md", "CLAUDE.md", ".agents/", ".pi/"]

[agents.worker]
harness = "pi"
provider = "openrouter"
model = "fake-model"

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
        machine.projects.lock().unwrap().push(path.clone());
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
        rusqlite::Connection::open_with_flags(
            self.path.join(".yard/local/store.sqlite"),
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
        )
        .expect("open the store read-only")
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

    /// Follow the audit stream from `since`.
    pub fn watch(&self, since: i64) -> Watch {
        let mut child = self
            .machine
            .command()
            .current_dir(&self.path)
            .args(["status", "--watch", "--since", &since.to_string(), "--json"])
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .expect("spawn yard status --watch");
        let stdout = child.stdout.take().unwrap();
        let (send, receive) = mpsc::channel();
        std::thread::spawn(move || {
            for line in BufReader::new(stdout).lines().map_while(Result::ok) {
                if send.send(line).is_err() {
                    break;
                }
            }
        });
        Watch {
            child,
            lines: receive,
            seen: Vec::new(),
        }
    }
}

/// `yard status --watch`: every event since a sequence, in order.
pub struct Watch {
    child: Child,
    lines: mpsc::Receiver<String>,
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
            let line = match self.lines.recv_timeout(left) {
                Ok(line) => line,
                Err(_) => panic!(
                    "no event {what} within {DEADLINE:?}; seen:\n{}",
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

/// One execution's script: `calls` when it opens, then done.
pub fn act(request: &ModelRequest, calls: Vec<ToolCall>) -> Reply {
    if request.opens() {
        Reply::Tools(calls)
    } else {
        Reply::Text("done".into())
    }
}

/// A review publication with these findings.
pub fn publish(findings: Value) -> ToolCall {
    tool(
        "yard_publish_review",
        serde_json::json!({ "findings": findings }),
    )
}

/// Write `text` to a file in the machine root and return its path.
pub fn scratch(machine: &Machine, name: &str, text: &str) -> PathBuf {
    let path = machine.root.join(name);
    let mut file = std::fs::File::create(&path).unwrap();
    file.write_all(text.as_bytes()).unwrap();
    path
}
