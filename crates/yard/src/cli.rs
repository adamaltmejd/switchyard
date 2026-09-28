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
    /// The project's tickets, attempts and attention; `--watch` follows events.
    Status {
        #[arg(long)]
        watch: bool,
        #[arg(long, default_value_t = 0, requires = "watch")]
        since: i64,
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
        #[arg(long, default_value = "")]
        text: String,
    },
    Reject {
        ticket: String,
        #[arg(long)]
        head: String,
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
fn write_line(text: &str) -> io::Result<()> {
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
        Command::Status { watch: true, since } => {
            let project = find_project(&start)?;
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

/// Follow the audit stream from `since`, one line per event.
async fn watch(socket: &Path, project: &str, since: i64, json: bool) -> Result<(), Fail> {
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

/// The `status` board: open, live, queued and waiting, then `seq`.
fn render_status(status: &Value) -> String {
    let mut out = String::new();

    let tickets = array(status, "tickets");
    if !tickets.is_empty() {
        out.push_str("tickets\n");
        for ticket in tickets {
            let state = if ticket["parked"].as_bool().unwrap_or(false) {
                "parked".to_string()
            } else {
                field(ticket, "state")
            };
            out.push_str(&format!(
                "  {}  {}  {}  {}\n",
                field(ticket, "ticket"),
                field(ticket, "priority"),
                state,
                field(ticket, "title"),
            ));
        }
    }

    let attempts = array(status, "attempts");
    let running = array(status, "running");
    if !attempts.is_empty() {
        out.push_str("attempts\n");
        for attempt in attempts {
            let id = attempt["attempt"].as_i64().unwrap_or_default();
            let head = attempt["head"].as_str().unwrap_or("-");
            let what: Vec<String> = running
                .iter()
                .filter(|execution| execution["attempt"].as_i64() == Some(id))
                .map(describe_execution)
                .collect();
            let what = if what.is_empty() {
                format!("state {}", field(attempt, "state"))
            } else {
                what.join("; ")
            };
            out.push_str(&format!(
                "  {}  head {head}  {what}\n",
                field(attempt, "ticket"),
            ));
        }
    }

    let queue = array(status, "queue");
    if !queue.is_empty() {
        out.push_str("queue\n");
        for item in queue {
            out.push_str(&format!(
                "  head {}  approved by {}  (approval {}, attempt {})\n",
                field(item, "head"),
                field(item, "actor"),
                item["approval"].as_i64().unwrap_or_default(),
                item["attempt"].as_i64().unwrap_or_default(),
            ));
        }
    }

    let attention = array(status, "attention");
    if !attention.is_empty() {
        out.push_str("attention\n");
        for item in attention {
            out.push_str(&format!(
                "  #{}  {}  {}  {}\n",
                item["attention"].as_i64().unwrap_or_default(),
                field(item, "kind"),
                field(item, "reason"),
                field(item, "ticket"),
            ));
            for command in exit_commands(item) {
                out.push_str(&format!("    {command}\n"));
            }
        }
    }

    out.push_str(&format!(
        "seq {}\n",
        status["seq"].as_i64().unwrap_or_default()
    ));
    out
}

/// What a running execution is: its kind, name and reason.
fn describe_execution(execution: &Value) -> String {
    let mut what = field(execution, "kind");
    if let Some(name) = execution["name"].as_str() {
        what = format!("{what} {name}");
    }
    if let Some(reason) = execution["reason"].as_str() {
        what = format!("{what} ({reason})");
    }
    if let Some(round) = execution["round"].as_i64() {
        what = format!("{what} round {round}");
    }
    what
}

/// The exact commands an open attention item's exits name.
fn exit_commands(item: &Value) -> Vec<String> {
    let kind = field(item, "kind");
    let ticket = item["ticket"].as_str().unwrap_or("-");
    let head = item["payload"]["head"].as_str().unwrap_or("-");
    let attention = item["attention"].as_i64().unwrap_or_default();
    item["exits"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(|exit| match (kind.as_str(), exit) {
            ("proposal", "accept") => format!("yard proposal accept {attention}"),
            ("proposal", "reject") => format!("yard proposal reject {attention}"),
            (_, "approve") => format!("yard attempt approve {ticket} --head {head}"),
            (_, "reject") => format!("yard attempt reject {ticket} --head {head} --text T"),
            (_, "start") => format!("yard attempt start {ticket}"),
            (_, "nudge") => format!("yard attempt nudge {ticket} --text T"),
            (_, "abandon") => format!("yard attempt abandon {ticket}"),
            (_, other) => format!("yard attempt {other} {ticket}"),
        })
        .collect()
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
