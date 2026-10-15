# Y72 read-only audit stop and retry

The exact c21a902 whole affected native case passed 1/1 in 60.69 seconds
(65.6608 seconds suite wall time). Evidence remains at
`/home/yard/validation/y72-c21a90-reuse-20261015-evidence`.
Log SHA256:
`73d9183823ba17308cb34af3bb4d1540fb0ccc91b04a7a407fb1be39d2d41b73`.
The parent restored source and left runtime unchanged. This pass covers
changed, no-op, selective and unknown-input reuse, immutable references and
rows, requested medium wire effort, packaged cleanup feedback, stale and
current approval, and actual native landing. Earlier normal failure and
operator-interrupted diagnostic records remain unchanged. This passing
evidence predates the stop repair; the complete repaired case and final
prepared 51-case host gate remain required before landing.

Review1978 identified a lifecycle gap. `attempt_stop` notifies an owned
running worker. The cleanup-audit report branch ended it interrupted but
raised no stopped item. The existing audit scheduler then required a resolved
start decision that the operator could never receive. After successful box
teardown, an operator-stopped audit now uses the existing stop helper with
reason `unchanged`, bound to that execution. Its result has no outputs.
The existing attention start and audit scheduling paths perform the retry.
Ticket revision and semantic plan supersession are checked against the audit
row before raising that decision. Superseded work supplies no eligible result
or new stopped decision. Capacity and unknown ownership handling are unchanged;
an unproved owner retains its reservation.

Test-audit disposition: extend the same G2/G9/G13 hard case at the first
`audit-current` opening. The external host model writes a FIFO signal and
holds its real request. History rejects an unexpected terminal event or
attention before the hold. The operator stops that exact execution while the
request remains held. History must show interrupted, then stopped attention
on that execution with the existing start exit, then checked release.
The read-only store must show ended, unreserved and teardown complete, with
no report, output or evidence reference and no automatically started retry.
The fixture takes the exact attention's start exit. A distinct fresh eligible
report then completes the original cleanup/reuse/approval/landing flow while
the original request remains held. The interrupted row stays immutable.

This protects a blocked attempt and prevents interrupted evidence from
advancing work. Sabotage: omit the stop helper for audits, auto-retry without
the stopped decision, release ownership prematurely, or reference interrupted
work. The stop command and host latch supply the cause, the fixture's exact
execution and attention supply identity, and history/store expose lifecycle
and empty outputs. The same existing allowed report/cleanup/landing path is
the positive retry control. Git, proof and host-model observations from the
previous case remain intact. G9's existing ordinary stop case does not enter
this report-only branch or audit retry scheduler.

Cost: one additional ordinary read-only audit box and one held opening model
request. Its response is never consumed by the stopped worker. The retry
uses the audit work already required by the case. No additional project,
fixture machine, historical build, package build or scenario is introduced.
No assertion block is removed. Test edits replace four lines; the binary
repair replaces one line. The original-result lookup now chooses the first
completed audit rather than the deliberately interrupted predecessor; all
historical rows remain checked for immutability. Only the current-audit
opening count increases, from two to three.

Candidate checks: `cargo fmt --check`,
`cargo clippy --all-targets --locked -- -D warnings`, `cargo build --locked`.
No e2e runs in the worker box. Parent host acceptance remains pending.
