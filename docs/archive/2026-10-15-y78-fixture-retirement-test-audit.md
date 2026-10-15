# Y-78 usage fixture retirement repair

The operator's host run of candidate 4566a97 failed the final project/attempt
aggregate equality in g07_usage.rs: 0/1 passed in 30.22 seconds, child 101.
The reported read-only store has old attempt 1 ended, new attempt 2 live
with execution 4 running/reserved, and cleanup execution 5 for attempt 1
ended/removed/released. The project correctly includes the new execution;
the historical attempt correctly excludes it. Known sums remain correct.
The host evidence paths supplied by the operator are not mounted here.

The existing G7 usage fixture now parks Y-1 through trusted operator MCP
before abandoning its attempt. The park result must report parked true.
It matches cleanup completion to the original attempt, requires removed,
then waits on that exact cleanup execution's execution.released event before
the deliberate idle daemon reopen. An ended row alone is not checked release.
No production code, product control or statistics population changes.

## Assertion gate and remaining fixture inspection

The retained G7 aggregate blocks protect omission fidelity, execution-local
increments and graph/usage evidence after cleanup/restart. Literal fixture
measurements, all per-field completeness/provenance, zero/partial/unknown
checks, artifact availability and identity/time assertions stay intact.
The existing sabotage comments still apply: sum cumulative predecessor
spend or coalesce missing costs into complete zero bills. The added park
result and cleanup outcome are setup observations through supported MCP
and history. Their expected values come from the fixture's requested park
and the ordinary removed cleanup outcome. Release is observed by exact
execution identity on status history, not by a sleep or retry.

The full usage fixture was read. Its earlier daemon reopen deliberately
retains the interrupted second execution while the host model request is
held; it makes no idle or project/attempt population equality assumption.
Its initial aggregate equality is before abandonment in the only live
attempt. The final equality becomes valid by retiring the ticket before
abandonment, not by filtering the project. The fixture adds no other ticket.

The independent G7 landing case was inspected and left unchanged. It lands
Y-1, which becomes done, while deliberately retaining unrelated live Y-2.
Its usage read is attempt-scoped and never equates project and attempt
populations or reopens the daemon at an ended cleanup boundary. Operator
qualification of that case remains separate.

Cost: no new fixture, historical build, box, model turn, deadline or retry.
Parking removes unintended replacement work. The original cleanup and
reopen remain; the release wait uses the existing history watcher.
Test lines removed: 2, replacing the broad cleanup predicate.
Binary lines removed: 0. No assertions or coverage were dropped. This is
fixture setup repair; ARCHITECTURE.md's product contract is unchanged.

Candidate validation: cargo fmt --check; cargo clippy --all-targets --locked
-- -D warnings; cargo build --locked. Host e2e remains operator-only and was
not run here. No operational binary, model, reviewer or .yard file changed.
