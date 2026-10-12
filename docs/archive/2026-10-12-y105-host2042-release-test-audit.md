# Y-105 host2042 failed release observation repair

The operator reports candidate 525c2c4e8c1b7f63b2e1f8fd8d6da9e3d51ed3ac
failed actual landing2038/host2042: 72 passed, 3 failed in 555.46 seconds.
The failed positive controls are G3 prepared_input_and_completed_child_survive_host_child_restart,
G3 target_advance_retires_prepared_input_and_requires_fresh_children (shared
helper line 430), and G8 selected_failures_continue_without_rewriting_judgments
(line 2672). Each acceptance returned stale ownership for the same original
landing: current ended/fail, reserved:true; expected released:true. No
acceptance succeeded. This is failed evidence, not a runtime or cache claim.

The operator retains the full supplied tail at
/tmp/switchyard-y105-host2042-failed.log and the authoritative attempt/execution
snapshot at /tmp/switchyard-y105-host2042-terminal.json. Neither path exists
in this worker filesystem. This record preserves the supplied diagnosis and
artifact locations; it does not claim inspection of their full contents or
actual host audit sequence numbers. The retained artifacts remain operator
evidence and must not be replaced by this source diagnosis.

## Source trace

queue::land uses capacity::effect repeatedly for the same parent: preparation,
child admission, and finally failed-gate attention publication. Each phase
reserves capacity; after work, effect drops its host lock and calls release.
release checks inventory and host-lock freedom, then calls executions::reserve
with false. reserve updates the row and emits execution.released in the same
transaction. Later phases reserve the same running parent again. A release
event therefore describes a phase, not permanent terminal ownership freedom.

In the selected failure phase, the parent is reserved. The transaction records
the failed child check, raises candidate/landing attention, then ends the
parent with fail. Only after that transaction and work return does effect drop
the lock and perform checked final release. Reading attention can legitimately
see an ended/fail parent whose reservation remains true during that proof.

Watch::find searches all previously consumed events and returns the first
match. The old positive-control predicates match only event name and original
parent id. They can return a preparation or child-admission release consumed
before the failed attention, without waiting for final release. The restart
watch begins at seq 0 and is likewise exposed for a fresh failed preparation.

prove_predecessors reads the current parent from executions::get, follows its
provenance, lists real project boxes and checks every parent and child.
get reads reserved directly from the selected store column. reserve's row
update and audit share the transaction; Project::read uses the same locked
store connection. Source inspection establishes the fixture's historical
observation defect. It establishes no production release or column-reading
defect. The reported stale row is consistent with the legitimate final-proof
window, but unavailable host chronology cannot rule out another host defect.

## Scoped G2/G3 test-audit

Correct the existing shared restart/target-advance helper's initial and fresh
acceptance prerequisites. Require the named parent's failed execution.ended
after its exact attention, then execution.released with seq greater than that
end. Observe the current store row as ended/fail with reserved=0 before
acceptance. Keep every existing acceptance, stale/no-write, original failed
snapshot, pinned host-Git input, held-child restart, changed-input retirement,
remaining-child and final landing assertion. No new scenario or helper.

The G2/G3 Shown by clauses already require exact acceptance, retained failed
evidence and released predecessor ownership before continuation. The changed
blocks observe external CLI history and the store. Expected fail comes from
the fixture's real false gate; ended and unreserved come from the contract's
completed, checked-release prerequisite. Ordering comes from external audit
seq, anchored to this failure attention and end, never copied release logic.
The harder case is a parent with several releases and reservations, plus a
reopened full history and fresh preparation. Generic cleanup and publication
release tests do not cover repeated phases of that parent.

Sabotage remains in the scenario comment: retaining the ended parent's
reservation must prevent acceptance despite its historical phase releases.
The new row assertion can fail independently if a release event is published
without clearing ownership. No timeout counts as success. The original
pre-teardown consumption, inherited acceptance, failure-rewrite and CAS
sabotage remain. Test lines removed: 4 (old wait lines); binary lines removed:
0. No assertion or guarantee is deleted or weakened. G8's equivalent wait is
corrected in its own test-file commit.

## Limits

Shared prove_predecessors, unknown-inventory refusal, all parent/child ownership
and open-intent guards, exact original preparation and evidence bindings,
capacity, teardown and final CAS are unchanged. No production code, barrier,
sleep, retry, deadline, concurrency, .yard, active runtime store, box or
operational configuration changes. fmt, locked all-targets clippy and diff
checks pass. Worker host e2e and sabotage were not run. A fresh full actual
merged-host gate must establish the positive controls and retained ownership
refusals. Y-104/Y-96/Y-61 retain their full incomplete integration contracts.
