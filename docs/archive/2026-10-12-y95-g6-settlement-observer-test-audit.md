# Y-95 G6 settlement observer repair

## Actual landing failure and target integration

Operator-supplied actual landing evidence names target
`7ec161f0bfd60f58d8a0ad5d48cff8a7347e97cc` and failed merged ref
`60b73f2fdef26a4c65269573d99a5e497a953f0c`. Host e2e ended with
71 passed and 1 failed in 496.97 seconds. G6
`selected_admission_refuses_without_work_and_freezes_operator_inputs`
failed at its full execution-row equality assertion. This remains a failed
actual gate, not successful landing evidence.

The supplied expected and observed planner execution1 are equal except
`reserved` changes from 1 to 0 and `teardown` changes from 0 to 1.
The initial snapshot was taken immediately after planned `attempt.ended`.
The excerpt shows selected-failure acceptance passing, but provides no new
complete-suite or landing success. No new host reproduction occurs here.

The operator supplied `/yard/input/target.bundle`. Fetching its main and
merging FETCH_HEAD integrates exact 7ec161f, adding only the dated
construction-scope/restart record. That record was read. Existing Y-95
required-proof source/assertions, immutable archives and complete architecture
union remain. No .yard file differs against that operator target.

## Demonstrated mechanism and bounded repair

`jobs/mod.rs::spawn` runs implementation inside `capacity::effect`.
The work records completion before returning to the wrapper, which drops
its inherited host lock and proves box absence and lock freedom before
`capacity::release` updates teardown and releases reservation. Therefore
attempt completion does not establish stable full execution rows.

`cleanup::next` excludes any attempt with a running or reserved execution.
Its cleanup then sets cleaned and ends before its own wrapper releases.
The existing public history exposes both ended and released events.
The observer now binds cleanup to the exact ended planner attempt, then
waits for release of that exact cleanup execution before snapshotting.
This settles both planner and cleanup rows before the refused admission.
It retains the full equality assertions rather than removing lifecycle
columns or treating unknown ownership as completed teardown.

Only the G6 observer changes. No production ownership, eligibility, deadline,
proof handling, lifecycle, capacity or containment change is justified here.
No retry, sleep, thread cap, synthetic row, private state edit, process kill
or new barrier is introduced. Failure to release cannot become a pass.

## Test-audit Gate

Disposition: keep every assertion and repair the observation boundary in
G6's existing selected admission scenario. G6 already promises unsupported
bounded-repeat admission changes no attempt/execution and makes no model
call, with a corrected same-project preset/override control. G2's immutable
selected-plan observations remain. No guarantee row is weakened or changed.

The existing sabotage comment names bypassing runnable admission, changing
persisted selection and losing overrides or explicit graph inputs. All would
still fail the same assertions. The whole-row comparison now starts after
legitimate asynchronous work finishes; an admission write still changes it.

This scenario tests admission after a real completed planner; inspection
validation does not admit that work. Extend the existing scenario and add no
twin. The settlement wait follows the same public event mechanism already
used by G2 planner cleanup, with no new crash or restart claim.

Expected plan/reasons/overrides come from fixture literals and G6. Heads
come from host git. No-write observations compare external store snapshots
and host model request counts. The valid held same-project override, synced
future preset and one-off controls are retained. No assertion is deleted.
Test lines removed: 1, replaced by the exact ended-attempt binding and
settled history wait. Binary lines removed: 0.

## Validation and remaining scope

Boxed `cargo fmt --check` passed and
`cargo clippy --all-targets --locked -- -D warnings` passed in 1.51 seconds.
`git diff --check` passed. Host e2e and sabotage remain
operator-owned and unrun here. This source-traced observer repair addresses
the supplied row delta; no new runtime pass is claimed. Fresh exact-head host
validation, published review and actual merged landing gates remain required.
The full original Y-95 acceptance contract, Y-96, Y-61 and redesign/release
through Y-80 remain. Y-81 stays deferred. Historical evidence is unchanged.
Models, reviewer, native package/pins and operational daemon are unchanged.
