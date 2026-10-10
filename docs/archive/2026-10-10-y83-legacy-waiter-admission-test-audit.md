# Y-83 legacy waiter admission audit, 2026-10-10

The operator's exact-head host observation at 7d382df found the fixture
waiting for Y-3's execution.waiting event while both original attempts
still held lanes. Control had reached candidate approval; it had not
landed or completed its ticket. With two lanes, Y-3 could not be admitted.
The reported log is /tmp/switchyard-y83-repair2-host.log, session19009.
This worker has not rerun the host scenario or inspected construction.

Keep the legacy pre-box scenario. When syncing max_executions=1, configure
three attempt lanes so the original two attempts and Y-3 can coexist.
Observe execution.waiting naming Y-3 and reason=capacity before releasing
the held child. Correct the control comment to candidate approval.
The execution cap stays one. Old execution 2, its surviving PID and old
row proof, the second startup, current red:ownership, retained reservation
and absence of waiter model requests all remain.

Test-audit Gate:

- Guarantee: G5's retained legacy owner excludes an admitted waiter. The
  binding shown-by clause now explicitly names that admitted waiter.
- Sabotage: accept absent new-lock proof for a legacy execution or omit
  legacy provenance, including after child completion, as the existing
  comment names. Releasing the owner lets the admitted waiter act.
- Distinction: the real old binary lacks the inherited-lock proof used by
  the existing current-binary restart scenario. The held pre-box child and
  second restart remain the hard case for legacy ownership reconsideration.
  This fixes that scenario's admission precondition and adds no twin.
- External expectations: the fixture configuration supplies three lanes
  and one execution slot; its ticket sequence supplies Y-3 and execution 2;
  the spec supplies capacity as the wait reason; the host wrapper supplies
  child completion and the scripted model supplies waiter absence. Waiting
  on an unadmitted ready ticket or a timeout is not exclusion evidence.

Assertion lines removed: 0. Binary lines removed: 0. All existing assertions
and guarantees remain. No sleep, retry, passing timeout or binary barrier
is added. Earlier archive evidence remains unchanged as dated history.

Validation: cargo fmt --check and cargo clippy --all-targets --locked --
-D warnings. Host e2e remains deferred to the landing gate as instructed.
