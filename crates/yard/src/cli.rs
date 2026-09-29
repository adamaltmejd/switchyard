//! Argument parsing, one call per command, rendering.

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
    Status {
        /// Return as soon as an attention item is open; print the open items.
        #[arg(long, conflicts_with = "history")]
        watch: bool,
        /// Follow the audit stream from --since, one line per event.
        #[arg(long, conflicts_with = "watch")]
        history: bool,
        #[arg(long, requires = "history", conflicts_with = "watch")]
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
    Nudge {
        ticket: String,
        #[arg(long)]
        text: String,
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
        Command::Daemon(DaemonCommand::Install) => return crate::daemon::install().map(Some),
        Command::Daemon(DaemonCommand::Uninstall) => return crate::daemon::uninstall().map(Some),
        Command::Daemon(DaemonCommand::Restart) => return crate::daemon::restart().map(Some),
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
            let project = find_project(&start)?;
            return history(&socket, &project, since.unwrap_or(0), cli.json)
                .await
                .map(|()| None);
        }
        Command::Status { watch: true, .. } => {
            let project = find_project(&start)?;
            return watch(&socket, &project, cli.json).await.map(|()| None);
        }
        Command::Doctor => ("doctor".into(), json!({})),
        Command::Sync => ("sync".into(), json!({})),
        Command::Status { .. } => ("status".into(), json!({})),
        Command::Ticket(command) => method("ticket", command),
        Command::Attempt(command) => method("attempt", command),
        Command::Proposal(command) => method("proposal", command),
    };
    if !matches!(&cli.command, Command::Init | Command::Daemon(_)) {
        params["project"] = json!(find_project(&start)?);
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

/// Return as soon as at least one attention item is open; print the open
/// items.
async fn watch(socket: &Path, project: &str, json: bool) -> Result<(), Fail> {
    loop {
        let result = api::call(socket, "attention", json!({ "project": project })).await?;
        let items = result["attention"].as_array().cloned().unwrap_or_default();
        if !items.is_empty() {
            let text = if json {
                Value::Array(items).to_string()
            } else {
                render_attention(&items).trim_end().to_string()
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

/// The `status` board: one line per open ticket saying what it is doing now,
/// then the attention items and `seq`.
fn render_status(status: &Value) -> String {
    let attempts = array(status, "attempts");
    let mut tickets: Vec<&Value> = array(status, "tickets").iter().collect();
    let live = |ticket: &Value| {
        attempts
            .iter()
            .any(|attempt| attempt["ticket"] == ticket["ticket"])
    };
    tickets.sort_by_key(|ticket| !live(ticket));

    let lines: Vec<(String, String, String, String)> = tickets
        .iter()
        .map(|ticket| {
            let name = field(ticket, "ticket");
            let attempt = attempts
                .iter()
                .find(|attempt| attempt["ticket"] == ticket["ticket"]);
            let (phase, clocks) = match attempt {
                Some(attempt) => attempt_line(status, attempt),
                None => (idle_reason(ticket), String::new()),
            };
            (name, field(ticket, "priority"), phase, clocks)
        })
        .collect();
    let width = lines
        .iter()
        .map(|line| line.2.chars().count())
        .max()
        .unwrap_or(0);
    let mut out = String::new();
    for (name, priority, phase, clocks) in lines {
        let line = format!("{name:<5} {priority}  {phase:<width$}  {clocks}");
        out.push_str(line.trim_end());
        out.push('\n');
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

/// Why an open ticket with no live attempt is not running.
fn idle_reason(ticket: &Value) -> String {
    let mut parts = Vec::new();
    if let Some(outcome) = ticket["outcome"].as_str() {
        parts.push(outcome.to_string());
    }
    let waiting: Vec<&str> = ticket["depends_on"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .collect();
    if ticket["parked"].as_bool().unwrap_or(false) {
        parts.push("parked".into());
    } else if !waiting.is_empty() {
        parts.push(format!("waiting on {}", waiting.join(", ")));
    } else if ticket["ready"].as_bool().unwrap_or(false) {
        parts.push("no free lane".into());
    } else {
        parts.push("waiting on a proposal".into());
    }
    parts.join("; ")
}

/// What a live attempt is doing now, and its clocks.
fn attempt_line(status: &Value, attempt: &Value) -> (String, String) {
    let id = attempt["attempt"].as_i64();
    let now = status["now_ms"].as_i64().unwrap_or_default();
    let queue = array(status, "queue");
    // The newest running execution is the innermost: a landing's gate over
    // the landing.
    let mine: Vec<&Value> = array(status, "running")
        .iter()
        .filter(|execution| execution["attempt"].as_i64() == id)
        .collect();
    let newest = mine
        .iter()
        .max_by_key(|execution| execution["execution"].as_i64());
    let item = array(status, "attention")
        .iter()
        .find(|item| item["attempt"].as_i64() == id);
    let position = queue.iter().position(|item| item["attempt"].as_i64() == id);

    let (phase, step) = if let Some(execution) = newest {
        let name = field(execution, "name");
        let phase = match execution["kind"].as_str().unwrap_or_default() {
            "implementation" if execution["reason"].as_str() == Some("repair") => "repair".into(),
            "implementation" => "implementing".to_string(),
            "gate" if mine.iter().any(|other| other["kind"] == "landing") => {
                format!("landing: gate {name}")
            }
            "gate" => format!("gates: {name}"),
            "review" => {
                let round = execution["round"].as_i64().unwrap_or_default();
                match attempt["max_rounds"].as_i64() {
                    Some(max) => format!("review {name}, round {round} of {max}"),
                    None => format!("review {name}, round {round}"),
                }
            }
            "landing" => "landing".to_string(),
            other => other.to_string(),
        };
        (phase, Some(*execution))
    } else if let Some(item) = item {
        let phase = match item["kind"].as_str().unwrap_or_default() {
            "approval" => "awaiting approval".to_string(),
            kind => format!("{kind}: {}", field(item, "reason")),
        };
        (phase, None)
    } else if let Some(position) = position {
        let phase = match position
            .checked_sub(1)
            .map(|before| field(&queue[before], "ticket"))
        {
            Some(ahead) => format!(
                "approved, {} in queue behind {ahead}",
                ordinal(position + 1)
            ),
            None => "approved, next in queue".to_string(),
        };
        (phase, None)
    } else if attempt["lane"].as_bool() == Some(false) {
        ("waiting for a lane".to_string(), None)
    } else {
        ("between steps".to_string(), None)
    };

    let mut clocks = Vec::new();
    if let Some(execution) = step {
        if let Some(started) = execution["started_ms"].as_i64() {
            clocks.push(duration(now - started));
        }
        if let Some(quiet) = execution["quiet_ms"].as_i64() {
            clocks.push(format!("quiet {}", duration(quiet)));
        }
    }
    if let Some(spent) = attempt["work_ms"]
        .as_i64()
        .filter(|_| attempt["lane"].as_bool() == Some(true))
    {
        let mut total = format!("total {}", duration(spent));
        if let Some(limit) = attempt["work_limit_ms"].as_i64() {
            total.push_str(&format!(" of {}", duration(limit)));
        }
        clocks.push(total);
    }
    (phase, clocks.join("   "))
}

fn ordinal(n: usize) -> String {
    let suffix = match (n % 10, n % 100) {
        (_, 11..=13) => "th",
        (1, _) => "st",
        (2, _) => "nd",
        (3, _) => "rd",
        _ => "th",
    };
    format!("{n}{suffix}")
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
        let exits: Vec<&str> = item["exits"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .collect();
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
