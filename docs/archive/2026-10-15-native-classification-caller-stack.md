# Y73 remaining native poll-chain stack pressure

The operator's exact 15289c41795b9f17cac8be88ad74aa13cc263901 again
aborted with a Tokio worker stack overflow in
`concurrent_readers_share_capacity_and_join_exact_results_across_restart`.
The retained host evidence is
`/home/yard/validation/y73-15289c-groups-20261015-evidence`.
Its waiting harness deadline is not a pass. The earlier 530fe30 fatal result
also remains evidence. Neither run captured the overflowing frames. The
previous child-capacity pin reduced reservations but did not fix that crash.

## Measurements

These measurements use Linux x86_64, rustc 1.99.0
`b940084d7 2026-09-28`, locked dependencies and the normal unoptimized
`cargo build --locked` binary. No Yard daemon or host test was run here.
`objdump -d -C --no-show-raw-insn target/debug/yard` supplies both the frame
prologues and the direct call edges. Frame bytes count stack-probe decrement,
remaining stack decrement and saved-register pushes, excluding return addresses.
This investigation measures generated poll frames and connected caller/callee
paths, rather than inferring stack use from future layouts.

| Poll frame | 15289c | Selected repair |
|---|---:|---:|
| Execution dispatch | 361,320 | 275,512 |
| Native landing | 949,912 | 854,488 |
| Native child dispatch | 87,688 | 43,192 |
| Child capacity effect | 144,792 | 77,928 |
| Child task-local work closure | 44,904 | 328 |
| Native classification | 167,624 | 72,248 |
| Classifier capacity effect | 149,512 | 77,928 |
| Classifier task-local work closure | 48,056 | 328 |

The gate poll itself remains 95,000 bytes and its box-gate poll remains
258,984 bytes. The classifier runner poll remains 132,376 bytes and scoped
work remains 225,672 bytes. The reduction comes from removing embedded work
storage from the caller and capacity/task-local wrappers, not altering those
runners or their effects.

Two selected direct-call paths expose the combined pressure. The gate path
runs from execution dispatch through landing, child dispatch, capacity,
task-local scope, gate, box gate, verification, Pinfold runtime identity,
version/control, output collection and its stderr join result replacement.
The classifier path shares dispatch/landing, then classification, capacity,
task-local scope, classifier run, scoped work, harness, Pinfold exec and the
same collection/join boundary. Each adjacent function is a generated direct
call, including the added `Pin<Box<_>>::poll` wrappers.

| Selected connected path, including return addresses | 15289c | Caller plus classifier-call pin experiment | Selected repair |
|---|---:|---:|---:|
| Gate | 2,266,944 | 2,112,960 | 1,929,936 |
| Classifier | 2,278,864 | 2,124,960 | 1,883,104 |

The intermediate experiment pinned landing at execution dispatch and the
whole classification call in landing, retaining the old child-capacity pin.
Both selected paths still exceeded 2 MiB before runtime callers. It was
insufficient. Moving the existing child pin only to its inner gate was also
rejected: the landing poll grew to 961,656 bytes. Keep the useful outer pin.

The selected repair keeps that outer child-capacity pin, pins its inner gate,
pins native classifier work before capacity, and pins landing at execution
dispatch. These three added boundaries reduce the measured gate path by
337,008 bytes and the classifier path by 395,760 bytes. They bring these
paths below the documented normal 2 MiB thread stack, with nominal remaining
space of 167,216 and 214,048 bytes before other callers. The directly resolved
Tokio boxed-task poll/catch-unwind caller segment adds 496 bytes beyond the
execution-dispatch frame; resolution stops at an indirect unwind call.

These are selected static paths, not a host trace, measured dynamic high-water
mark or proof of the maximum stack. Indirect calls, other branches, recursive
identity binding and platform/compiler differences limit that inference.
The runtime continues to use its normal stack settings. Only the operator's
unchanged host case can confirm whether this repair resolves the fatal crash.
Compact call paths, prologues and extraction scripts are in
`/yard/proof/y73-stack-caller`; they are candidate proof, not merged code.

## Contract and validation

The pins stay in existing execution dispatch. The gate and classifier still
enter the same capacity effect and task-local host-lock scope. Acquisition,
reservation, release, target locks, error finalization and operator-retry
waiting are unchanged. There is no additional task, owner or lifecycle.
All original Y73 routes, identities, failure records and Y132 source groups
remain intact, including the earlier directory and publication-schema repairs.
No tests or assertions changed. No runtime, concurrency or deadline changes
were made. `cargo fmt --check`, `cargo clippy --all-targets --locked -- -D warnings`
and `cargo build --locked` passed. Current review and real native host
confirmation remain operator gates.
