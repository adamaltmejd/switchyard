//! `yard`: client and daemon in one binary.

mod api;
mod r#box;
mod cli;
mod config;
mod daemon;
mod git;
mod jobs;
mod mcp;
mod pi;
mod store;

fn main() {
    use clap::Parser;
    std::process::exit(cli::main(cli::Cli::parse()));
}
