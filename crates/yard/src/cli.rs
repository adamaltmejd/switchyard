//! Service, executable and operator-MCP entrypoints.

use crate::api::{self, Fail};
use clap::{Parser, Subcommand};
use serde_json::{Value, json};
use std::io::{self, Write};

#[derive(Parser)]
#[command(name = "yard", about = "Tickets to landed code with coding agents")]
pub struct Cli {
    /// Print service or version results as JSON.
    #[arg(long, global = true)]
    json: bool,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    #[command(subcommand)]
    Daemon(DaemonCommand),
    /// Serve trusted operator MCP on stdin and stdout.
    Mcp,
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
    if matches!(cli.command, Command::Mcp) {
        let result = runtime.block_on(crate::operator_mcp::serve());
        // Stdin's blocking reader may still await bytes after stdout failed.
        // Accepted forwards have drained before the runtime shuts down.
        runtime.shutdown_background();
        return match result {
            Ok(()) => 0,
            Err(error) => {
                eprintln!("yard mcp: {error}");
                1
            }
        };
    }
    match runtime.block_on(dispatch(&cli)) {
        Ok(result) => {
            let text = if cli.json {
                result.to_string()
            } else {
                serde_json::to_string_pretty(&result).expect("service result")
            };
            let _ = write_line(&text);
            0
        }
        Err(fail) => {
            if cli.json {
                let _ = write_line(&json!({ "error": fail.to_json() }).to_string());
            } else {
                eprintln!("yard: {}: {}", fail.code, fail.message);
            }
            1
        }
    }
}

/// A closed stdout pipe ends the caller.
pub(crate) fn write_line(text: &str) -> io::Result<()> {
    let mut stdout = io::stdout().lock();
    writeln!(stdout, "{text}")?;
    stdout.flush()
}

async fn dispatch(cli: &Cli) -> Result<Value, Fail> {
    match &cli.command {
        Command::Daemon(DaemonCommand::Status) => {
            api::call(&api::socket_path(), "daemon.status", json!({})).await
        }
        Command::Daemon(DaemonCommand::Install) => crate::daemon::install().await,
        Command::Daemon(DaemonCommand::Uninstall) => crate::daemon::uninstall(),
        Command::Daemon(DaemonCommand::Restart) => crate::daemon::restart().await,
        Command::Daemon(DaemonCommand::Run) | Command::Version | Command::Mcp => {
            unreachable!("handled above")
        }
    }
}
