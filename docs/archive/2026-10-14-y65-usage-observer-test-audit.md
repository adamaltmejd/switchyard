# Y-65 usage observer ordering repair

The operator confirmed the inspector routing mechanism with one bounded
read-only discovery probe in an existing fixture box. Inherited routing
returned HTTP 403 with zero bytes. The scrubbed debugger client returned
HTTP 200 with 662 bytes and one target. The retained record is
/home/yard/validation/y65-1d183e-host-continuations-20261014-evidence/operator-inspector-routing-probe.json.
This is operator-supplied host evidence, not a worker-box reproduction.
The effective loopback proxy correction remains in the candidate.

The operator's active native three-scenario run recorded initial usage
1000/100 in every scenario but had reached no restart or successor when
this repair was requested. It remains retained evidence, not a passing
continuation or host acceptance. No timeout counts as success.

## Test-audit Gate

Disposition: retain the G2/G5 continuation scenarios and every assertion.
The usage observer now uses the existing Watch::find instead of Watch::until.
observeUsage emits execution.usage before the tool executes and pauses.
said_before_end(debug-held) can read that event into Watch::seen. A later
until looks only at future frames and can time out on the paused tool.
find checks already-read history, then observes future frames if necessary.
The predicate still requires the exact event, execution and fixture token
value. Store assertions still require both exact input and output usage.

The original sabotage comments, external token constants and observations
remain. This changes event lookup, not the assertion's expected value or
held point. Existing scenarios do not replace this paused-tool mechanism;
no twin, new scenario, lifecycle or helper is added. The real debugger-held
unsafe tool intent must still be proved before the daemon is killed, and
all exclusive continuation, edited-input, corrupt-checkpoint and usage
increment controls remain. Test assertion lines removed: zero. Binary
lines removed: zero.

The landed Y-152 shared dependency target, clean/rebuild/copy sequence,
fixture mutex, distinct executables and concurrent scenario runtimes remain.
There is no sleep, retry, deadline change or guarantee narrowing.

## Validation

Boxed cargo fmt --check, cargo clippy --all-targets --locked -- -D warnings
and cargo build --locked passed. No host e2e was run in the worker box.
Fresh review and affected actual host scenarios remain gates for this
candidate. No full landing gate has been submitted.
