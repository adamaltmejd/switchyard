# Y-60 held abandonment fixture repair

The operator's exact candidate
`94db24ac1260e5f0232f3a8f5d6831f40f58b8ec` passed boxed fmt/clippy/build
807-809. Its focused real host run of a_gate_errors_start_reruns_that_gate
ended 101: 0 passed, 1 failed, 61 filtered, 21.02s. The log is
`/tmp/switchyard-y60-94db-focused-host.log`. No full suite ran on that head.

The failure at g08_review_is_a_publication.rs:812 observed attempt3,
execution13 stopped:unchanged before the second ticket's held gate.
The first ticket now lands feature.txt. The shared fixture implementer
wrote those same bytes for the second ticket, so Git had no new commit.
This was a fixture precondition error in the preceding candidate.
All preceding landing-error and supported requeue assertions ran, but
that does not make the failed scenario a pass or complete Y-60 acceptance.

The second ticket's scripted host model now asks the real worker to commit
abandon.txt. Its new candidate can reach the existing FIFO-held host gate.
The first ticket's candidate-error and landing-error paths remain unchanged.
All exact merged-input, same-approval retry, cleanup and held abandonment
assertions remain. No production code, binary barrier or binding contract
changed. The previous integration and audit records remain historical.

## Focused test-audit gate

1. G8's existing Shown by clause: a gate ending after abandonment records
   abandoned, no check and no attention. No assertion or clause changes.
2. Existing sabotage: remove supervise::gate's end-path liveness check;
   the released failing gate records a judgment after abandonment. The
   scenario comment still names it. Landing error classification and queue
   repair sabotages remain unchanged.
3. The held host gate establishes the distinct abandonment race. Candidate
   and landing infrastructure errors do not exercise it. A new committed
   fixture file restores reachability, with no twin scenario, weaker
   observation or retry. The successful first ticket remains the same-project
   allowed control.
4. Expected outcomes remain from G8; the host fixture supplies the committed
   bytes, FIFO hold and release. CLI history and the store observe the ended
   gate, checks and attention from outside the binary. No expected value
   comes from a copy of production logic.

Test diff: 8 added lines, 1 removed line. No assertion block removed.
Binary diff: 0 lines added or removed. No crash point or containment changes.
This is a focused fixture gate, not a whole-tree test sweep.

Cargo fmt --check and cargo clippy --all-targets --locked -- -D warnings
both passed for the repaired candidate. Host e2e and sabotage remain unrun here
under the worker instructions. Fresh exact-head focused and full host runs,
published review and landing validation are still mandatory before acceptance.
