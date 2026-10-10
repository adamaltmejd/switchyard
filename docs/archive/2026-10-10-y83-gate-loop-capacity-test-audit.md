# Y-83 landing gate-loop capacity repair, 2026-10-10

Review1202/check637 passed exact combined candidate
2f8ee592fe4a163b78d80f13385aa4c21e18f088 with a P3 finding. The operator
identified an integrity exception: gate-loop admission could read protected
paths through Git outside the landing reservation and inherited host lock.
This is static source evidence, not a claimed host reproduction. The operator
reports the combined host suite passed 49 scenarios in 179.04 seconds. That
run does not establish this repair or the omitted path.

Wrap gate-loop admission in capacity::effect on the existing landing row.
Acquire canonical inside the effect, as final admission already does. Keep
the current approval, policy, revision, proof and digest reads unchanged.
The effect returns and proves release before the child reserves and runs.
No parent reservation spans a child gate. No lifecycle, ownership record,
configuration setting or dependency is added.

Source audit of queue::land: preparation's rev-parse, ancestry, merge-tree,
commit-tree and update-ref share its landing effect. Conflict-return bundle
creation stays inside that effect. Failed-child return reserves the landing
again before bundle creation. Final admission's protected-path read,
compare-and-swap and ancestry proof already share the final landing effect.
The gate-loop protected-path read was the omitted execution-owned launch.
Both returned call sites remain guarded. queue::next's pre-execution policy
screen and reconcile::decide_intent's recovery observations are separate
call sites, unchanged here; this focused audit does not claim new runtime
coverage of those paths.

Test-audit Gate: keep landing_gate_policy_reads_hold_the_shared_execution_slot.

- G4's new shown-by clause names the protected-path read between landing
  gates. The first real host gate supplies the allowed control. After it is
  held, arm the existing host Git wrapper for the next protected-path diff.
  Release the gate and hold that actual Git child before the second gate.
- The observer is the host FIFO, the external store and canonical Git. The
  fixture supplies the gate names, limit one and expected landed bytes.
  The first child row supplies the parent identity; the guarantee here is
  agreement with the sole reserved landing row, not an invented fixed id.
  No second gate exists while the protected-path child is held. Releasing
  that child lets the second host gate write its marker and the queue land.
- Removing the gate-loop effect fails the reserved-owner assertion.
  Keeping the parent reservation during its child prevents the allowed
  completion. Both sabotages are named in the scenario comment.
- Existing G4 blocks hold a worker or host gate, not the Git read between
  landing gates. Existing G2 policy blocks prove withdrawal and liveness
  decisions, not reservation ownership. None is removed or weakened.

The scenario uses real boxes, a real daemon and the scripted host model.
It adds no sleep, retry, binary barrier or internal seam. It does not prove
an inherited-lock crash specifically at this policy read: that boundary is
source-audited through capacity::effect's HOST_LOCK scope and Git's common
inheritance path. Existing G5 held-child restart assertions remain intact.
Assertion lines removed: 0. Binary statements removed: the explicit
canonical-guard drop, now performed by the effect's scope. All Y58 stdio
code/scenarios and G6/G7 clauses, Y83 guarantees and prior integrity repairs
remain unchanged. Repository .yard is untouched.

Worker validation: cargo fmt --check, cargo clippy --all-targets --locked
-- -D warnings and git diff --check passed. The host e2e suite was not run
in this worker, as instructed. Fresh checks, review and full exact-head host
validation remain required before final approval.
