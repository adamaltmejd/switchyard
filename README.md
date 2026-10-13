# switchyard

Switchyard (`yard`) turns tickets into landed code with coding agents: one
ticket to one agent in one disposable box, gated, reviewed and landed
through one verified merge queue. [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md)
is the binding spec.

## Requirements

- pinfold 0.1.1 or newer, on the daemon's `PATH`
- a container runtime pinfold supports: rootless podman on Linux, Apple
  container on macOS
- git, on the daemon's `PATH`

## Install

Download the binary for your target from the
[release](https://github.com/adamaltmejd/switchyard/releases), verify it
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

To update pinfold, close its boxes, run `pinfold update`, then
`yard daemon restart`. The daemon reads pinfold's harness versions at
startup. Pinfold downloads the pinned harnesses when a box needs them.

Write `$XDG_CONFIG_HOME/yard/operator.env` (`~/.config/yard/operator.env`
when unset), mode 0600. It holds one credential per connection and the
machine's settings:

    OPENROUTER_API_KEY=...
    OPENCODE_API_KEY=...
    ANTHROPIC_API_KEY=...
    CLAUDE_CODE_OAUTH_TOKEN=...
    YARD_BOX_MEMORY=4G

`YARD_BOX_MEMORY` gives each box a share of memory;
`YARD_ORIGIN_<NAME>` overrides a
connection's origin (`http` for a host service). The connections, logins
and their keys are in [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md).

Claude agents select either `provider = "anthropic"` for direct API access
or `login = true` for subscription access. Direct API review uses
`model = "claude-opus-5-5"` and `effort = "medium"`.

Install and start the daemon as the user service:

    yard daemon install

In a git checkout, scaffold `.yard` and register the project:

    yard init

Edit `.yard/config.toml` (agents, workflows, gates), commit, then import
the branch into canonical:

    yard sync

`yard sync feature/a` imports or consumes that local branch's committed
history. Consume requires a clean checkout of the selected branch. Named
import keeps the project default. `yard sync feature/a --default` selects
it explicitly after its committed `target.ref` names `feature/a`, with no
landing intent. Live attempts keep their admitted targets. Omitted sync uses canonical's symbolic HEAD.
Selection uses an imported canonical head when the checkout branch is absent
or behind, without changing the checkout. Its clean matching checkout rule
applies only to consume.

Check pinfold, the service, the configuration, the image, the credentials
and registered project paths that are gone:

    yard doctor

## The loop

    yard ticket new --title T --body B [--target BRANCH]
    yard status --watch
    yard attempt diff Y-n
    yard attempt approve Y-n --target BRANCH --head SHA [--proof DIGEST]
    yard sync

A ticket freezes its effective target at admission; omission uses the project
default. An inactive open ticket can change its override with revision-bound
`ticket edit --target BRANCH`, or clear it with `--default-target`. Gates,
review, repair and landing use the frozen branch and its trusted inputs.

The rest is in the `yard-operator` skill
(`.agents/skills/yard-operator/SKILL.md`).

`yard mcp` serves trusted operator MCP on stdio through the existing daemon
socket. Tools cover the operations above, projects, tickets, proposals,
attempt inspection and decisions, and bounded history and attention polls.
Pass an absolute registered `project` path. Status omits ticket bodies;
use `yard_ticket_show`, `yard_attempt_tail` and `yard_attempt_stats` for
drilldowns. Cancelling a wait releases the request while work continues.
Workers keep their separate execution-scoped HTTP grants.

Before admission, candidate gate, seat and landing gate overrides may select
`operator_failure = true`. Eligible failures then ask the operator to
`accept` the named attention with text or take `repair`. Acceptance keeps
the failed evidence and runs every remaining selected candidate check
before approval. A failed selected merged gate keeps approval active and
asks the same question on its pinned prepared input. Its answer also names
the original `landing`, `child`, `node`, candidate `base`, ticket `revision`,
target `old` and `merged` head. Acceptance runs the remaining landing gates
before queue landing and compare-and-swap. Infrastructure errors and unknown
ownership cannot be accepted; fresh preparation inherits no answer.
Repair retires approval and binds the exact failed evidence to the next
implementer execution. The original failure stays failed. Its fresh candidate
must pass selected verification and gain approval before landing.

To explicitly accept landing omissions, approve with repeated
`--observe-landing-skipped NODE`. This requests a decision after merge
preparation, including an empty landing selection. Answer its attention with
`attempt accept --accept-skipped NODE` for each requested node, the candidate
bindings, `--landing`, `--base`, `--revision`, `--old`, `--merged` and nonempty
`--text`. Omitted nodes keep no execution or result. Every retained landing
gate still runs. Ordinary omissions record no explicit acceptance.

## Development

    cargo fmt --check
    cargo clippy --all-targets --locked -- -D warnings

The e2e suite (`cargo test -p e2e --locked`) runs on a host with the
container runtime, not inside a box.
CI pins pinfold 0.2.3.

Development binaries use separate state, sockets, projects and runtime names.
They never open the construction backlog. Release preparation does not
authorize publication, installation, daemon replacement or configuration sync.

On the operator's Mac:

    cargo test -p e2e --locked -- --test-threads=4
