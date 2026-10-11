# Y-76 interrupted output oracle repair

## Observed host failure and source-only diagnosis

The operator reports that exact head `5a32f64` ran the full host suite:
59 passed, one failed, exit 101, 236.65 seconds. The sole failure was G7's
Claude interrupted execution 3. The execution record had input 1100,
output 0, null cost and partial harness token metadata. The assertion
expected output 100. No approval or landing occurred.

That record is not evidence that Claude emitted an output-100 update.
The provider fixture emits output 0 in `message_start` and 100 in
`message_delta`. The previous oracle used the provider's completion counter
for the interrupted record. Yard receives Claude stream-json instead.
The shared reader observes assistant snapshots and result totals. Source
inspection alone cannot establish whether the pinned harness emitted an
additional output report, or whether a later snapshot overwrote one.

The operator's log path is absent in this worker. `/yard/proof` is empty,
and no raw pinned frames from that host run are available here. A request
for those frames was made while the repair proceeded. No host e2e was run
in the worker. This diagnosis does not assert that Claude only emitted
zero, that the parser is correct, or that the repaired head passes on host.

## Assertion dispositions and Gate answers

Keep the one hardest common accounting scenario and all four phases.
Change only the interrupted output oracle. Its turn has exactly one
provider response before the held request; a new assertion enforces that
fixture precondition. The existing host wrapper saves every real stdout
line before any measurement-field transformation. Execution 3 has no
transformation. The test now reads those raw frames and prints them to
captured test output for inspection on failure or a host `--nocapture` run.

Inspect every numeric `output_tokens` report in the raw JSON, including
reports outside assistant snapshots. For this single response, compare
the greatest actually reported output count with the stored partial output.
This external observer does not duplicate Yard's message-id accumulation
or select the last assistant frame. A report of 100 followed by 0 still
expects 100 and exposes lost/overwritten usage. A report in another emitted
frame is also detected. If all actual reports are zero, the expectation is
the harness's explicit numeric zero, not the un-emitted provider delta.
No report fails the scenario's reported-output precondition; it cannot
silently turn this block into an unknown-output test.

This block serves G7's truthful optional usage and interrupted persistence
clause. Sabotages are ignoring a nonzero emitted update, overwriting it
with an earlier snapshot, inventing output from provider counters, or
discarding explicit-zero output. The test comment names missed/overwritten
updates. Expected output comes from unmodified real harness stdout, not
Yard output or an SDK frame invented by the fixture. Partial harness
metadata comes from G7. The output comparison remains an assertion on the
execution record; it is not replaced by a logging-only observation.

Keep the nonzero input/cache oracle from external fixture counters, failed
result and terminal-counter comparisons, replacement rather than addition,
no-result interruption, null versus explicit-zero cost, resume identities,
interrupted reopening equality and raw fresh/resumed estimate comparisons.
Those expressions are unchanged. Successful terminal output still must
equal the provider's nonzero completion counters. An always-zero output
reader therefore fails the terminal controls. The raw estimate baseline
still comes from the real harness; no guessed bill or pricing table was
added. The independent G8 API/login authentication controls are unchanged.

The hardest interrupted case remains a resumed multi-block tool response
with its next model request held, followed by real box teardown, stdout
drain, daemon reopening and further resume. Existing terminal-only Claude
observations and Pi/Codex scenarios cannot establish which output updates
this pinned Claude stream exposes before interruption. No scenario twin
was added. Production parsing is unchanged pending actual frame evidence;
no speculative handling of an invented frame type was added.

The binding spec clarifies that explicit-zero assistant output is a partial
observation and that provider counters not emitted by the harness are not
inferred. The full API/login contract and G7 Shown by clauses remain.
The operator's failure is addressed with an external raw-frame oracle;
exact-head host validation must establish the runtime explanation. Test
source lines removed: 6; added: 52. Binary lines removed: zero. No promise
or independently necessary assertion block was deleted.

## Boxed validation

`cargo fmt --check`, `cargo clippy --all-targets --locked -- -D warnings`
and `cargo build --locked` pass. No daemon, host e2e, sabotage run, public
model call or construction configuration change was made. Actual pinned
evidence, exact-head host validation and fresh review remain operator-owned.
