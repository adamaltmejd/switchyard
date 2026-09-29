//! Argument parsing, project discovery, RPC dispatch and rendering.

use crate::api::{self, Fail};
use clap::{Parser, Subcommand};
use serde_json::{Value, json};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(name = "yard", about = "Tickets to landed code with coding agents")]
pub struct Cli {
    /// The project, instead of the one the working directory is in.
    #[arg(long, global = true)]
    project: Option<PathBuf>,
    /// Print the result as JSON.
    #[arg(long, global = true)]
    json: bool,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Scaffold `.yard` in this checkout and register it.
    Init,
    /// Report pinfold, the configuration, the image and the credentials.
    Doctor,
    /// Import the checkout's branch into canonical, or consume canonical into it.
    Sync,
    /// The project's tickets, attempts and attention.
    #[command(group(clap::ArgGroup::new("stream").args(["watch", "history"])))]
    Status {
        /// Return once an attention item is open (with --since, one raised
        /// after it); print the open items and the seq.
        #[arg(long, conflicts_with = "history")]
        watch: bool,
        /// Follow the audit stream from --since, one line per event.
        #[arg(long, conflicts_with = "watch")]
        history: bool,
        #[arg(long, requires = "stream")]
        since: Option<i64>,
    },
    #[command(subcommand)]
    Daemon(DaemonCommand),
    #[command(subcommand)]
    Project(ProjectCommand),
    #[command(subcommand)]
    Ticket(TicketCommand),
    #[command(subcommand)]
    Attempt(AttemptCommand),
    #[command(subcommand)]
    Proposal(ProposalCommand),
    /// Print the version.
    Version,
}

#[derive(Subcommand)]
enum DaemonCommand {
    /// Run the daemon in the foreground.
    Run,
    /// Write and start the user service.
    Install,
    /// Stop and remove the user service.
    Uninstall,
    /// Whether a daemon answers.
    Status,
    /// Restart through the service manager.
    Restart,
}

#[derive(Subcommand, serde::Serialize)]
#[serde(rename_all = "lowercase")]
enum ProjectCommand {
    List,
    /// Remove a project from the registry.
    Forget {
        path: PathBuf,
    },
}

#[derive(Subcommand, serde::Serialize)]
#[serde(rename_all = "lowercase")]
enum TicketCommand {
    New {
        #[arg(long)]
        title: String,
        #[arg(long, default_value = "")]
        body: String,
        #[arg(long, default_value = "P2")]
        priority: String,
        #[arg(long, default_value = "default")]
        workflow: String,
        #[arg(long = "depends-on")]
        depends_on: Vec<String>,
        #[arg(long)]
        parked: bool,
    },
    Show {
        ticket: String,
    },
    Edit {
        ticket: String,
        /// The revision this edit was written against.
        #[arg(long)]
        revision: i64,
        #[arg(long)]
        title: Option<String>,
        #[arg(long)]
        body: Option<String>,
        #[arg(long)]
        priority: Option<String>,
        #[arg(long)]
        workflow: Option<String>,
    },
    Park {
        ticket: String,
    },
    Unpark {
        ticket: String,
    },
    /// Make TICKET depend on ON.
    Depend {
        ticket: String,
        on: String,
    },
    List,
    /// Close a ticket by hand.
    Done {
        ticket: String,
        #[arg(long)]
        reason: String,
    },
    Abandon {
        ticket: String,
        #[arg(long, default_value = "")]
        reason: String,
    },
}

#[derive(Subcommand, serde::Serialize)]
#[serde(rename_all = "lowercase")]
enum AttemptCommand {
    /// Admit a ticket now, or retry its stopped or red item.
    Start {
        ticket: String,
        #[arg(long)]
        attention: Option<i64>,
    },
    Stop {
        ticket: String,
    },
    Approve {
        ticket: String,
        /// The candidate head this approval binds.
        #[arg(long)]
        head: String,
        /// The candidate proof digest this approval binds, when it has one.
        #[arg(long)]
        proof: Option<String>,
        #[arg(long, default_value = "")]
        text: String,
    },
    Reject {
        ticket: String,
        #[arg(long)]
        head: String,
        /// The candidate proof digest this reject answers, when it has one.
        #[arg(long)]
        proof: Option<String>,
        #[arg(long)]
        text: String,
    },
    Abandon {
        ticket: String,
        #[arg(long, default_value = "")]
        reason: String,
    },
    Show {
        ticket: String,
    },
    Diff {
        ticket: String,
    },
    Tail {
        ticket: String,
    },
}

#[derive(Subcommand, serde::Serialize)]
#[serde(rename_all = "lowercase")]
enum ProposalCommand {
    Accept {
        attention: i64,
        #[arg(long, default_value = "")]
        text: String,
    },
    Reject {
        attention: i64,
        #[arg(long, default_value = "")]
        text: String,
    },
}

pub fn main(cli: Cli) -> i32 {
    if let Command::Daemon(DaemonCommand::Run) = cli.command {
        return crate::daemon::run();
    }
    if let Command::Version = cli.command {
        let version = env!("CARGO_PKG_VERSION");
        let text = if cli.json {
            json!({ "version": version, "boundary": api::boundary() }).to_string()
        } else {
            format!("yard {version}")
        };
        let _ = write_line(&text);
        return 0;
    }
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("tokio runtime");
    let json = cli.json;
    match runtime.block_on(dispatch(&cli)) {
        Ok(Some(result)) => {
            let text = if json {
                result.to_string()
            } else if matches!(&cli.command, Command::Status { .. }) {
                render_status(&result)
            } else {
                render(&result)
            };
            let _ = write_line(&text);
            0
        }
        Ok(None) => 0,
        Err(fail) => {
            if json {
                let _ = write_line(&json!({ "error": fail.to_json() }).to_string());
            } else {
                eprintln!("yard: {}: {}", fail.code, fail.message);
            }
            1
        }
    }
}

/// One line to stdout. A closed pipe ends the command; the caller stops.
pub(crate) fn write_line(text: &str) -> io::Result<()> {
    let mut stdout = io::stdout().lock();
    writeln!(stdout, "{text}")?;
    stdout.flush()
}

async fn dispatch(cli: &Cli) -> Result<Option<Value>, Fail> {
    let socket = api::socket_path();
    let start = match &cli.project {
        Some(path) => path.clone(),
        None => std::env::current_dir().map_err(|error| Fail::invalid(error.to_string()))?,
    };
    let (method, mut params) = match &cli.command {
        Command::Daemon(DaemonCommand::Status) => ("daemon.status".into(), json!({})),
        Command::Daemon(DaemonCommand::Install) => return crate::daemon::install().await.map(Some),
        Command::Daemon(DaemonCommand::Uninstall) => return crate::daemon::uninstall().map(Some),
        Command::Daemon(DaemonCommand::Restart) => return crate::daemon::restart().await.map(Some),
        Command::Daemon(DaemonCommand::Run) | Command::Version => unreachable!("handled above"),
        Command::Init => {
            let dir = std::fs::canonicalize(&start)
                .map_err(|error| Fail::invalid(format!("{}: {error}", start.display())))?;
            ("init".into(), json!({ "path": dir }))
        }
        Command::Project(command) => {
            let (method, params) = method("project", command);
            return api::call(&socket, &method, params).await.map(Some);
        }
        Command::Status {
            history: true,
            since,
            ..
        } => {
            let project = resolve_project(&socket, &start).await?;
            return history(&socket, &project, since.unwrap_or(0), cli.json)
                .await
                .map(|()| None);
        }
        Command::Status {
            watch: true, since, ..
        } => {
            let project = resolve_project(&socket, &start).await?;
            return watch(&socket, &project, *since, cli.json)
                .await
                .map(|()| None);
        }
        Command::Doctor => ("doctor".into(), json!({})),
        Command::Sync => ("sync".into(), json!({})),
        Command::Status { .. } => ("status".into(), json!({})),
        Command::Ticket(command) => method("ticket", command),
        Command::Attempt(command) => method("attempt", command),
        Command::Proposal(command) => method("proposal", command),
    };
    if !matches!(&cli.command, Command::Init | Command::Daemon(_)) {
        params["project"] = json!(resolve_project(&socket, &start).await?);
    }
    api::call(&socket, &method, params).await.map(Some)
}

/// A subcommand as its RPC: `group.variant`, with the variant's fields as
/// the params.
fn method(group: &str, command: &impl serde::Serialize) -> (String, Value) {
    match serde_json::to_value(command).expect("a command serializes") {
        Value::Object(variant) => {
            let (name, params) = variant.into_iter().next().expect("one variant");
            (format!("{group}.{name}"), params)
        }
        name => (
            format!("{group}.{}", name.as_str().unwrap_or_default()),
            json!({}),
        ),
    }
}

/// `find_project`, except that a linked worktree resolves to its main
/// checkout when that is a registered project, whether or not the worktree
/// checks out `.yard/config.toml`.
async fn resolve_project(socket: &Path, start: &Path) -> Result<String, Fail> {
    let dir = std::fs::canonicalize(start)
        .map_err(|error| Fail::invalid(format!("{}: {error}", start.display())))?;
    let git = crate::git::Git::new(std::env::var("PATH").unwrap_or_default());
    if let Some(main) = git.main_checkout(&dir).await {
        let listed = api::call(socket, "project.list", json!({})).await?;
        let registered = listed.as_array().is_some_and(|roots| {
            roots
                .iter()
                .any(|root| root.as_str().map(Path::new) == Some(main.as_path()))
        });
        if registered {
            return Ok(main.to_string_lossy().into_owned());
        }
    }
    find_project(&dir)
}

/// The nearest directory at or above `start` holding `.yard/config.toml`.
fn find_project(start: &Path) -> Result<String, Fail> {
    let start = std::fs::canonicalize(start)
        .map_err(|error| Fail::invalid(format!("{}: {error}", start.display())))?;
    let mut dir = start.as_path();
    loop {
        if dir.join(crate::config::CONFIG_PATH).is_file() {
            return Ok(dir.to_string_lossy().into_owned());
        }
        dir = dir.parent().ok_or_else(|| {
            Fail::not_found(format!(
                "no .yard/config.toml at or above {}; run `yard init`",
                start.display()
            ))
        })?;
    }
}

/// Return once an attention item is open, or with `since` once one raised
/// after it is, or the board is idle; print every open item and the seq to pass next.
async fn watch(socket: &Path, project: &str, since: Option<i64>, json: bool) -> Result<(), Fail> {
    loop {
        let result = api::call(
            socket,
            "attention",
            json!({ "project": project, "since": since }),
        )
        .await?;
        let items = result["attention"].as_array().cloned().unwrap_or_default();
        if !items.is_empty() || result["idle"].as_bool() == Some(true) {
            let seq = result["seq"].as_i64().unwrap_or_default();
            let text = if json {
                json!({ "seq": seq, "attention": items }).to_string()
            } else {
                format!("{}\nseq {seq}", render_attention(&items).trim_end())
            };
            let _ = write_line(&text);
            return Ok(());
        }
    }
}

/// Follow the audit stream from `since`, one line per event.
async fn history(socket: &Path, project: &str, since: i64, json: bool) -> Result<(), Fail> {
    let mut seq = since;
    let width = terminal_width();
    loop {
        let result = api::call(
            socket,
            "events",
            json!({ "project": project, "since": seq }),
        )
        .await?;
        for event in result["events"].as_array().into_iter().flatten() {
            seq = event["seq"].as_i64().unwrap_or(seq);
            let line = if json {
                event.to_string()
            } else {
                watch_line(event, width)
            };
            if write_line(&line).is_err() {
                return Ok(());
            }
        }
    }
}

/// `seq event ticket`, then the first line of `text`, on one line.
fn watch_line(event: &Value, width: usize) -> String {
    let seq = event["seq"].as_i64().unwrap_or_default();
    let name = event["event"].as_str().unwrap_or_default();
    let ticket = event["ticket"].as_str().unwrap_or("-");
    let first = event["text"]
        .as_str()
        .unwrap_or_default()
        .lines()
        .next()
        .unwrap_or_default();
    let line = if first.is_empty() {
        format!("{seq} {name} {ticket}")
    } else {
        format!("{seq} {name} {ticket} {first}")
    };
    truncate(&line, width)
}

fn render(value: &Value) -> String {
    match value {
        Value::String(text) => text.clone(),
        other => serde_json::to_string_pretty(other).unwrap_or_default(),
    }
}

/// The `status` board, in the order a ticket moves: `parked`, `waiting`
/// (idle open tickets with their reason), `running` (live attempts), and
/// `landing` (the queue), then the attention items and `seq`. Each open
/// ticket is in exactly one section; an empty section is not printed.
fn render_status(status: &Value) -> String {
    let width = terminal_width();
    let title_line = |indent: usize, id: &Value, out: &mut String| {
        let title = array(status, "tickets")
            .iter()
            .find(|ticket| ticket["ticket"] == *id)
            .map(|ticket| field(ticket, "title"))
            .unwrap_or_default();
        if !title.is_empty() {
            out.push_str(&truncate(&format!("{:indent$}{title}", ""), width));
            out.push('\n');
        }
    };
    let attempts = array(status, "attempts");
    let queue = array(status, "queue");
    let queued = |ticket: &Value| queue.iter().any(|item| item["ticket"] == ticket["ticket"]);
    let attempt_of = |ticket: &Value| {
        attempts
            .iter()
            .find(|attempt| attempt["ticket"] == ticket["ticket"])
    };
    let open: Vec<&Value> = array(status, "tickets")
        .iter()
        .filter(|ticket| !queued(ticket))
        .collect();
    let is_parked = |ticket: &Value| ticket["parked"].as_bool().unwrap_or(false);
    let pick = |keep: &dyn Fn(&Value) -> bool| -> Vec<&Value> {
        open.iter().copied().filter(|ticket| keep(ticket)).collect()
    };
    let sections = [
        ("parked", pick(&|t| attempt_of(t).is_none() && is_parked(t))),
        (
            "waiting",
            pick(&|t| attempt_of(t).is_none() && !is_parked(t)),
        ),
        ("running", pick(&|t| attempt_of(t).is_some())),
    ];

    let row = |ticket: &Value| {
        let (phase, clocks) = match attempt_of(ticket) {
            Some(attempt) => attempt_line(status, attempt),
            None => (idle_reason(ticket), String::new()),
        };
        let edges = if phase.starts_with("waiting on") {
            String::new()
        } else {
            depends_on(ticket)
        };
        (
            ticket["ticket"].clone(),
            field(ticket, "ticket"),
            field(ticket, "priority"),
            phase,
            clocks,
            edges,
        )
    };
    let rows: Vec<Vec<_>> = sections
        .iter()
        .map(|(_, tickets)| tickets.iter().map(|ticket| row(ticket)).collect())
        .collect();
    let phase_width = rows
        .iter()
        .flatten()
        .map(|line| line.3.chars().count())
        .max()
        .unwrap_or(0);
    let clock_width = rows
        .iter()
        .flatten()
        .map(|line| line.4.chars().count())
        .max()
        .unwrap_or(0);
    let mut out = String::new();
    for ((title, _), lines) in sections.iter().zip(rows) {
        if lines.is_empty() {
            continue;
        }
        out.push_str(title);
        out.push('\n');
        for (id, name, priority, phase, clocks, edges) in lines {
            let line = format!(
                "  {name:<5} {priority}  {phase:<phase_width$}  {clocks:<clock_width$}  {edges}"
            );
            out.push_str(line.trim_end());
            out.push('\n');
            title_line(2 + 5 + 1 + priority.chars().count() + 2, &id, &mut out);
        }
    }

    if !queue.is_empty() {
        out.push_str("landing\n");
        for item in queue {
            let head: String = field(item, "head").chars().take(7).collect();
            let line = format!(
                "  {:<5} {head}  {}  {}",
                field(item, "ticket"),
                landing_phase(status, item["attempt"].as_i64()).unwrap_or_default(),
                depends_on(item),
            );
            out.push_str(line.trim_end());
            out.push('\n');
            title_line(
                2 + 5 + 1 + head.chars().count() + 2,
                &item["ticket"],
                &mut out,
            );
        }
    }

    let attention = array(status, "attention");
    if !attention.is_empty() {
        out.push_str("attention\n");
        out.push_str(&render_attention(attention));
    }

    out.push_str(&format!(
        "seq {}",
        status["seq"].as_i64().unwrap_or_default()
    ));
    out
}

fn names(value: &Value) -> Vec<&str> {
    value
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .collect()
}

/// A ticket's unfinished dependencies, as the board prints them.
fn depends_on(ticket: &Value) -> String {
    let edges = names(&ticket["waiting_on"]);
    if edges.is_empty() {
        String::new()
    } else {
        format!("[depends on: {}]", edges.join(", "))
    }
}

/// The running executions of one attempt.
fn running_of(status: &Value, attempt: Option<i64>) -> Vec<&Value> {
    array(status, "running")
        .iter()
        .filter(|execution| execution["attempt"].as_i64() == attempt)
        .collect()
}

/// The landing a queued attempt is in: the landing, or the gate it runs.
fn landing_phase(status: &Value, attempt: Option<i64>) -> Option<String> {
    let mine = running_of(status, attempt);
    if !mine.iter().any(|execution| execution["kind"] == "landing") {
        return None;
    }
    let gate = mine.iter().find(|execution| execution["kind"] == "gate");
    Some(match gate {
        Some(gate) => format!("landing: gate {:?}", field(gate, "name")),
        None => "landing".to_string(),
    })
}

/// Why an open ticket with no live attempt is not running, when Yard's own
/// state says it.
fn idle_reason(ticket: &Value) -> String {
    let waiting = names(&ticket["waiting_on"]);
    if ticket["parked"].as_bool().unwrap_or(false) {
        "parked".into()
    } else if !waiting.is_empty() {
        format!("waiting on {}", waiting.join(", "))
    } else if ticket["no_lane"].as_bool().unwrap_or(false) {
        "no lane".into()
    } else {
        "ready".into()
    }
}

/// What a live attempt is doing now, and its two clocks: total work and the
/// time since its worker last produced output.
fn attempt_line(status: &Value, attempt: &Value) -> (String, String) {
    let id = attempt["attempt"].as_i64();
    let mine = running_of(status, id);
    // The newest running execution is the innermost.
    let newest = mine
        .iter()
        .max_by_key(|execution| execution["execution"].as_i64());
    let item = array(status, "attention")
        .iter()
        .find(|item| item["attempt"].as_i64() == id);

    let phase = if let Some(execution) = newest {
        let name = field(execution, "name");
        match execution["kind"].as_str().unwrap_or_default() {
            "implementation" if execution["reason"].as_str() == Some("repair") => {
                "repair".to_string()
            }
            "implementation" => "implementing".to_string(),
            "gate" => format!("gate {name:?}"),
            "review" => format!(
                "review {name:?} round {}",
                execution["round"].as_i64().unwrap_or_default()
            ),
            other => other.to_string(),
        }
    } else if let Some(item) = item {
        format!("{}: {}", field(item, "kind"), field(item, "reason"))
    } else if let Some(reason) = attempt["next"]["reason"].as_str() {
        if attempt["lane"].as_bool().unwrap_or(false) {
            format!("{reason} next")
        } else {
            format!("{reason}: no lane")
        }
    } else {
        String::new()
    };

    let mut clocks = Vec::new();
    if let Some(spent) = attempt["work_ms"].as_i64() {
        clocks.push(format!("run {}", duration(spent)));
    }
    if let Some(quiet) = mine
        .iter()
        .find_map(|execution| execution["quiet_ms"].as_i64())
    {
        clocks.push(format!("quiet {}", duration(quiet)));
    }
    (phase, clocks.join("  "))
}

/// A span as its two largest whole units: `40s`, `6m`, `1h05m`.
fn duration(ms: i64) -> String {
    let secs = (ms / 1000).max(0);
    match secs {
        0..=59 => format!("{secs}s"),
        60..=3599 => format!("{}m", secs / 60),
        _ => format!("{}h{:02}m", secs / 3600, secs % 3600 / 60),
    }
}

/// The open attention items, as the `status` board lists them.
fn render_attention(items: &[Value]) -> String {
    let mut out = String::new();
    for item in items {
        let exits = names(&item["exits"]);
        out.push_str(&format!(
            "  #{}  {}  {}  {}  {}\n",
            item["attention"].as_i64().unwrap_or_default(),
            field(item, "kind"),
            field(item, "reason"),
            field(item, "ticket"),
            exits.join(" | "),
        ));
        if let Some(bindings) = exit_bindings(item) {
            out.push_str(&format!("       {bindings}\n"));
        }
    }
    out
}

/// What an approval's exits bind and a person cannot retype: the candidate
/// head, and its proof digest when one must be named. Other items bind
/// nothing, so they render as one line.
fn exit_bindings(item: &Value) -> Option<String> {
    if item["kind"].as_str() != Some("approval") {
        return None;
    }
    let head = item["payload"]["head"].as_str().unwrap_or("-");
    let digest = item["payload"]["proof"].as_str().unwrap_or_default();
    let mut line = format!("--head {head}");
    if crate::jobs::proof::required(digest) {
        line.push_str(&format!(" --proof {digest}"));
    }
    Some(line)
}

fn array<'a>(value: &'a Value, key: &str) -> &'a [Value] {
    value[key].as_array().map(Vec::as_slice).unwrap_or_default()
}

fn field(value: &Value, key: &str) -> String {
    match &value[key] {
        Value::String(text) => text.clone(),
        Value::Null => "-".to_string(),
        other => other.to_string(),
    }
}

/// The terminal's width in columns, or 80 when stdout is not a terminal.
fn terminal_width() -> usize {
    let mut size = nix::libc::winsize {
        ws_row: 0,
        ws_col: 0,
        ws_xpixel: 0,
        ws_ypixel: 0,
    };
    // SAFETY: TIOCGWINSZ fills the winsize pointer on success.
    let result =
        unsafe { nix::libc::ioctl(nix::libc::STDOUT_FILENO, nix::libc::TIOCGWINSZ, &mut size) };
    if result == 0 && size.ws_col > 0 {
        return size.ws_col as usize;
    }
    std::env::var("COLUMNS")
        .ok()
        .and_then(|columns| columns.parse().ok())
        .filter(|columns| *columns > 0)
        .unwrap_or(80)
}

fn truncate(line: &str, width: usize) -> String {
    if line.chars().count() <= width {
        return line.to_string();
    }
    let mut out: String = line.chars().take(width.saturating_sub(1)).collect();
    out.push('…');
    out
}
