# Y-83 initial-read history observer repair, 2026-10-10

The operator reports the exact b9f57547639ea2c2e2f7161619f90d5933da2c76
host suite remains live in session 93181, log
/tmp/switchyard-y83-initial-load-repair-host.log. Read-only observation of
fixture ye-2361306-g4-gate-policy-slot shows execution 2 running, reserved,
lock_contract=1, without landing children or a prepared intent. Its start
is audit seq 10 and reservation seq 11. Git child PID 2371972 is held at
branch validation, with fd 12 referencing the actual landing lock, awaiting
initial-release. These are operator observations, not worker-run tests or
a terminal suite result. No process or fixture state was manually changed.

The observer ordering was wrong. watch.said records events in Watch.seen
while awaiting the FIFO. watch.event delegates to until, which reads only
future events. If the landing start arrived before the FIFO, the observer
waited for another start that cannot occur while that child remains held.

Use existing Watch::find for this lookup. Its predicate remains exactly
execution.started with data.kind=landing. It returns the already-seen
start or waits for that same event. No assumed execution id, broader match,
retry, sleep, timeout-as-pass or runtime barrier is added. The code guard,
all original assertions and all 50 scenarios remain unchanged. The old
host fixture is left to finish through its own cleanup.

Test-audit Gate: keep the extended G4 initial-target-read block in
landing_gate_policy_reads_hold_the_shared_execution_slot.

- G4's existing shown-by clause still describes the initial owned target
  read before prepared ref/child existence and one-slot continuation.
  There is no guarantee or expected-value change.
- The original omission sabotage remains: move the initial load outside
  preparation and the first owned target read occurs after the prepared
  ref. The canonical-state assertion fails. The gate-loop and parent-slot
  sabotages remain in the comment. None was run in this worker.
- This is a history-consumption repair within the same scenario, not a new
  mechanism or duplicate assertion. FIFO and event arrival order can vary;
  the existing helper observes the same recorded identity in either order.
- Expected phase order, names and landed bytes remain fixture-defined.
  Host Git, FIFOs/open descriptors and external rows remain the observers.
  Parent identity still agrees across the start event, sole reservation
  and first child. Every reservation, lock, ref/child absence and allowed
  continuation assertion is preserved.

Assertion lines removed: 0. Binary lines changed: 0. Binding architecture,
prior dated archives, retained-owner refusal/control, Y58 MCP integration,
operational .yard and frozen daemon/reviewer settings are unchanged.

Worker validation: cargo fmt --check, cargo clippy --all-targets --locked
-- -D warnings and git diff --check passed. Host e2e was not run here, as
instructed. Fresh review must judge this observer and the runtime guard;
full exact repaired-head host validation remains required before approval.
