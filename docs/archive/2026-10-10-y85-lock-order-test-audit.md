# Y-85 capacity and target lock repair

## Failed host result

The operator ran cargo test -p e2e --locked on exact candidate
0dc20aa8ac594219cc42f9e3d1cbcfdbe608c0b4. It exited 101 after 773.81 seconds:
53 passed and one failed. The existing G4 scenario
independent_target_landings_share_the_project_execution_cap timed out at
line 881 waiting for target landings within 600 seconds. The log is
/tmp/switchyard-y85-0dc-host.log. All new frozen-input, future-admission and
real legacy migration controls printed ok. The full suite failed; those
individual results do not establish acceptance.

The operator's live read-only evidence is
/tmp/switchyard-y85-0dc-capacity-held-rows.json. Abandoned Y-4 attempt 4 was
readmitted as attempt 5. Implementation execution 8 was running, reserved,
and had no handle. Y-3 feature/c landing execution 6 was running without a
reservation or intent. Y-1 and Y-2 had landed. Both held rows persisted
until the observation deadline. No retries, operator process kills or
fixture latch changes were used.

## Root cause and repair

Abandonment ends an attempt without changing ticket readiness, so the
readmission was legitimate. The runtime regression was inconsistent lock
ordering. Worker launch reserved capacity before taking its selected target
lock for live containment and box launch. Final landing took the target lock
before waiting for capacity. At cap one, a reserved same-target worker
could wait for the target while landing waited for that worker's slot.

Final landing now acquires the target lock inside capacity::effect, after
reservation. It retains the guard outside the work future through ownership
proof and retained-reservation publication invalidation. Intent, current
approval and policy reads, canonical publication and target serialization
remain protected. Capacity and host ownership use their existing boundary;
no slot bypass, lifecycle or plan adoption was added. The shared boundary's
comment and binding architecture state the ordering.

## Test-audit dispositions and gate

The existing G4 independent-target scenario parks Y-4 before abandonment,
which prevents unrequested later executions. Every existing assertion is
retained. A new G4/G10 scenario supplies explicit same-target competition;
parking is not the runtime fix or the sole regression coverage.

The new scenario begins with a live limit of two. A real landing host gate
holds one slot and a same-target worker's real Git clone holds the other.
Both report their held points through fixture FIFOs before the operator
syncs the default limit down to one. Neither effect is killed. Releasing the
gate makes final landing report a capacity wait while the worker stays held.
Releasing the clone must let the reserved worker reach the real boxed
harness and scripted host model. The fixture holds that request while the
test observes the sole worker reservation, its pinfold box and the unchanged
target. Releasing the model lets the worker reach approval and the first
candidate land. Approving the worker candidate then lands it on the same
target. Host Git verifies both candidate identities in that target.

| Assertion block | Guarantee clause | Expected values outside Yard |
|---|---|---|
| Two held effects on feature/a; the worker has no box handle | G4 shared capacity and drainage after lowering the live limit | Fixture starts one landing host gate and one pre-box Git clone on the named branch; their FIFO reports hold both effects. |
| Final landing reports capacity, target remains unchanged and intent absent | G4/G10 same-target drainage | Spec capacity reason; host Git's old target; fixture has not released the reserved worker. |
| Released clone reaches the held model, sole owner is Y-2 and its real box exists, target remains unchanged | G4 shared slot and G10 target serialization | Fixture ticket order and held host model request; pinfold's box identity and host Git's old target. |
| Worker approval, first landing, second landing and both candidate ancestors | G4/G10 positive completion after drainage | Candidate commits read with host Git from real clones; canonical Git ancestry. |

The sabotage is restoring target-before-capacity in final landing. The
capacity event then proves landing is waiting while holding the target;
the released worker cannot reach the model and neither completion control
can pass. The test comment names it. A deadline is failure, never evidence
of refusal. Existing cap-one blocks hold one effect or race unrelated
targets; none forces a same-target reserved worker to meet final landing
after lowering the live cap. This holds the actual competing effects and
does not depend on scheduler timing. G4's Shown by clause records the new
scenario; G10's existing serialization mechanism is retained.

No assertion was removed or weakened. The fixture helper reuses the existing
FIFO hold for distinct effects. The diff removes nine test lines and five
binary lines, replacing helper setup, the misleading cancellation comment,
and the old final-lock placement and references. All frozen-input controls,
literal counters, provenance equality, live policy withdrawal and legacy
unknown semantics remain intact. No sleep, retry, fabricated row, hidden
barrier, model suppression or test-only binary path was added.

No repaired host run or sabotage run is claimed. The operator will review
the repair and run the full host suite on the new exact candidate. Candidate
checks are cargo fmt --check and cargo clippy --all-targets --locked -- -D
warnings. The host result above is evidence from the operator, not a run
performed inside this candidate box.
