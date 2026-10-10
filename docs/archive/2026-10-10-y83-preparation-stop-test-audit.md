# Y-83 preparation stop audit, 2026-10-10

The operator reproduced the preparation stop gap through the real CLI on
a67a8e6. The reported log is /tmp/switchyard-y83-stop-prep-probe.log. After
an allowed first ticket reached approval, execution 2's real clone child
was held with reserved=1 and no box handle. Stop returned stopping=2.
Releasing the child let the worker reach approval instead of stopped.
This worker has not rerun that host probe or inspected construction state.

Register the implementation's existing Notify before its effect task can
reserve capacity. Keep that same handle when the harness starts. Check
stop after preparation awaits and before the worker turn. An already
acting preparation child finishes under the inherited ownership lock;
the same execution then ends unchanged and raises stopped:unchanged.
Only capacity's existing external ownership proof releases its slot.
Also observe a pending stop after worker inspection before deciding a
candidate. There is no new execution state, owner or recovery path.

Test-audit Gate for the added assertion blocks:

- Guarantee: G9's stop contract now names held reserved preparation in
  its shown-by clause. G4 requires the same reservation to remain while
  that child acts. The binding execution contract documents both.
- Sabotage: move stop registration back to run_harness and the held clone
  loses its stop. Ignore preparation stop checks and the execution gets
  a box handle despite the pending stop. Clear reserved at stop acceptance
  and the held-child row assertion fails. The comment names these changes.
- Distinction: the existing stopped waiter has no external effect or
  reservation. The existing acting-worker stop has already registered its
  harness handle. Neither observes the reserved pre-box preparation gap.
  The new scenario holds that gap with a real host Git wrapper and FIFO.
- External expectations: fixture ticket order supplies execution 2. The
  wrapper supplies the held PID; kill -0 proves it still exists after stop.
  The store is read from outside for running/reserved/no-handle before and
  after acceptance, then ended/unchanged/no-handle after release. The spec
  supplies stopped:unchanged and the scripted host model supplies absence
  of a Prepared turn. Same-project first-ticket approval and deliberate
  later start provide allowed controls. The latter reaches approval and
  has exactly one opening turn, with the old reservation cleared.

Keep all existing assertions and guarantee clauses. Assertion lines
removed: 0. Binary lines removed: 4, replacing late handle registration
and mutable run handling. No sleep, retry, passing timeout, public endpoint
or test-only binary barrier was added. This is a stop scenario, not a crash
scenario; the existing current-binary and legacy crash proofs are unchanged.

Validation: cargo fmt --check, cargo clippy --all-targets --locked --
-D warnings and git diff --check passed. The host e2e suite and sabotage
runs were not run, as the ticket requires host validation after approval.

Integration input: /yard/input was empty, no Git remote was configured,
and f9f4602f1b1f7df7abc0019c9ab55e8e4231d1db was absent from the clone's
object database. The supported frozen target bundle was requested.
Y58 integration cannot be claimed from these inputs.
