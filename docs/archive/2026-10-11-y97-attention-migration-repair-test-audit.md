# Y-97 attention migration repair and test audit — 2026-10-11

The operator reported a real focused-host failure on candidate
`de85397dde1aa84f56f5e1e289195523deffa10c`. After admission and an eligible
failed candidate gate, the daemon could not persist `candidate` attention:
the store still constrained kinds to approval, proposal, stopped and red.
The operator reported session 53489 still live, with log
`/tmp/switchyard-y97-de85-stale-host.log`. No terminal result or full-suite
pass is claimed. No host suite was started in this repair attempt.

## Repair

Store::open owns migration, after taking the daemon's writer lock and
enabling foreign keys. Each forward migration and its user_version change
commit in one transaction. The new migration rebuilds only attention,
adding exactly candidate to its closed kind constraint. It copies all
columns with explicit IDs, preserves state and resolution constraints and
outgoing foreign keys, and recreates the open-item index. No table has an
incoming foreign key to attention; audit names attention IDs as values.
Foreign-key enforcement stays enabled. Neither the initial migration nor
operational store files were edited.

## Test-audit gate

Keep the extension to G15's existing real historical-binary upgrade
scenario, under G8's upgrade clause. It adds a second proposal, rejected
through the old CLI, and compares the open and resolved proposal records
before migration and after two reopens. It compares every limit attention
field, allowing only the existing specified target payload backfill, and
compares the original audit prefix. A read-only foreign_key_check observes
referential integrity. Existing proposal acceptance and approve-at-limit
landing remain the successful controls in the same project.

Sabotage: lose IDs, payloads, resolutions or audit history in the table
rebuild. The test comment names it. Previous upgrade assertions established
usable exits but did not establish complete persisted-row preservation.
The resolved proposal and second reopen exercise historical decisions and
migration idempotence. Expected rows come from the external read-only store
observer before upgrade; equality is the promised preservation contract.
The target is the fixture's main branch. Foreign-key validity comes from
SQLite's independent integrity observer. No fixture writes the schema.

The original G2/G8 admission, stale identity, failure preservation,
held-publication, restart and teardown controls are retained. Those real
candidate scenarios also fail if the new attention kind cannot persist.
No assertions were removed and no runtime or test-only barrier was added.
This is a gate review of the changed blocks, not a whole-suite sweep.
Test lines removed: 0. Binary lines removed: 0.

## Validation

`cargo fmt --check` and `cargo clippy --all-targets --locked -- -D warnings`
pass. Clippy compiles the extended upgrade scenario. `git diff --check`
passes.

Host e2e and sabotage execution are unrun in this candidate, as instructed.
Fresh focused tests on the repaired candidate, the full host suite,
published review and actual landing gates remain required. The operator
must end the old live run before another host suite begins. The initial
candidate's source gate does not establish runtime correctness after the
reported store failure.

## Operator focused-host results and observer repair

The operator reported two fresh focused failures on repaired candidate
`c1f5e85428ed9ce41edcc99e574e6278a9c9f22f`. Both reached real candidate
attention and stale-decision controls, confirming progress past the original
store constraint failure, without establishing a focused or full-suite pass.

- G2 session 32982: terminal exit 101, 0 passed / 1 failed, 15.84 seconds;
  log `/tmp/switchyard-y97-c1f5-stale-host.log`. Line 778 read the repair
  success as repaired[accepted], but trusted MCP returned
  {result: {accepted: false, attention: 2, evidence: ...}}.
- G8 session 87521: terminal exit 101, 0 passed / 1 failed, 23.04 seconds;
  log `/tmp/switchyard-y97-c1f5-failures-host.log`. Line 1606 read the gate
  acceptance success as accepted[accepted], but trusted MCP returned
  {result: {accepted: true, ...}}. The later carried-block acceptance had
  the same incorrect observer near line 1777.

Read Operator.call and operator_mcp::tool_result at the actual boundary.
The helper returns structuredContent, which intentionally retains the
operator result/error envelope. Correct only those three success observers
to read result.accepted. Keep the CLI acceptance observer near line 1663
direct, and leave the helper and public contracts unchanged.

Test-audit disposition: retain all three assertions in their existing G2/G8
blocks. Their expected false/true values come from the fixture's explicit
repair/accept choices and the specified operator decisions. Their sabotage,
hard cases and absence of twins remain as recorded in the original Y-97
audit and scenario comments: lose exact identity binding, rewrite failures,
skip remaining checks or consume publication before teardown. This repair
corrects the external observer; it adds no scenario and weakens no assertion.
All stale named identities, no-write comparisons, positive repair/default
controls, held publications, restart, unchanged judgments and landing
observations remain. Test lines removed: 0. Binary lines changed: 0.

No fresh host or sabotage pass is claimed. Both reported sessions are
terminal; the full repaired suite has not run. Fresh exact repaired focused
and full host tests, published review and actual landing gates remain
required. No e2e execution occurred in this candidate.

After the observer correction, `cargo fmt --check`,
`cargo clippy --all-targets --locked -- -D warnings` and `git diff --check`
pass. These source checks do not claim a host runtime pass.
