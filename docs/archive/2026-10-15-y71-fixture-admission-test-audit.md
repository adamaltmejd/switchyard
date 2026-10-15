# Y-71 integration fixture admission repair

The operator's native validation of 0278c2 failed both integration scenarios
before integration. Starting the parked fixture ticket correctly refused as
not ready; the collision control therefore never reached input validation.
Those runs are failures, not acceptance evidence.

Both existing G2/G9 scenarios now create a real read-only planning ticket,
publish its report through the scripted host model, observe the planned end,
and wait for that attempt's cleanup execution to release. Explicit preset
admission then uses the settled ticket. No parked readiness rule, scheduler,
or binary path changes. There is no unpark race, sleep, retry or new scenario.

The G2 collision block retains its exact invalid reason and input identity.
Its outside store observer now compares attempt and execution IDs before and
after refusal, because real planner rows already exist. The expected unchanged
sets come from the settled store before the request. Removing reserved-name
validation or creating an attempt/execution before refusal breaks this block.
The supported preset remains its positive control on the same ticket. This is
the existing namespace-collision observation, not a twin; other admission tests
do not declare this candidate source name.

Successful admission supplies the actual graph attempt ID for retained clone
paths and source refs. The source publication query also scopes to that attempt.
This prevents the preliminary planner from shifting fixture identity assumptions.
All prior held-partial restart, real source write refusal, canonical containment,
Git content, conflicting merge, retained repair, exact stale approval and valid
C-only approval/landing observations remain. Their expected bytes and conflicts
come from fixture commands and host Git, not a copy of Yard's logic. The host
model latch still holds the real integration box after its partial write; the
store and host file establish the state before daemon death.

Disposition: retain both scenarios and repair their admission precondition.
G2's Shown by clause now describes the settled planner and no new rows.
No guarantee or assertion block is removed. Binary lines removed: zero.
Two empty-table assertions are replaced with unchanged-ID-set assertions.
No operator approval question or follow-up ticket is needed.

Validation: boxed cargo fmt --check, cargo clippy --all-targets --locked --
-D warnings, and cargo build --locked. Host e2e remains operator-run and
has not been rerun in this workspace.
