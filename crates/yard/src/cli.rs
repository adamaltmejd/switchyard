//! Argument parsing, one call per command, rendering.

use crate::api::{self, Fail};
use clap::{Parser, Subcommand};
use serde_json::{Value, json};
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
        if cli.json {
            println!(
                "{}",
                json!({ "version": version, "boundary": api::boundary() })
            );
        } else {
            println!("yard {version}");
        }
        return 0;
    }
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("tokio runtime");
    let json = cli.json;
    match runtime.block_on(dispatch(cli)) {
        Ok(Some(result)) => {
            if json {
                println!("{result}");
            } else {
                render(&result);
            }
            0
        }
        Ok(None) => 0,
        Err(fail) => {
            if json {
                println!("{}", json!({ "error": fail.to_json() }));
            } else {
                eprintln!("yard: {}: {}", fail.code, fail.message);
            }
            1
        }
    }
}

async fn dispatch(cli: Cli) -> Result<Option<Value>, Fail> {
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
    if !matches!(cli.command, Command::Init | Command::Daemon(_)) {
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
    loop {
        let result = api::call(
            socket,
            "events",
            json!({ "project": project, "since": seq }),
        )
        .await?;
        for event in result["events"].as_array().into_iter().flatten() {
            seq = event["seq"].as_i64().unwrap_or(seq);
            if json {
                println!("{event}");
            } else {
                println!(
                    "{} {} {} {}",
                    event["seq"],
                    event["event"].as_str().unwrap_or_default(),
                    event["ticket"].as_str().unwrap_or("-"),
                    event["text"].as_str().unwrap_or_default()
                );
            }
        }
        use std::io::Write;
        if std::io::stdout().flush().is_err() {
            return Ok(());
        }
    }
}

fn render(value: &Value) {
    match value {
        Value::String(text) => println!("{text}"),
        other => println!(
            "{}",
            serde_json::to_string_pretty(other).unwrap_or_default()
        ),
    }
}
