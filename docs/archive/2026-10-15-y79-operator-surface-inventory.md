# Y-79 operator surface inventory and proposed split — 2026-10-15

Inspection baseline: `03b7eb6` (Y-78). No implementation or assertion
changed. No host e2e or construction daemon was run. This inventory proposes
prerequisites for operator review; it does not authorize their admission.

## Current surfaces

`crates/yard/src/cli.rs` is 932 lines. Its task dispatch forwards to the
same socket API as `operator_mcp.rs`. The operator MCP operation table
already advertises every CLI daemon operation. Advertisement is not
parity evidence. The inventory is:

| Operation | Operator MCP/API reading |
| --- | --- |
| Service status | `daemon.status` |
| Registry list, forget | `project.list`, `project.forget` |
| Scaffold/register, diagnose | `init`, `doctor` |
| Committed target import/consume/default selection | `sync` with `branch`, `default` |
| Board | `status` |
| History, attention waits | `events`, `attention` with `since` |
| Create, inspect, list tickets | `ticket.new`, `ticket.show`, `ticket.list` |
| Revision-bound edits and target override/reset | `ticket.edit` |
| Park, unpark, dependency | `ticket.park`, `ticket.unpark`, `ticket.depend` |
| Manual closure | `ticket.done`, `ticket.abandon` |
| Admit/retry, stop | `attempt.start`, `attempt.stop` |
| Exact failed-evidence decisions | `attempt.accept`, `attempt.repair` |
| Exact candidate decisions | `attempt.approve`, `attempt.reject` |
| Withdraw, inspect, diff, transcript | `attempt.abandon`, `attempt.show`, `attempt.diff`, `attempt.tail` |
| Proposal decisions | `proposal.accept`, `proposal.reject` |
| Plan discovery/resolution/validation (MCP already) | `plan.catalog`, `plan.resolve`, `plan.validate` |
| Graph revision and candidate selection (MCP already) | `attempt.revise`, `attempt.select` |
| Standalone execution (MCP already) | `execution.start`, `execution.show`, `execution.stop` |
| Usage (MCP already) | `project.stats`, `attempt.stats` |

Operator tool names are `yard_` plus these methods with dots replaced by
underscores. Required expected identities must survive migration unchanged.
MCP returns `{result}` or `{error}` inside `structuredContent`; the existing
CLI fixture helpers return the unwrapped success or error object. Status
and ticket-list MCP responses deliberately omit ticket bodies.

Keep executable/service entrypoints `daemon run|install|uninstall|status|restart`,
`mcp`, and `version`. Service installation and executable version are local
operations. CLI project discovery (including linked-worktree resolution),
task parsing and text rendering can disappear only after all meaningful
observers use explicit absolute project paths. No argv-to-API compatibility
parser belongs in the permanent fixture.

## Observer migration size and gaps

`crates/e2e/src/lib.rs` has 1,032 lines. `Machine::json`, `Machine::refused`,
`Project::new`, and `Project::watch` use the task CLI. `Watch` spawns the
unbounded CLI history stream; replacing it requires real bounded API/MCP
pages, ordered cursors, cancellation and disconnect behavior, and preserving
its coordination with held crash-point FIFOs. Historical executable
fixtures need explicit treatment so they still test real upgrades.

The guarantee files contain 257 lines calling `.json`, `.refused` or
`.yard`, across about 14,000 lines of scenarios. This count is a search
result, not a scenario or runtime measurement. Direct process observations
also need review. Shipped skills and README contain 48 lines referencing
task/project CLI commands. They include board exits, exact expected
identities and the park/attempt-abandon/ticket-close retirement order.

The G7 watch blocks cited in the ticket are absent from this baseline's
`g07_lands_end_to_end.rs`; it now has 438 lines and no direct `status --watch`
probe. The new observer work must retain both promised clauses explicitly:
idle `since` ignores an older attempt end; `since` beyond an open item waits
for a new item. Merely sending a request, receiving a ping, or checking a
child immediately before mutation does not prove the first sample occurred.
Use a held real external boundary and name what proves that sample. No
sleep, expiry-as-success, retry, or binary-only testing barrier is allowed.
Do not treat this inspection finding as a reproduced product defect.

G6's named-import missing-Dockerfile block remains in
`g11_only_sync_changes_yard.rs`. It checks prose for the fixture commit and
path. `jobs::read_sources` in `jobs/mod.rs` constructs the error without
structured identity. Replace this with a specified missing-input reason,
committed identity and path, observed through MCP/API. Keep the restored
committed-input control and unchanged canonical refs. A nonzero error code
alone cannot replace those observations.

## Proposed sequence

1. Specify and implement the G6 missing committed-input refusal through the
   real operator boundary, adapting the existing named-sync block.
2. Replace shared fixture startup/history and operator call seams with
   structured real MCP/API access. Establish deterministic G7 wait evidence
   in the existing real fixture, retaining both clauses.
3. Migrate remaining scenario operations and observers by method and typed
   arguments. Audit assertions and twins, including historical fixtures,
   stale/control probes, exact decisions, stop/retry and graph edits. Prove
   parity with the full real host landing gate before retiring dispatch.
4. Narrow Y-79 to CLI deletion plus shipped skills, README, AGENTS and
   ARCHITECTURE updates after those prerequisites land. Preserve the frozen
   construction executable and require deliberate operator switchover.

Each implementation updates the binding contract only for what it ships.
All proposals remain parked for Adam's review. No `.yard` edits are needed
in worker candidates. The daemon remains the sole store writer and all
work stays on its existing lifecycle and grants.

Submitted prerequisite proposals: 579 (G6 refusal), 580 (shared seams and
G7 waits), 581 (remaining scenario migration), in that dependency order.
The proposed Y-79 edit keeps final retirement behind all three. Proposal
numbers are not admitted ticket IDs; the operator resolves dependencies
when deciding the split.

Test-audit was read for this inventory. No assertion blocks were changed or
removed, so no assertion gate or whole-suite sweep is claimed. The proposed
migration must record the guarantee, sabotage, external oracle, fixture cost
and twin disposition for every changed block. Source checks cannot prove
runtime parity; host e2e remains the operator's landing gate.

`cargo fmt --check`, `cargo clippy --all-targets --locked -- -D warnings`
and `git diff --check` passed for this documentation-only candidate.
