# Y-91 legacy recovery repair

Operator integrity review of check836 found an impossible recovery: an
unbound read-only attempt could start a worker, while publication required
admission and node evidence the old attempt never recorded. Missing report
then stopped it again. The repair chooses refusal, not inferred evidence.

`attempt start` now refuses `unbound_report`, naming the live attempt,
before resolving attention or recording another execution. Scheduler
readiness also excludes an unbound read-only attempt, including after an
edit resolves its old stopped item. The supported recovery is abandon then
fresh admission. G5 ownership checks remain unchanged: this repair cannot
release a retained legacy owner. Classification uses the legacy attempt's
current target workflow configuration. Admitted publication validation is
unchanged. Ended planner proposals still use their existing upgrade path.

The read-only prompt again instructs the planner to propose child tickets
and an edit of its own body, as the current spec requires.

## Source and assertion gate

Read the publication validator, implementer settlement, explicit start,
scheduler readiness, edit steering, reconciliation, ownership release,
execution results and schema migration. The guard acts before another
read-only execution can be recorded. It does not change migration evidence
or promote unbound outputs. Existing report, edit, held and missing-report
assertions are kept verbatim.

Keep the new legacy scenario in `g08_audit_reports.rs`, G8/G6. It runs the
real pre-target binary, holds a planner request, stops the real execution
through the CLI, and proves no execution is running before upgrading. This
is a quiescent upgrade, not a crash scenario. It deliberately avoids a
legacy missing-ownership-proof refusal masking the report refusal.

The new binary exposes unknown admission, binding, result and teardown.
Start refuses with the spec's `unbound_report` and the fixture attempt ID.
An edit and second start cannot continue the old attempt. Abandon then
fresh start publishes the fixture report on revision 2 in the same project.
The old implementation rows remain unchanged through that successful
control, and old admission stays null. CLI and store are outside observers;
the old binary supplies historical state, the fixture supplies IDs, edit
revision and report content, and the spec supplies refusal and unknown
semantics. Sabotage: admit the unbound continuation, schedule its edit,
or fabricate its bindings. Existing ended-planner upgrade assertions test
proposal acceptance, not continuation of a still-live planner. Those
assertions remain unchanged. No assertion is deleted or weakened.

Keep the event identity observation in the existing successful-publication
block, G7/G8. The one event must name the publication's ticket, attempt and
execution, as the Store contract now states. Ticket and attempt IDs come
from the single-ticket fixture. The execution is a required agreement with
the report. Sabotage: omit the event, emit it twice, or misbind its target.
The closed-list assertion checks allowed event names; it does not show
report target identity. This extends the successful control instead of
adding a twin scenario.

Operator findings: P2 unbound recovery is resolved by refusal and fresh
admission; P3 planner instruction is aligned; P3 publication event identity
is covered in the existing block. No outstanding operator decision.

Test assertion lines removed: 0. Binary lines removed: 1, the replaced prompt.

Validation: `cargo fmt --check` and
`cargo clippy --all-targets --locked -- -D warnings` pass. No e2e or sabotage
execution runs in this clone, per the original instruction. Exact78ff's
focused/full host results do not validate this repair. The next committed
head requires fresh exact-head host validation and a published review.
