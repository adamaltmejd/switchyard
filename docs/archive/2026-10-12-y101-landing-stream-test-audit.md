# Y-101 landing stream repair and scoped test-audit

The operator's landing gate on 83249b9 passed 70 scenarios and failed both
existing packaged audit scenarios. This record is a source repair assessment,
not a fresh host runtime pass. The worker did not run host e2e.

The corrected-image hold watched history from sequence zero. Its reported
failure named execution 1, the earlier missing-asset refusal, while the hold
belonged to attempt 3. The watch now starts at the current status sequence
before the corrected execution is admitted. It still rejects fresh attention;
no refusal, publication or isolation assertion is removed.

The bound scenario published the fixture report but its completed control
remained ineligible. The supplied gate also showed a stream ending with exit
zero and no terminal frame. The native bridge previously used JavaScript
stdout writes from Pi's handled input hook. It now writes its session and
terminal JSON lines directly to descriptor 1, synchronously and with partial
writes handled, before returning to Pi. This removes dependence on Pi's
stdout handling and buffered delivery at print-mode exit. It does not infer
success from exit, publication or child text. The existing reader still
requires the native terminal frame and separately records child failure.
The successful-control assertion now prints the execution on failure.

The supplied gate does not include the successful control's captured stream
or failure detail. The operator's next host gate must verify this stream
repair; source inspection and Rust checks cannot establish that runtime pass.

## Scoped assertion gate

Both complete package scenarios are retained. Only the history starting
point, sabotage comments and failure diagnostic change. No new assertion
block or scenario is added. No assertion is removed or weakened. G8's
existing Shown by clauses cover these controls; the package contract now
states direct native frame delivery. G12/G13 isolation, exact identities,
native registration/session, secret routing, checked teardown and reservation
controls remain intact.

| Clause and disposition | Sabotage | Distinct hardest case | External oracle |
|---|---|---|---|
| G8 missing assets then corrected-image native publication: retain the complete scenario and bind its hold watch to fresh history. | Send selected native frames through Pi's JavaScript stdout handling; skip image checks or restore outer-model election. | Corrected execution follows deliberate missing-asset and traversal refusals in the same project; native publication precedes a real exec-wrapper hold and later replacement. | Fixture FIFO, status sequence and ticket/execution history, external fixture report, real boxed probes and Git blobs. |
| G8 bounded controller and bare-answer refusals with an eligible publication control: retain every assertion and add the failed control as diagnostic context. | Send the selected terminal frame through Pi's JavaScript stdout handling, remove the request bound or synthesize a report from terminal text. | Successful native report publication after request-bound and bare-answer refusals, without an outer Pi model turn to emit a terminal frame. Ordinary Pi publication scenarios do not exercise this handled input path. | Spec's 12-request bound, fixture model requests and report, captured real streams and CLI result eligibility after checked teardown. |

Binary source lines removed: 2. Test source lines removed: 4, replaced by
sabotage comments, a fresh history cursor and the diagnostic. No sleeps,
retries, public model, mock, binary barrier, timeout or concurrency masking
was added. Installed `.yard` and earlier archives remain untouched.

Y-100/Y-93 retain their complete integration acceptance. The dependent
selected-controller cancellation and failed-teardown child remains required.

## Validation

Boxed `cargo fmt --check`,
`cargo clippy --all-targets --locked -- -D warnings` and `git diff --check`
passed. Host e2e remains operator-owned after approval and was not run in
this attempt.
