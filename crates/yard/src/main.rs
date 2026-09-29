//! `yard`: client and daemon in one binary.

mod agent_env;
mod api;
mod r#box;
mod claude;
mod cli;
mod codex;
mod config;
mod daemon;
mod git;
mod harness;
mod jobs;
mod mcp;
mod pi;
mod store;

fn main() {
    use clap::Parser;
    std::process::exit(cli::main(cli::Cli::parse()));
}
