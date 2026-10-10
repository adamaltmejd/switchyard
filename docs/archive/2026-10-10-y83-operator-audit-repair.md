# Y-83 operator audit repair, 2026-10-10

The operator inspected candidate 400f717 and reported four control gaps.
These were static source/fixture findings, not host reproductions. This
repair preserves existing assertion blocks and guarantee clauses. The
worker does not run the host e2e suite or open construction state.

## Controls

Stop now ends an unreserved implementation on its existing execution row
and raises stopped:unchanged. Reservation checks the current row inside
its transaction, so it cannot launch from the row stop already ended.
Reserved runs retain their existing stop handling. No waiting lifecycle or
owner is added.

The failed landing-child path now reserves the existing landing execution
around returned, including target.bundle creation. The conflict path calls
returned inside the Git preparation reservation and does not acquire a
nested slot. Source audit of queue.rs: merge-tree, commit-tree and the
landing ref are in preparation; target update and its normal ancestry
proof are in the final phase; both repair bundle paths are now covered.
Configuration, policy and reconciliation reads observe canonical state;
they do not write a landing effect. The existing G3/G10 assertions remain.

A forward migration records whether a row's creating binary promised the
new inherited execution lock. Existing rows are marked without that
contract, including rows from the first Y-83 candidate. Reservations are
backfilled for running rows, ended open intents, non-cleanup failed/error,
interrupted, timeout, refused or abandoned rows, red/conflict landing
returns, and gate/review judgments whose old teardown was unchecked.
A missing new lock never releases them. Ended red/conflict returns may
have used the old unowned bundle path and retain that ambiguity too.

Supported legacy proofs are explicit: an open landing intent with its
existing free landing lock; a recorded host gate with its existing free
attempt/landing lock; or a terminal boxed gate/review pass/fail with its
recorded box handle and pinfold proving box absence. These states passed
the old pre-box Git phase. All other reserved legacy rows keep capacity
and red:ownership. Operator start cannot bypass proof for an open intent.
There is no host-quiescence attestation or CLI override. An ambiguous
upgrade stops for host investigation and the final Y80 rehearsal. This
repair does not claim that manually releasing a known fixture child
supplies identity evidence absent from the old store.

## Focused test-audit gate

Keep stopping_a_capacity_waiter_prevents_its_turn_until_start (G4/G9).
The host model holds one real worker at limit one. The other ticket's wait
event names the execution the operator stops. Its terminal unchanged row
and stopped:unchanged attention establish the stop contract. After the
holder reaches approval, the host model has received no waiter request.
A deliberate start then reaches approval with exactly one waiter turn.
Sabotage: only notify the absent in-memory handle, or reserve using a stale
row after stop. G9's existing stop scenario acts inside a live box, so it
cannot expose this wait boundary. Expected tokens come from the binding
stop contract; requests and the hold come from the external host fixture.

Keep an_upgrade_retains_a_legacy_pre_box_child_without_an_ownership_proof
(G5, in g15_plans_and_proposals.rs to reuse its real old-binary builder).
The old daemon first completes an allowed control candidate. Then its Git
wrapper holds a clone call before the second worker has a box or handle.
Before restarting, the fixture proves the exact PID survived SIGKILL and
the store shows execution 2 running without a handle. After migration,
red:ownership names that row and its missing lock contract. The operator
syncs the limit to one, admits a new ticket and observes its capacity wait.
The wrapper reports completion of the real clone after release, but the
old row still retains capacity: neither a new lock nor an old exact host
identity exists. Expected IDs come from the fresh fixture sequence;
process liveness and completion come from the host wrapper, and requests
come from the scripted host model. Sabotage: omit legacy provenance or
accept absence of the new lock. The existing G5 current-binary restart has
an inherited lock; it cannot catch this missing old proof. The new scenario
is the ambiguous migration boundary, not a simulated store mutation.

Keep the current-binary old-child block in G4. Add kind=red to its existing
ownership reason/identity observations. Its comment now names changing the
kind as sabotage and removes release-on-row-end: that row remains running
while ownership is unknown, so the removed sabotage was not exercised.
Daemon-death and omitted-inherited-lock sabotage remain. All original
assertions in G3/G4/G5/G7/G9/G10 remain intact. No weaker clause or twin is
introduced, and no timeout supplies a passing result.

The real legacy fixture covers pre-box Git ambiguity. Legacy ended open
intents and failed teardown selection are source-audited here; runtime
upgrade evidence for those ambiguous old states remains for Y80. No host
reproduction or completed upgrade rehearsal is claimed.

Validation: cargo fmt --check and cargo clippy --all-targets --locked --
-D warnings. The latter compiles all e2e scenarios. E2e execution remains
the real host landing gate, per ticket instructions.

Test assertion lines removed: 0. The existing sabotage comment is corrected.
Binary lines removed: 21, replacing the guessed new-lock legacy proof and
unreserved red-return call; no binary module or guarantee is removed.
