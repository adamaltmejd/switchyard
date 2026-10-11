# Y-97 cleanup observer repair and test audit — 2026-10-11

The operator reports combined candidate
`636298a7e0651ff3ed2d20a841a762a214581099` passed source review and
focused G2 session 64852 passed 1/0 in 27.38 seconds. G8 session 18559
was still live when this repair was requested. Its log is
`/tmp/switchyard-y97-636-failures-host.log`. No terminal G8 result or
combined full-suite pass is claimed.

The operator's read-only fixture store observation shows the default
control landed, then Y-2's selected gate failure and blocking review were
accepted. Remaining checks and final approval ran. The held final
publication was interrupted and reconciled, and Y-2 actually landed.
Audit 137 started cleanup execution 22 for attempt 3. Audit 139 ended
that execution with outcome removed and a null ticket. Audit 140 released
the same execution. These are operator observations, not a new run here.

The test instead waited for a cleanup-end event naming ticket Y-2.
Read the actual store event boundary: execution.ended carries the end's
optional ticket and the execution's attempt and ID. execution.released
carries the attempt and execution ID. Read Watch.find and the existing
same-project cleanup waits. They search already observed history before
waiting for a later event.

## Test-audit gate

Keep the G8 selected-failure scenario and every assertion. Correct only
its post-landing cleanup observer: find cleanup completion for fixture
attempt 3, then find release of that exact cleanup execution. Attempt 3
comes from the fixture's default attempt, planner attempt and selected
candidate attempt sequence. The execution identity comes from the history
event; this boundary requires completion and release to name the same job.
No hardcoded host execution ID is introduced.

This boundary lets G2's unchanged failed checks and results be observed
through a quiescent restart, and lets G8's remaining carried-block control
run after cleanup releases its reservation. The existing row clauses and
test sabotage comment cover rewriting failures, skipping remaining checks,
ignoring selected identities and consuming publication before teardown.
The held publication and restart remain the hard case. Default repair
alone does not establish acceptance immutability. This observer correction
adds no scenario or duplicate assertion block.

No assertion or production line is removed. Shared helpers, public events,
fixtures, deadlines, migrations and protected guarantee rows are unchanged.
All downstream immutability, override, restart, final block, approval and
actual landing assertions remain. Prior archive records are unchanged.

## Validation

`cargo fmt --check`, `cargo clippy --all-targets --locked -- -D warnings`
and `git diff --check` pass. Clippy compiled the affected e2e target.

No e2e or sabotage execution ran in this candidate. Fresh exact repaired
focused G2/G8, infrastructure and upgrade tests, full host suite, published
review and actual landing gates remain required. The old G8 session must
be terminal before any next host suite; no termination was attempted here.
Ticket scope, dependencies and unparked status remain unchanged.
