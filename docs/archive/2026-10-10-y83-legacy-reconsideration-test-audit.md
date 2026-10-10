# Y-83 legacy reconsideration test audit, 2026-10-10

The operator's static Gate at 376c31d found that the legacy-child test's
final snapshot could precede a delayed incorrect release. The fixture
reported child completion but did not force another ownership decision.

Keep an_upgrade_retains_a_legacy_pre_box_child_without_an_ownership_proof.
After the host wrapper reports completion, stop and restart the current
daemon through the existing harness. Wait for its normal serving output
and read current status through the CLI. Then observe current red:ownership
for execution 2 and its running, reserved, legacy-contract row, followed by
the host model's absence of waiter requests. Startup reconciliation has
now reconsidered the owner after completion. A historical attention event
alone does not supply this proof.

Test-audit Gate:

- Guarantee: G5's real pre-capacity child migration clause, now explicitly
  including retained capacity and current attention after a second restart.
- Sabotage: accepting absent new-lock proof for a legacy execution,
  including on reconsideration after the old child completes. The test
  comment names this boundary.
- Distinction: current-binary inherited-lock coverage cannot catch missing
  legacy proof. The initial crash holds the real old Git child, proves its
  surviving PID and reads its old row before upgrade. Reconsidering after
  completion is the hard case for a delayed guessed release. No new twin
  scenario is introduced.
- External expectations: the fresh fixture sequence supplies execution 2;
  the host FIFO supplies completion; serving and the supported CLI read
  establish completed startup; the store supplies the current reservation;
  the binding contract supplies red:ownership and retention; the scripted
  host model supplies waiter absence.

The initial real-old-binary crash, surviving PID proof, old-row observation
and allowed control remain. All previous assertions remain. No sleep,
retry, passing timeout or binary barrier is added. Binary lines removed: 0.
Assertion lines removed: 0. The architecture's shown-by clause is updated
in the same commit. This is a static repair, not host-run evidence.

Validation: cargo fmt --check and cargo clippy --all-targets --locked --
-D warnings. Host e2e remains deferred to the landing gate as instructed.
