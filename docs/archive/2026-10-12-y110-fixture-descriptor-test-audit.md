# Y-110 held startup fixture descriptor correction

Candidate c9581f23a09e8c3faf99e89e802d76b551ca251f is not approved.
The operator's real host run stalled in execution 8 after capturing native
ready and exiting the outer wrapper. The store still showed running,
reserved 1 and a null handle. The actual native box and controller lived.
This is a fixture deadlock, not evidence of a failed production stop repair
or a passing ownership check.

The operator's descriptor evidence showed the background drain shell kept
Yard stdout pipe96315963 on saved fd10 and stderr pipe96315964 on fd11.
The feeding shell also kept Yard stderr on saved fd10. Scoped compound
shell redirections saved these descriptors for restoration. Yard could
not observe EOF while the intended descendants survived.

The original stalled result remains operator evidence at
/home/yard/validation/y110-c9581f-review-20261012/
operator-fixture-stdout-deadlock-evidence.json and operator-startup-control.log.
The command was cargo test -p e2e --locked
stopping_failed_preparation_keeps_ownership_and_prevents_the_turn -- --nocapture.
The earlier archive describes the earlier candidate; it is unchanged.

## Correction

The feeding and draining pipeline stages now use subshells with permanent
exec redirections before their FIFO holds. They need no restoration
descriptors for Yard's output streams. The feeding stage keeps its pipeline
stdout and redirects stderr. The draining stage redirects stdout and
stderr. No arbitrary inherited descriptor is closed, preserving the
execution lock. Native argv and stdin spec, native ready and diagnostics,
and the outer exit 73 remain unchanged. Production source is unchanged.

All original clone controls and assertions remain. So do the no-stop
startup error control, successful returned-owner pending-stop teardown,
capacity 1, exact reservation checks, and actual box and kernel-lock
observations. No deadline, containment or ownership proof changes.

## Test-audit gate

No assertion block is added, removed or weakened. The existing gate answers
in 2026-10-12-y110-box-startup-test-audit.md still apply:

1. G4/G5 retain capacity until supported ownership proof. The Work contract
   and G9 shown-by cover failed and successful preparation startup handoffs
   with acknowledged stop and no worker turn. Their scope is unchanged.
2. The source mutations remain propagating up_worker Err before honoring
   stop, discarding a successful returned LiveBox, and releasing on box
   absence alone. The test comment names each sabotage.
3. Clone completion cannot expose this distinct startup ownership case.
   The hardest case loses native ready while a real controller and box
   survive, then holds the inherited host lock after actual box absence.
   The same extended scenario and its observers remain; no twin is added.
4. Expected values remain from fixture execution order and exit codes,
   actual native replies and inventory, host PID and kernel lock checks,
   CLI acknowledgements and history, read-only store observations, and the
   host model's turn observations. The spec supplies stopped:unchanged.

The stalled run never reached execution 8's stopped/unchanged and diagnostic
assertions or red:ownership assertion. It also never reached box absence
with the inherited owner held, the two checked real inventory probes,
checked release 8, supported attention answers, successful pending-stop
startup and held teardown 9, or deliberate final start and approval 10.
None of those later blocks is claimed as validated by that run.

This is a scoped fixture audit. The diff removes 13 test source lines and
zero binary source lines. All removed lines are replaced fixture syntax
and formatting, with no removed assertions. The guarantee shown-by clauses
remain the intended host gate; no spec update is needed.

## Validation

cargo fmt --check and cargo clippy --all-targets --locked -- -D warnings
passed. Both resulting multiline shell fixtures were re-read and passed
sh -n after static placeholder expansion. No fixture runtime was simulated.

No e2e run was performed here, as instructed. Static checks are not runtime
validation. The operator host gate must prove the wrapper failure is
observed while actual native ownership survives, reach every later held
ownership point, and complete all assertions before landing.
