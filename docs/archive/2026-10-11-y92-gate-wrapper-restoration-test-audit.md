# Y-92 gate fixture wrapper restoration

Date: 2026-10-11. Parent: fdc15e4bd05cf542007afa4403daa87a7a1f664b.
Ticket edit: revision 15, bounded fixture correction.

## Actual evidence and source-audit miss

The operator reports exact fdc15e4 normal-review focus terminal PASS:
1 passed, 0 failed, 22.17 seconds. This is historical evidence for that head,
not validation of this edit or the whole integration.

Gate-error focus session 50139/test 657765 remained live at this admission.
The operator observed execution 5's normal gate ownership reconciliation
and approval, attention 3, seq 33-36. Later landing 6 unexpectedly landed at
seq 64, ticket done at 65 and cleanup 10 removed at 70. The original expected
landing red did not appear. No whole-focus terminal PASS or failure is
claimed here; the live session is left to finish naturally.

The prior source audit missed wrapper composition. JudgmentOwner::new
replaces the daemon PATH pinfold wrapper. Its wrapper retains the new real
completion and listing/lock controls but omits the original machine.root/armed
box-up refusal. The subsequent original landing-error block recreates that
ordinary fixture file, expecting the old wrapper to refuse box up. With no
wrapper reading it, the real landing succeeds. This is a reproduced fixture
regression, not a diagnosis of a production ownership failure or the earlier
historical gate-error first probes. Prior immutable records are unchanged.

## Correction

After normal gate judgment ownership reconciliation, the fixture reinstalls
the original pinfold wrapper's exact armed-path box-up refusal. All other
calls again delegate directly to the real pinfold. This occurs before the
original approval/target-advance/armed-landing-error block. The ownership
helper's hold and listing controls have finished; their files and the actual
inherited inode are not deleted or replaced as ownership evidence.

The original block still arms the same file, observes landing infrastructure
error with unchanged canonical and no judgment/repair, removes the file,
answers supported start, and proves actual same-approval git landing. The
later abandonment and unstarted-child controls remain unchanged. No assertion,
scenario, deadline or binary flag is added, removed or weakened. Production,
ARCHITECTURE.md, package pins/closure, native bridge, CI and .yard are unchanged.
The normal-review fixture is unchanged.

## Test-audit Gate

1. G8's existing shown clause: landing-child infrastructure error leaves
   canonical unchanged, publishes no judgment and starts no repair; supported
   start requeues under the same approval and lands the exact merged input.
   G5/G8's normal completed gate ownership refusal/reconciliation clauses and
   existing abandonment/unstarted-child clauses are all retained. The spec
   already describes them, so it needs no contract edit.
2. Existing binary sabotage and comments remain: treating a gate infrastructure
   error as a verdict or allowing a repair route breaks the retained no-output,
   no-repair and canonical assertions. Losing completed-gate ownership
   reconciliation still prevents the earlier same-head retry's approval.
   This fixture defect instead prevented the intended box-up refusal from
   occurring; restoration makes the original sabotage controls meaningful.
3. This repairs the original scenario's seam, without a twin scenario. The
   new ownership hold remains the hardest release case. The subsequent
   landing block still checks a target advance and exact merged identity,
   infrastructure refusal, then real same-approval success. Other scenarios
   do not replace this chain. No crash boundary is added or modified.
4. The armed file is ordinary fixture input. The wrapper's explicit exit
   supplies infrastructure failure through the real pinfold user seam; it
   does not fabricate a model frame or listing reply. The real Pi harness,
   native registration and actual box exec still run. Expected red kind and
   reason, absent judgment/route/check and unchanged canonical come from G8.
   Expected merged parents/heads/proof come from fixture commits and host git;
   execution, check and approval identities are externally observed through
   supported CLI and read-only SQLite. The allowed control removes the same
   ordinary armed file and executes the real gate/landing under the original
   approval. Every existing assertion and expected identity is preserved.

Disposition: retain all blocks; restore only wrapper behavior after the
bounded ownership exercise. Test assertion lines removed: 0. Binary lines
removed: 0. No private attempt inputs/store writes, fake liveness, new box
owner, replaced Pi, sleeps, retries, runtime pruning/kills/restarts, changed
threads/deadlines or host evaluation are introduced.

## Validation and outstanding gates

Local validation passed:
cargo fmt --check; cargo clippy --all-targets --locked -- -D warnings;
cargo build --locked; git diff --check. The only source delta is nine added
fixture lines; a path-scoped diff confirms production, spec, package, bridge,
CI and .yard are unchanged. No e2e is run in this candidate box.
No host checkout or live test is edited, interrupted or restarted.

The operator must read this exact delta and wait until session 50139 is
terminal before running the changed gate-error focus and appropriate
interrupted-publication regression, followed by the unchanged full 72-test
suite, current review and actual landing. No earlier head's pass transfers.
The package/native bridge/build inputs remain byte-identical; no redundant
focused package run is requested. Original Y-92 package and integration
scope, release dependencies and missing-path P3 carry remain unchanged.
