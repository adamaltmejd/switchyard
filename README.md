# switchyard

Switchyard (`yard`) turns tickets into landed code with isolated coding agents,
gates, review and one verified merge queue. [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md)
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

Start the trusted operator MCP adapter on the host:

    yard mcp

Connect an MCP client to its stdin/stdout and use `tools/list` for schemas.
Operator tools take an absolute registered `project` path.
`yard_init` instead takes `{"path":"/path/to/checkout"}`; it scaffolds
`.yard` and registers the checkout. Edit `.yard/config.toml`, commit, then
call `yard_sync` with `{"project":"/path/to/checkout"}`.

`yard_sync` with `branch: "feature/a"` imports or consumes that local
branch's committed history. Consume requires a clean checkout of the
selected branch. Named import keeps the default; `default: true` selects
it after its committed `target.ref` names that branch, with no landing
intent. Live attempts keep their admitted targets. Omitted branch uses
canonical's symbolic HEAD. Selection can use an already imported canonical
head without consuming or switching the checkout. `yard_doctor` reports
pinfold, configuration, image and credential availability.

## The loop

1. Call `yard_ticket_new` with project, title, body and optional target.
2. Poll `yard_attention`. An empty result is drained only when idle is true.
3. Read `yard_attempt_show` and `yard_attempt_diff` for the named ticket.
4. Call `yard_attempt_approve` with the exact target, head and other
   identities named by the item, including proof and choice when present.
5. Wait for `landing.recorded` through `yard_events`, then call `yard_sync`.

A ticket freezes its effective target at admission. An inactive ticket's
override can change through revision-bound `yard_ticket_edit`; default_target
clears it. Status omits bodies: use `yard_ticket_show` to read them.
`yard_attempt_tail` and `yard_attempt_stats` provide explicit drilldowns.

MCP returns `structuredContent.result` or `structuredContent.error`;
stale errors retain expected/current identity data. History pages contain
at most 500 events; advance since only through consumed events. Empty pages
keep the cursor. Attention polls wait at most 25 seconds; with since they
wait past older open items or old attempt ends. Repeat an empty non-idle
poll at the same cursor. Cancelling observation leaves work running.
Workers retain separate execution-scoped HTTP grants.

Attempt and project statistics expose local model-trial source choices,
eligible samples, frozen plan provenance and partial usage. Picker selection
measures usefulness to that picker, not correctness.

The `yard-operator` skill (`.agents/skills/yard-operator/SKILL.md`) lists
operations; `yard-drive` supplies board judgment. The executable retains
only service commands, MCP startup and version. Construction remains on
its frozen installed executable until a deliberate operator switchover.

Use `yard_plan_catalog` with `project` to inspect target-bound presets,
profiles, provider/model settings, selected skills and publication tools.
New projects ship ordinary, small, skill audit, writable cleanup, failed-check
classifier, concurrent review/check join, independent selector/integration,
and implementation/review model-trial presets. Existing project templates
are updated only by the operator through `yard_sync`.

For example, resolve `{"project":"/path/to/project","preset":"ordinary",
"overrides":{"implementer":{"profile":"trial-1"}}}` with
`yard_plan_validate`, then supply the same selection and `ticket` to
`yard_attempt_start`. A profile replaces role capabilities and preserves its
inputs and routes. Define model and effort in normal named agents; profile
selection grants no additional tools.

`implementation-model-trial` has five isolated implementers, the same
reviewer bound separately to each candidate, then a read-only candidate
selector. `review-model-trial` has one implementer, two reviewer profiles,
an exact join and finding grouping for remediation. Trial agents initially
use the same model; the operator chooses their settings before sync.
Existing capacity limits apply. Replace the scaffold's minimal Git gate
with project checks before use. `test-audit` selects the installed
`.agents/skills/test-audit/SKILL.md` and reads source without write access.

Before admission, set `operator_failure = true` on selected candidate gate,
seat or landing gate overrides. Answer eligible failures with
`yard_attempt_accept` or `yard_attempt_repair`, naming the exact identities in
the attention and nonempty text. Failures stay failed; remaining selected
checks still run. Infrastructure errors and unknown ownership cannot be
accepted. See [Attention](docs/ARCHITECTURE.md#attention) for decision bindings.

To explicitly accept landing omissions, approve with the
`observe_landing_skipped` node array. This requests a decision after merge
preparation, including an empty landing selection. Answer its attention with
`yard_attempt_accept` with `accept_skipped`, the candidate
bindings, `landing`, `base`, `revision`, `old`, `merged` and nonempty
`text`. Omitted nodes keep no execution or result. Every retained landing
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
