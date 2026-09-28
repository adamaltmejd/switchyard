//! One process per machine: the unix-socket API, the MCP listener and the
//! scheduler tick over every registered project.

use crate::api::{self, Fail};
use crate::r#box::Pinfold;
use crate::git::Git;
use crate::store::Store;
use hyper::service::service_fn;
use serde_json::{Value, json};
use std::collections::{BTreeMap, HashMap};
use std::fs::File;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::Notify;

pub struct Daemon {
    pub machine: Machine,
    pub git: Git,
    pub pinfold: Pinfold,
    pub projects: Mutex<BTreeMap<PathBuf, Arc<Project>>>,
    pub grants: crate::mcp::Grants,
    pub mcp_port: u16,
    /// Wakes the scheduler.
    pub wake: Notify,
    /// Running executions' stop signals, by execution id.
    /// Keyed by project key and execution id; execution ids are per project.
    pub stops: Mutex<HashMap<(String, i64), Arc<Notify>>>,
    /// Serialises admission, so capacity is counted once per decision.
    pub admission: Mutex<()>,
    /// Image ids built per target head, by project key.
    pub images: Mutex<HashMap<String, (String, String)>>,
    pub image_build: tokio::sync::Mutex<()>,
    /// Held for the daemon's life: one daemon per machine.
    _lock: nix::fcntl::Flock<File>,
}

/// The machine's own settings, from `operator.env`.
pub struct Machine {
    pub max_lanes: Option<u32>,
    pub box_memory: Option<String>,
    /// Every variable `operator.env` set, for connection keys and origins.
    pub vars: BTreeMap<String, String>,
}

impl Machine {
    /// The origin a connection's route leads to.
    pub fn origin(&self, connection: &crate::harness::Connection) -> String {
        let var = format!(
            "YARD_ORIGIN_{}",
            connection.name.to_ascii_uppercase().replace('-', "_")
        );
        self.vars
            .get(&var)
            .cloned()
            .unwrap_or_else(|| connection.origin.to_string())
    }
}

pub struct Project {
    pub root: PathBuf,
    /// Short and stable: names boxes and images host-wide.
    pub key: String,
    pub store: Mutex<Store>,
    /// Every canonical mutation serialises here.
    pub canonical: tokio::sync::Mutex<()>,
    /// Wakes `status --watch` readers after a write.
    pub events: Notify,
    /// The configuration at the target head it was read from.
    pub loaded: Mutex<Option<Arc<crate::jobs::Loaded>>>,
}

impl Project {
    pub fn local(&self) -> PathBuf {
        self.root.join(".yard/local")
    }

    pub fn canonical_dir(&self) -> PathBuf {
        crate::git::canonical_dir(&self.root)
    }

    pub fn attempt_dir(&self, attempt: i64) -> PathBuf {
        self.local().join("attempts").join(attempt.to_string())
    }

    /// One decision transaction; wakes event readers.
    pub fn tx<T>(
        &self,
        f: impl FnOnce(&rusqlite::Transaction) -> Result<T, Fail>,
    ) -> Result<T, Fail> {
        let result = self.store.lock().expect("store lock").tx(f);
        self.events.notify_waiters();
        result
    }

    pub fn read<T>(
        &self,
        f: impl FnOnce(&rusqlite::Connection) -> Result<T, Fail>,
    ) -> Result<T, Fail> {
        f(self.store.lock().expect("store lock").read())
    }
}

pub fn run() -> i32 {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("tokio runtime");
    match runtime.block_on(serve()) {
        Ok(()) => 0,
        Err(error) => {
            eprintln!("yard daemon: {error}");
            1
        }
    }
}

async fn serve() -> Result<(), String> {
    let state = api::state_dir();
    std::fs::create_dir_all(&state).map_err(|error| format!("{}: {error}", state.display()))?;
    let lock = File::create(state.join("daemon.lock")).map_err(|error| error.to_string())?;
    let lock = nix::fcntl::Flock::lock(lock, nix::fcntl::FlockArg::LockExclusiveNonblock)
        .map_err(|_| "another daemon is running on this machine".to_string())?;

    let machine = read_machine()?;
    let path = std::env::var("PATH").unwrap_or_default();
    let mut pinfold_env = Vec::new();
    for var in [
        "PATH",
        "HOME",
        "XDG_STATE_HOME",
        "XDG_CONFIG_HOME",
        "XDG_CACHE_HOME",
        "XDG_RUNTIME_DIR",
        "DBUS_SESSION_BUS_ADDRESS",
    ] {
        if let Ok(value) = std::env::var(var) {
            pinfold_env.push((var.to_string(), value));
        }
    }
    let mcp_listener = tokio::net::TcpListener::bind(("127.0.0.1", 0))
        .await
        .map_err(|error| format!("bind the MCP listener: {error}"))?;
    let mcp_port = mcp_listener
        .local_addr()
        .map_err(|error| error.to_string())?
        .port();
    let daemon = Arc::new(Daemon {
        machine,
        git: Git::new(path),
        pinfold: Pinfold::new(pinfold_env, state.clone()),
        projects: Mutex::new(BTreeMap::new()),
        grants: crate::mcp::Grants::default(),
        mcp_port,
        wake: Notify::new(),
        stops: Mutex::new(HashMap::new()),
        admission: Mutex::new(()),
        images: Mutex::new(HashMap::new()),
        image_build: tokio::sync::Mutex::new(()),
        _lock: lock,
    });

    match daemon.pinfold.artifacts().await {
        Ok(pins) => crate::harness::pin(pins)?,
        Err(error) => return Err(format!("pinfold artifacts: {error}")),
    }

    for root in registry()? {
        if !root.join(crate::config::CONFIG_PATH).is_file() {
            eprintln!(
                "yard daemon: skipping {}: no .yard/config.toml",
                root.display()
            );
            continue;
        }
        register(&daemon, root).await.map_err(|fail| fail.message)?;
    }

    tokio::spawn(crate::mcp::serve(daemon.clone(), mcp_listener));
    tokio::spawn(scheduler(daemon.clone()));

    let socket = api::socket_path();
    let _ = std::fs::remove_file(&socket);
    let listener = tokio::net::UnixListener::bind(&socket)
        .map_err(|error| format!("bind {}: {error}", socket.display()))?;
    let mut terminate = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
        .map_err(|error| error.to_string())?;
    // Reconciliation has committed: say so once, for the service log and
    // for whoever started the daemon.
    let _ = crate::cli::write_line(
        &json!({ "event": "serving", "socket": socket, "pid": std::process::id(), "boundary": api::boundary() })
            .to_string(),
    );
    loop {
        let stream = tokio::select! {
            accepted = listener.accept() => match accepted {
                Ok((stream, _)) => stream,
                Err(error) => {
                    eprintln!("yard daemon: accept: {error}");
                    continue;
                }
            },
            _ = terminate.recv() => return Ok(()),
            _ = tokio::signal::ctrl_c() => return Ok(()),
        };
        let daemon = daemon.clone();
        tokio::spawn(async move {
            let service = service_fn(move |request| {
                let daemon = daemon.clone();
                api::serve(request, move |method, params| async move {
                    handle(&daemon, &method, params).await
                })
            });
            let _ = hyper::server::conn::http1::Builder::new()
                .serve_connection(hyper_util::rt::TokioIo::new(stream), service)
                .await;
        });
    }
}

async fn scheduler(daemon: Arc<Daemon>) {
    loop {
        let projects: Vec<_> = daemon
            .projects
            .lock()
            .expect("projects lock")
            .values()
            .cloned()
            .collect();
        for project in projects {
            if let Err(fail) = crate::jobs::step(&daemon, &project).await {
                eprintln!(
                    "yard daemon: {}: step: {}",
                    project.root.display(),
                    fail.message
                );
            }
        }
        tokio::select! {
            _ = daemon.wake.notified() => {}
            _ = tokio::time::sleep(Duration::from_secs(2)) => {}
        }
    }
}

/// Open a project, reconcile it, and serve it.
async fn register(daemon: &Daemon, root: PathBuf) -> Result<(), Fail> {
    let project = open_project(&root)?;
    crate::jobs::reconcile::project(daemon, &project).await?;
    daemon
        .projects
        .lock()
        .expect("projects lock")
        .insert(root, project);
    Ok(())
}

fn read_machine() -> Result<Machine, String> {
    let path = api::config_dir().join("operator.env");
    let mut vars = BTreeMap::new();
    if let Ok(text) = std::fs::read_to_string(&path) {
        for (number, line) in text.lines().enumerate() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let line = line.strip_prefix("export ").unwrap_or(line);
            let (name, value) = line
                .split_once('=')
                .ok_or_else(|| format!("{}:{}: not NAME=VALUE", path.display(), number + 1))?;
            let value = value.trim();
            let value = value
                .strip_prefix('"')
                .and_then(|value| value.strip_suffix('"'))
                .or_else(|| {
                    value
                        .strip_prefix('\'')
                        .and_then(|value| value.strip_suffix('\''))
                })
                .unwrap_or(value);
            if !crate::config::valid_env_name(name.trim()) {
                return Err(format!("{}:{}: bad name", path.display(), number + 1));
            }
            vars.insert(name.trim().to_string(), value.to_string());
        }
    }
    let max_lanes = vars
        .get("YARD_MAX_LANES")
        .map(|value| {
            value
                .parse()
                .map_err(|_| format!("YARD_MAX_LANES {value:?} is not a number"))
        })
        .transpose()?;
    Ok(Machine {
        max_lanes,
        box_memory: vars.get("YARD_BOX_MEMORY").cloned(),
        vars,
    })
}

fn registry_path() -> PathBuf {
    api::state_dir().join("projects")
}

fn registry() -> Result<Vec<PathBuf>, String> {
    match std::fs::read_to_string(registry_path()) {
        Ok(text) => Ok(text
            .lines()
            .filter(|line| !line.is_empty())
            .map(PathBuf::from)
            .collect()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(Vec::new()),
        Err(error) => Err(error.to_string()),
    }
}

fn write_registry(roots: &[PathBuf]) -> Result<(), String> {
    let text: String = roots
        .iter()
        .map(|root| format!("{}\n", root.display()))
        .collect();
    let path = registry_path();
    let temp = path.with_extension("tmp");
    std::fs::write(&temp, text).map_err(|error| error.to_string())?;
    std::fs::rename(&temp, &path).map_err(|error| error.to_string())
}

fn open_project(root: &Path) -> Result<Arc<Project>, String> {
    use sha2::{Digest, Sha256};
    let key =
        crate::config::hex(&Sha256::digest(root.as_os_str().as_encoded_bytes()))[..10].to_string();
    let store = Store::open(&root.join(".yard/local"))?;
    Ok(Arc::new(Project {
        root: root.to_path_buf(),
        key,
        store: Mutex::new(store),
        canonical: tokio::sync::Mutex::new(()),
        events: Notify::new(),
        loaded: Mutex::new(None),
    }))
}

/// The registered project at `params.project`.
pub fn project(daemon: &Daemon, params: &Value) -> Result<Arc<Project>, Fail> {
    let root = params["project"]
        .as_str()
        .ok_or_else(|| Fail::invalid("the call names no project"))?;
    daemon
        .projects
        .lock()
        .expect("projects lock")
        .get(Path::new(root))
        .cloned()
        .ok_or_else(|| {
            Fail::not_found(format!(
                "{root} is not a registered project; run `yard init` there"
            ))
        })
}

async fn handle(daemon: &Arc<Daemon>, method: &str, params: Value) -> Result<Value, Fail> {
    match method {
        "daemon.status" => Ok(json!({
            "pid": std::process::id(),
            "boundary": api::boundary(),
            "projects": daemon.projects.lock().expect("projects lock").keys().collect::<Vec<_>>(),
        })),
        "init" => init(daemon, &params).await,
        "project.list" => Ok(json!(registry()?)),
        "project.forget" => {
            let path = PathBuf::from(params["path"].as_str().unwrap_or_default());
            let path = std::fs::canonicalize(&path).unwrap_or(path);
            let roots: Vec<_> = registry()?
                .into_iter()
                .filter(|root| *root != path)
                .collect();
            write_registry(&roots)?;
            daemon.projects.lock().expect("projects lock").remove(&path);
            Ok(json!({ "forgotten": path }))
        }
        "events" => events(daemon, &params).await,
        _ => {
            let result = crate::jobs::command(daemon, method, params).await;
            daemon.wake.notify_one();
            result
        }
    }
}

async fn events(daemon: &Daemon, params: &Value) -> Result<Value, Fail> {
    let project = project(daemon, params)?;
    let since = params["since"].as_i64().unwrap_or(0);
    let deadline = tokio::time::Instant::now() + Duration::from_secs(25);
    loop {
        let notified = project.events.notified();
        tokio::pin!(notified);
        notified.as_mut().enable();
        let events = project.read(|conn| crate::store::events_since(conn, since, 500))?;
        if !events.is_empty() {
            return Ok(json!({ "events": events }));
        }
        if tokio::time::timeout_at(deadline, notified).await.is_err() {
            return Ok(json!({ "events": [] }));
        }
    }
}

/// Write the scaffold, create canonical and register the project.
async fn init(daemon: &Arc<Daemon>, params: &Value) -> Result<Value, Fail> {
    let root = PathBuf::from(
        params["path"]
            .as_str()
            .ok_or_else(|| Fail::invalid("init names no path"))?,
    );
    if !root.join(".git").exists() {
        return Err(Fail::refused(format!(
            "{} is not the top of a git checkout",
            root.display()
        )));
    }
    let yard = root.join(".yard");
    std::fs::create_dir_all(&yard).map_err(|error| error.to_string())?;
    let branch = daemon
        .git
        .current_branch(&root)
        .await?
        .unwrap_or_else(|| "main".into());
    let config = yard.join("config.toml");
    if !config.exists() {
        let text =
            crate::config::SCAFFOLD_CONFIG.replace("ref = \"main\"", &format!("ref = {branch:?}"));
        std::fs::write(&config, text).map_err(|error| error.to_string())?;
    }
    let dockerfile = yard.join("Dockerfile");
    if !dockerfile.exists() {
        std::fs::write(&dockerfile, crate::config::SCAFFOLD_DOCKERFILE)
            .map_err(|error| error.to_string())?;
    }
    std::fs::write(yard.join(".gitignore"), "local/\n").map_err(|error| error.to_string())?;
    let dockerignore = root.join(".dockerignore");
    let existing = std::fs::read_to_string(&dockerignore).unwrap_or_default();
    if !existing.lines().any(|line| line.trim() == ".yard/local/") {
        std::fs::write(&dockerignore, format!("{existing}.yard/local/\n"))
            .map_err(|error| error.to_string())?;
    }
    let skill = root.join(".agents/skills/yard-operator/SKILL.md");
    if !skill.exists() {
        std::fs::create_dir_all(skill.parent().expect("skill dir"))
            .map_err(|error| error.to_string())?;
        std::fs::write(&skill, OPERATOR_SKILL).map_err(|error| error.to_string())?;
    }
    let canonical = crate::git::canonical_dir(&root);
    if !canonical.join("HEAD").exists() {
        daemon.git.init_bare(&canonical, &branch).await?;
    }
    let mut roots = registry()?;
    if !roots.contains(&root) {
        roots.push(root.clone());
        write_registry(&roots)?;
    }
    let known = daemon
        .projects
        .lock()
        .expect("projects lock")
        .contains_key(&root);
    if !known {
        register(daemon, root.clone()).await?;
    }
    Ok(json!({ "project": root, "target": branch }))
}

const OPERATOR_SKILL: &str = r#"---
name: yard-operator
description: Drive Switchyard (yard) for this project: file tickets, answer attention, land work.
---

# Operating yard

Yard turns tickets into landed code. `yard status --json` shows `tickets`,
`attempts`, `attention`, `queue`, `running` and `seq`. Answer attention as
it opens; the scheduler runs everything else. Wait with `yard status --watch
--since SEQ`, the `seq` from the last `status --json`; it is the only
history view.

## The loop

`yard ticket new --title T --body B` files a ticket; the body is the plan.
Add `--priority P`, `--depends-on Y-n`, `--parked`, or `--workflow plan`.
A planning worker changes no code: it proposes child tickets and an edit to
its own body. The scheduler admits ready tickets, so a ticket starts on its
own once it is open, unparked and its dependencies are done. `yard attempt
start Y-n` admits one now, and is the same command a `stopped` or `red`
item's exit names.

## Attention

`status --json` lists each open item with its `kind`, `reason`, `ticket`,
`attempt` and `exits`. Answer with an exit the item names.

- `approval` — first read `yard attempt show Y-n` and `yard attempt diff
  Y-n`, then `yard attempt approve Y-n --head SHA`, `yard attempt reject
  Y-n --head SHA --text T`, or `yard attempt abandon Y-n`. Approval binds
  the exact `--head`; a reject sends its notes back as a repair.
- `proposal` — `yard proposal accept ID` or `yard proposal reject ID`,
  where `ID` is the attention id.
- `stopped` — `yard attempt start Y-n`, `yard attempt nudge Y-n --text T`,
  or abandon. On `timeout` and `limit` only nudge and abandon; a nudge's
  text reaches the next execution.
- `red` — the same three, except a landing's item (no `attempt`) exits only
  `yard attempt start Y-n`.

## Judging and syncing

- `yard attempt tail Y-n` prints the live transcript.
- `yard ticket edit Y-n --revision R --body B` needs the revision it read.
  A stale `--revision` changes nothing; re-read `status` and retry.
- `yard sync` imports the checkout's commits, `.yard/` changes included,
  and consumes landed work back into the checkout. A divergence is refused,
  naming both heads.
- `yard doctor` reports pinfold, the configuration, the image and the
  credentials; run it when a connection or the image looks wrong.
"#;

const LABEL: &str = "se.altmejd.yard";

/// The launchd agent or systemd user unit file.
fn service_file() -> PathBuf {
    if cfg!(target_os = "macos") {
        PathBuf::from(std::env::var_os("HOME").unwrap_or_default())
            .join(format!("Library/LaunchAgents/{LABEL}.plist"))
    } else {
        api::config_dir()
            .parent()
            .expect("the config dir has a parent")
            .join("systemd/user/yard.service")
    }
}

pub fn install() -> Result<Value, Fail> {
    let exe = std::env::current_exe().map_err(|error| error.to_string())?;
    let exe = std::fs::canonicalize(&exe).unwrap_or(exe);
    if cfg!(target_os = "macos") {
        let plist = service_file();
        let text = format!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n<plist version=\"1.0\"><dict>\n<key>Label</key><string>{LABEL}</string>\n<key>ProgramArguments</key><array><string>{}</string><string>daemon</string><string>run</string></array>\n<key>RunAtLoad</key><true/>\n<key>KeepAlive</key><true/>\n</dict></plist>\n",
            exe.display()
        );
        std::fs::create_dir_all(plist.parent().expect("agents dir"))
            .map_err(|error| error.to_string())?;
        std::fs::write(&plist, text).map_err(|error| error.to_string())?;
        service(&["launchctl", "load", "-w", &plist.to_string_lossy()])?;
        return Ok(json!({ "service": plist }));
    }
    let unit = service_file();
    let text = format!(
        "[Unit]\nDescription=Switchyard daemon\n\n[Service]\nExecStart={} daemon run\nRestart=on-failure\n\n[Install]\nWantedBy=default.target\n",
        exe.display()
    );
    std::fs::create_dir_all(unit.parent().expect("unit dir")).map_err(|error| error.to_string())?;
    std::fs::write(&unit, text).map_err(|error| error.to_string())?;
    service(&["systemctl", "--user", "daemon-reload"])?;
    service(&["systemctl", "--user", "enable", "--now", "yard.service"])?;
    let user = std::env::var("USER").unwrap_or_default();
    let linger = if loginctl_linger_enabled(&user) {
        "enabled".to_string()
    } else {
        match service(&["loginctl", "enable-linger", &user]) {
            Ok(()) => "enabled".to_string(),
            Err(fail) => format!("not enabled: {}; the daemon stops at logout", fail.message),
        }
    };
    Ok(json!({ "service": unit, "linger": linger }))
}

pub fn uninstall() -> Result<Value, Fail> {
    if cfg!(target_os = "macos") {
        let plist = service_file();
        let _ = service(&["launchctl", "unload", "-w", &plist.to_string_lossy()]);
        let _ = std::fs::remove_file(&plist);
        return Ok(json!({ "removed": plist }));
    }
    let _ = service(&["systemctl", "--user", "disable", "--now", "yard.service"]);
    let unit = service_file();
    let _ = std::fs::remove_file(&unit);
    let _ = service(&["systemctl", "--user", "daemon-reload"]);
    Ok(json!({ "removed": unit }))
}

pub fn restart() -> Result<Value, Fail> {
    if cfg!(target_os = "macos") {
        let uid = nix::unistd::getuid();
        service(&[
            "launchctl",
            "kickstart",
            "-k",
            &format!("gui/{uid}/{LABEL}"),
        ])?;
    } else {
        service(&["systemctl", "--user", "restart", "yard.service"])?;
    }
    Ok(json!({ "restarted": true }))
}

fn loginctl_linger_enabled(user: &str) -> bool {
    let output = std::process::Command::new("loginctl")
        .args(["show-user", user, "-p", "Linger", "--value"])
        .output();
    match output {
        Ok(output) if output.status.success() => {
            String::from_utf8_lossy(&output.stdout).trim() == "yes"
        }
        _ => false,
    }
}

fn service(argv: &[&str]) -> Result<(), Fail> {
    let status = std::process::Command::new(argv[0])
        .args(&argv[1..])
        .status()
        .map_err(|error| Fail::refused(format!("{}: {error}", argv[0])))?;
    if status.success() {
        Ok(())
    } else {
        Err(Fail::refused(format!("{} exited {status}", argv.join(" "))))
    }
}
