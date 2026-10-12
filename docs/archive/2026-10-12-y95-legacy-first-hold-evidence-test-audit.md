# Y-95 legacy planner first-hold evidence

## Failed run and source boundary

The operator reports actual landing1848/host1852 failed with 71 passed,
1 failed in 499.01 seconds on exact candidate
`2860c3051031d7bbc78bd7cfb452de30fde3371d`. G6's checked-settlement observer
repair passed. G8 `an_unbound_legacy_planner_requires_a_fresh_admitted_attempt`
failed at its first `Latch::wait_held`, after the unchanged 300-second hold
limit. This remains a failed actual gate. No new landing or full-suite success
is established by the other passing scenarios.

Machine Drop removed the fixture. The retained terminal tail cannot establish
whether the old execution failed to start, its box/harness failed, a model
request arrived, or the script missed the hold. None is claimed as the cause.
No old fixture, process, ownership outcome or model response is reconstructed.

Source reading confirms the fixture builds and uses the real pre-target
`c85369dfaec8f8554da49ddf02a4dbec03bcad02` binary. Its daemon serves and
project init/ticket creation complete before the first wait. The old worker
must prepare its image, clone, box and harness before a model request can
hold. Its spawn error path can end the exact execution and raise red:error
before any request. This is a source possibility, not a reproduced defect.
The model records a request before invoking the script. A request offering
`yard_publish_report` takes the planned branch; other requests take the hold.
The old schema has no current reserved/lock_contract/teardown columns, so
current ownership fields cannot be invented for this historical execution.

The test currently panics at the wait and Machine Drop removes the root.
Source evidence does not select a concrete runtime cause. This bounded pass
therefore adds failure capture, not a production repair or a green claim.

## Narrow capture

Only this scenario's first hold is wrapped. On its existing panic, before
fixture destruction, the test reads the host model's request count and count
of requests offering the report tool. It projects read-only store observations:
up to 16 attempts, executions and attention rows, and the last 64 audit event
identities. These include exact execution/attempt/ticket IDs, lifecycle and
exit fields, registration, handle presence and attention kinds/reasons.

Raw detail is represented only by length and fixed substring flags for image,
box, timeout, ownership and registration. These flags identify observed text
categories, not proven causes; absence of a flag proves no success. The output
excludes detail text, audit text/payloads, handles, model bodies/headers,
credentials, session/bearer values, configuration and environment contents.
All selected columns exist in the real old schema and migrations.

Evidence is printed in the failed test output before cleanup. Each query is
bounded and read-only; separate observations are not an atomic snapshot.
A capture failure prints a fixed unavailable message. The original panic is
resumed in either case. No failure becomes a pass. No diagnostic CLI child,
box listing, extra model call, private state write, or new wait is introduced.
Machine Drop and the hold deadline remain unchanged. The diagnostic does not
retain raw fixture files or interfere with existing cleanup.

This capture can distinguish an absent model request, a request taking the
other script branch, a still-running execution and recorded startup/error or
ownership attention. Its projected flags may not identify an underlying
runtime cause. Further host diagnosis must use those actual identities and
observations rather than infer a repair from flags alone.

## Test-audit Gate and validation

Disposition: retain all G8/G6 historical unbound refusal, unknown evidence,
edit/no-scheduling and same-project abandon/fresh-publication assertions.
No guarantee row or named control changes. This is a change Gate, not a sweep.

G8's existing Shown by is the stopped real pre-target planner refusing
unbound_report and completing through a fresh admitted attempt. Existing
sabotage comments name unbound continuation, pending-edit scheduling and
fabricated historical bindings. Those assertions and their failure behavior
remain. This adds diagnostic output only, not an assertion block or a twin
scenario. The hard stopped-old-execution upgrade and successful same-project
control remain. Historical rows come from the real old binary; refusal and
unknown bindings come from the spec; report content is a fixture literal.
Capture values are observations from outside the binary, with no expected
runtime outcome asserted. Test lines removed: 1, the plain hold call is wrapped
without changing its wait or panic. Binary lines removed: 0.

After formatting, `cargo fmt --check` passed. Clippy all-targets with the
locked dependencies and warnings denied passed; the scenario compiled.
Host reproduction, capture-path execution and sabotage remain unrun here.
No unchanged suite is repeated. Fresh operator-owned exact-head host evidence,
published review and actual landing gates remain required. No assertion,
wait, concurrency, ownership or containment control is weakened.
The whole original Y-95 scope, Y-96, Y-61 and redesign/release through Y-80
remain; Y-81 stays deferred. Required-proof behavior, native package/pins,
models, reviewer, .yard and frozen operational daemon remain unchanged.
Prior archives and failed-run evidence are immutable.
