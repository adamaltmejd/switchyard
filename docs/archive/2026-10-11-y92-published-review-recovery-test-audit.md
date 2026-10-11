# Y-92 published-review ownership recovery repair, 2026-10-11

The operator carries this bounded repair within Y-92. Proposals 205/206
were rejected as a separate prerequisite route. This new record supersedes
that proposed route only; earlier records and archive bytes are unchanged.
The complete package and integration scope remains required.

## Evidence and diagnosis

Exact combined source `6e0c6211a9b81ac133ad33dd2fafc6fbfc0d7d7a` passed
focused package tests (2/0, 306.57 seconds), standalone full suite
(72/0, 743.83 seconds), and review 1735/check 912. Approval 99's actual
landing 1747/host 1751 instead failed (70/2, 1165.64 seconds). Approval
was retired; canonical remains `e4ce7d57e5537679f78125a00f46a3ab4dc7d725`.
Those earlier greens do not establish actual landing acceptance.

The operator preserved live external evidence for Y-1 attempt 2, review
execution 4, node review-0. It published check 1/pass. Seq 32 raised
red:ownership, seq 33 released the execution, and seq 34 ended it
interrupted. Real CLI attempt show proved eligible=true, teardown=true,
reserved=false, and the exact existing publication/pass/blocked[] result
with interrupted infrastructure and original implementer-3 input.
Check 1/execution 4 remains pass and there is no second review. Its
ownership attention 2 remains open and approval never appears.

Source agrees: capacity::reap proves release and calls executions::end,
while jobs::runnable refuses the retained open red item. The repair is in
the existing end transaction. Only an interrupted review with its recorded
lock contract, durable registration, publication and eligible result can
resolve red:ownership on that exact execution. Eligibility is read through
executions::get, using the existing authoritative predicate. Release still
requires real box absence and inherited lock freedom. No owner, lifecycle,
lease, duplicate result predicate or dependency is added. Legacy ownership,
absent/invalid publication and unrelated attention retain their refusals.
The implemented G5/G8 contract and shown-by clauses are updated together.

## Test-audit Gate

Keep and extend a_seat_killed_after_publishing_has_published. No new
scenario is added and no original assertion is removed. This is G5's
retained inherited owner across restart and G8's same-execution interrupted
publication settlement after checked teardown.

1. The new blocks show retained reservation, null/ineligible result,
   unchecked teardown, no approval/gate/landing successor before owner
   release, then exact ownership resolution and original approval.
2. Sabotage: leave the exact ownership item open after proof and settlement;
   the original approval expectation fails. Omit inherited locking or
   release without proof; the external lock probe or retained-state block
   fails. The test comment names this sabotage.
3. The existing held-publication crash now forces the late recovery branch
   deterministically. G4's old Git child holds an implementation before any
   publication; it cannot expose a settled review blocked by obsolete
   attention. G8's held blocking publication tests repair routing after
   restart, without retaining this host owner. Neither subsumes this case.
4. Expected pass/empty findings and second-publication refusal come from
   the scripted model. The wrapper reports its own PID and actual box
   argument; external store observation associates that box with the real
   review. CLI execution/binding/publication identities must agree across
   restart. OS kill -0 and a read-only opened file's try_lock prove the
   delegated process and inherited lock remain held after daemon death,
   before restart. Expected retained/eligible states and reconciled
   resolution come from G5/G8, not copied Yard code.

The pinfold PATH wrapper delegates the same real command, arguments,
stdout/stderr and exit status. It holds its completion on a fixture-owned
FIFO while the real Pi child registers and publishes against the host
scripted model. No harness is replaced and no model frame is fabricated.
The writer is held open by the fixture so dropping it frees the holder on
failure. After restart, history names ownership for the exact review;
external lock and box-list probes prove the remaining owner while the box
is gone. Only then does the fixture release the FIFO, observe checked
reservation release, and retain the original approval, single review,
exact result/binding/check and quick-ticket controls. No sleeps, retries,
private launch-input writes, binary flags or changed deadlines are used.
Test lines removed: zero. Binary lines removed: zero.

## Independent gate-error trace

Keep a_gate_errors_start_reruns_that_gate and all its assertions unchanged.
The historical failure block is absent from the frozen 8000-byte tail,
and its fixture was removed by normal Drop. No cause is invented.
Source was traced separately: box_gate returns an infrastructure error
when real box up fails; gate ends error without judgment output or check
and raises red:gate. answer_start resolves the named item and leaves
implementation unchanged so the same-head gate reruns. Landing-child
infrastructure error ends without a judgment; the parent records error
and raises red:landing without returning repair input. Supported start
requeues under the same approval. Abandoned gate settlement ends abandoned
before recording a check or raising attention. Capacity proof remains
required along these paths. No demonstrated second defect is repaired.
The operator will capture this scenario independently on the repaired head;
a later green cannot establish its historical cause.

## Validation and remaining gates

No e2e, host package installation, host Node/client, image build or boxed
controller execution was run in this candidate. Package code, exact pins,
sealed closure, CI provisioning, native bridge, Y-98 production and tests,
repository .yard and old archives are unchanged. All 15 guarantee rows
remain; only G5/G8 receive the explicit recovery clause.
Fresh exact-head recovery, both package cases, independent gate-error
capture, full 72 suite, published review and actual landing remain required.
Y-93/parent Y-64 and release through Y-80 remain required; Y-81 is deferred.

Candidate checks passed: cargo fmt --check,
cargo clippy --all-targets --locked -- -D warnings, cargo build --locked,
and git diff --check. Protected guarantee rows other than the explicit
G5/G8 additions byte-match the preceding head; all 15 rows are retained.
No test assertion or binary line was removed. No new dependency is needed:
the fixture's lock observer uses the pinned Rust standard library.
