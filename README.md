# switchyard

Switchyard (`yard`) turns tickets into landed code with coding agents: one
ticket to one agent in one disposable box, gated, reviewed and landed
through one verified merge queue. [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md)
is the binding spec.

## Requirements

- pinfold 0.0.9 or newer, on the daemon's `PATH`
- a container runtime pinfold supports: rootless podman on Linux, Apple
  container on macOS
- git, on the daemon's `PATH`

## Install

Download the binary for your target from the
[release](https://github.com/adamaltmejd/switchyard-v3/releases), verify it
against the release's `SHA256SUMS`, and put it on `PATH`.

| Target | Role |
|---|---|
| `aarch64-apple-darwin` | Mac |
| `aarch64-unknown-linux-musl`, `x86_64-unknown-linux-musl` | Linux hosts |

Or build it and put it on `PATH`:

    cargo build --release --locked -p yard
    install -m 0755 target/release/yard ~/.local/bin/yard

## Setup

Build pinfold's default image once per host; the scaffold's Dockerfile
starts `FROM pinfold/profile-default:latest`:

    pinfold build --profile default

Write `$XDG_CONFIG_HOME/yard/operator.env` (`~/.config/yard/operator.env`
when unset), mode 0600. It holds one credential per connection and the
machine's settings:

    OPENROUTER_API_KEY=...
    OPENCODE_API_KEY=...
    CLAUDE_CODE_OAUTH_TOKEN=...
    YARD_BOX_MEMORY=4g

`YARD_BOX_MEMORY` gives each box a share of memory;
`YARD_ORIGIN_<NAME>` overrides a
connection's origin (`http` for a host service). The connections, logins
and their keys are in [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md).

Install and start the daemon as the user service:

    yard daemon install

In a git checkout, scaffold `.yard` and register the project:

    yard init

Edit `.yard/config.toml` (agents, workflows, gates), commit, then import
the branch into canonical:

    yard sync

Check pinfold, the service, the configuration, the image, the credentials
and registered project paths that are gone:

    yard doctor

## The loop

    yard ticket new --title T --body B
    yard status --watch
    yard attempt diff Y-n
    yard attempt approve Y-n --head SHA [--proof DIGEST]
    yard sync

The rest is in the `yard-operator` skill
(`.agents/skills/yard-operator/SKILL.md`).

## Development

    cargo fmt --check
    cargo clippy --all-targets --locked -- -D warnings

The e2e suite (`cargo test -p e2e --locked`) runs on a host with the
container runtime, not inside a box.
