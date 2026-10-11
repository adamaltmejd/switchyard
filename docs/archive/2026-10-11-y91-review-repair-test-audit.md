# Y-91 review repair

The correctness review found an unlisted `report.published` event and a
superseded audit classified as `missing_report`. The Store contract now
names the event. The G7 closed-list fixture includes the same event.
Settlement reads the existing pending-edit truth before requiring a report.
The old audit records no output; the existing edit continuation reruns it.

## Test-audit gate

Keep the new held-edit scenario in `g08_audit_reports.rs`, G8/G2. The G8
Shown by clause now names late `superseded_report` refusal, no old result or
missing-report error, and successful publication on the edited revision.
The host model holds before publication while the operator edits the ticket.
The old harness then submits its report, observes refusal, and ends normally.
A fresh harness submits the allowed report on revision 2 in the same attempt.

The store is an outside observer for the execution and publication rows;
CLI history observes completion. Revision 1 and the edit to revision 2 come
from the fixture's ticket creation and one operator edit. Report content is
the scripted model literal. The publication's execution identity must agree
with its row, as required by G8. Model tool results observe the exact
`superseded_report` token. No clock expiry or private seam supplies a pass.

Sabotage: remove the pending-edit exemption from `missing_report`; the old
execution raises failure and the successful continuation cannot complete.
Accept a superseded report instead; the old publication/no-result observation
fails. The allowed second execution controls the refused old submission in
the same project. Existing G8 report tests hold after publication or omit
publication without an edit. Existing G2 tests edit candidate/reviewer inputs.
None exercises this read-only publication race.

Keep G7's closed event-list assertion. Its expected allowlist remains the
Store spec, with the new named event added. No assertion block is removed
or weakened. Operator findings are resolved by these two changes; no
operator decision or follow-up ticket is needed.

Test assertion lines removed: 0. Binary lines removed: 2.

Validation: `cargo fmt --check` and
`cargo clippy --all-targets --locked -- -D warnings` passed. E2e and sabotage
execution remain deferred to the host landing gate as instructed.
