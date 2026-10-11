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
