# Y-92 candidate completion fixture correction, 2026-10-11

## Exact-head host failure

The operator's focused real host run 6862/test 377836 at
1391ece57328c452515a2f39f00a1654cdb04d52 ended exit 101: 0 passed,
1 failed, 71 filtered, 14.62 seconds. At line 1730, watch.said(down-said)
received approval seq 19 with second review check 2. The down FIFO never
signaled. The original normal registration flow completed; none of the
new ownership controls ran. This is a fixture seam failure, not evidence
that completed-candidate production reconciliation failed. The observer
launch found the test PID already gone and refused; no observer evidence
is claimed. The operator retains the full log at
/tmp/switchyard-y92-1391-noreg-host.log outside this candidate.

Normal supervise teardown calls LiveBox::down in crates/yard/src/box.rs.
It closes the real box-up child's stdin and waits for termination. It does
not call the distinct Pinfold::down CLI control. The previous wrapper's
box-down condition could never hold normal implementation completion.

## Correction and preserved contract

The hold now follows the first real box-exec invocation for implementation
box 1. The wrapper delegates installed pinfold with unchanged arguments,
stdout, stderr and status, then signals the real box name and exit code
through the fixture FIFO and waits for explicit release before returning.
run_harness waits for that child's streams and exit. The wrapper still
inherits the actual effect lock and holds the implementation before its
completion and ordinary teardown. A fixture-owned marker makes this hold
one-shot: subsequent real inspection exec calls proceed normally.

The read-only external store associates the signaled actual box with the
implementation handle before release. The existing OS-lock conflict
assertion proves that inherited owner remains held. The fixture removes
only its ordinary listing executable link while this real boundary is
held, removes the one-shot marker, then releases the wrapper. Subsequent
candidate completion encounters actual listing-executable unavailability.
All current same-inode, held real listing, reservation, ownership,
ineligibility, no-review, exact resolution, unchanged result/bindings and
allowed review continuation assertions remain. The original two real
review checks and second-only approval assertions remain unchanged.

This record corrects the hold description in
2026-10-11-y92-completed-candidate-ownership-test-audit.md: its statements
about a real down-command hold were intended fixture behavior, disproved
by this host run. That historical archive remains immutable. No previous
source validation is represented as successful host coverage.

Production reconciliation, binding ARCHITECTURE.md and all 15 guarantee
rows are unchanged. Existing gate-error, completed-cleanup, unstarted-child
and published-review recovery controls are unchanged. Package source,
Node/npm pins, closure, CI, native bridge, imported Y-97/Y-98 controls,
old archive bytes and repository .yard are unchanged. No dependency added.
No prompt selector, fake liveness result, private input/store write,
harness/model substitution, scenario, assertion deletion, binary hook,
sleep, retry, deadline/thread change or runtime kill/restart/prune is added.

## Test-audit Gate

1. G5 shown-by requires completed candidate ownership refusal through
   real listing failure and restored box absence with the inherited lock
   held; G8 requires automatic continuation of the same candidate into
   the original unregistered/registered review control. The changed FIFO
   observation and handle/lock assertions establish the actual held owner
   before the existing refusal and allowed-control blocks.
2. Existing named sabotage remains: omit candidate ownership resolution
   and the exact resolution/approval control fails; use box absence alone
   and held-lock reservation/no-review assertions fail; omit registration
   eligibility and the original second-only approval assertion fails.
   This correction makes those controls reachable at a real invoked child
   boundary. No timeout is evidence or a passing result. No sabotage/e2e
   execution is performed inside the candidate.
3. This extends the same hardest unregistered-publication scenario, with
   no twin. Interrupted registered review and completed cleanup cannot
   expose the already-successful candidate's pre-review blocked question.
   There is no crash; the FIFO holds a real delegated child's completion
   until fixture release, independent of image-store overload.
4. FIFO box name and status come from the actual installed pinfold
   invocation; expected zero is real successful Pi execution, not a
   manufactured frame. SQLite supplies the external daemon handle for
   association. The OS lock conflict and current descriptor/path inode
   equality prove the actual inherited file. All existing subsequent
   external CLI/store observations and fixture review-check oracles are
   retained. The fixture controls the unavailable/restored executable
   and explicit FIFO release; it never manufactures box-list output.

Disposition: correct the existing seam, retain every assertion. Eleven
test lines are replaced (names, condition and hold comment); zero binary
lines or assertion blocks removed. The first-review registration filter
still delegates real Pi, discards only its proof line as the original
sabotage, and leaves the second review untouched.

## Validation limits

Candidate checks passed: cargo fmt --check, cargo clippy --all-targets
--locked -- -D warnings, cargo build --locked and git diff --check.
No host e2e runs here. Fresh changed focus, gate-error,
published-review and both package controls, full unchanged 72-test host
suite, exact-head published review and successful actual landing remain
required. Prior historical failures and uncertain first-probe/runtime
causes remain. This fixture correction claims no runtime-delay fix.
Y-93, parent Y-64 and release through Y-80 remain required; Y-81 is parked.
Prior 923 missing-path P3 remains uncorrected. No push, publication,
deployment, frozen operational daemon/reviewer/model change or new proposal.
