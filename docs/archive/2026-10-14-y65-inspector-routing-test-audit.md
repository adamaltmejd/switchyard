# Y-65 inspector routing repair

## Retained failure

The operator reports that exact candidate
04109d5d4176d4e5e511bef82f652dba12852ec4 passed boxed checks 1519–1521
and review 1522, then failed all three new host continuation scenarios.
The run took 230.55 suite seconds and 244.39 wall seconds. Each debugger
client failed JSON.parse on an empty inspector discovery HTTP response.
Each initial controller ended failed/harness with named missing_report and
zero observed usage before debug-armed. No held tool, crash point or
continuation was reached. These are failures, not acceptance evidence.

The raw log and terminal remain at
/home/yard/validation/y65-04109d-host-continuations-20261014-evidence.
That path is absent in the worker box. This record retains the supplied
failure report; it does not replace or claim to reproduce the raw evidence.
No full landing gate was submitted for that candidate.

## Repair and test-audit Gate

The observer previously launched Node with the box's inherited proxy
environment and used fetch plus WebSocket for its guest-loopback inspector.
The observer now unsets the proxy variables and NODE_USE_ENV_PROXY before
Node starts. This applies only to that boxed debugger client. The real
controller retains its injecting model route. Discovery still makes one
HTTP request, with no retries or sleeps. Empty or unsuccessful discovery
now reports HTTP status and body length before JSON parsing.

The controller starts its inspector before making the fixture-held model
request. That existing held request is the discovery readiness boundary.
The fixture releases the request only after the breakpoint is armed; it
still requires the real unsafe tool's committed intent and debugger-held
invocation before daemon death. A failed discovery leaves that request held
and cannot reach the required point. Fixing observer routing is intended
to unblock this interaction; no controller timeout or deadline changes.
The worker requested a bounded operator probe of inherited versus scrubbed
observer routing, plus the retained terminal and request excerpts. Host
confirmation of this mechanism is pending.

Disposition: keep all G2/G5 assertion blocks, their external expected values,
refusal controls and independently runnable scenarios unchanged. No new
behavioral assertion or duplicate scenario is added. The change repairs
fixture routing; it adds no production tool, binary barrier or test-only
binary path. The existing sabotage comments and held-point proof remain.
Test assertion lines removed: zero. Binary lines removed: zero.

The same attempt retains the merge of exact landed Y-152 canonical
e2fe045704160871b3ca49706e0214b7be050e10. Its shared dependency target,
Yard invalidation, rebuild, named executable copies and whole-sequence
fixture mutex remain. Runtime test concurrency is unchanged. Every original
Y-65 continuation guarantee and acceptance requirement remains.

## Validation

Boxed cargo fmt --check, cargo clippy --all-targets --locked -- -D warnings
and cargo build --locked passed. Worker boxes cannot run host e2e; none was
run here. Fresh review and operator qualification of the affected actual
host scenarios are required before full landing. Prior review success and
isolated runs do not establish host acceptance of the integrated repair.
