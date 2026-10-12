# Y-102 held history observer correction

The operator's landing gate on 3f1758348cf4337c0da69d37cf3c0da7b0dba4c9
reported 74 passing scenarios and one failure. The selected Durable bounds
and publication scenario waited for a start event after its FIFO wait had
already consumed that event. The supplied history records execution 6
starting at sequence 47 and publishing at sequence 49. Execution 5 had
already stopped and released at sequence 44. This is a test observer defect,
not evidence of a supervision defect or a passing whole scenario.

Keep the G8/G13 assertion block and its strict release-before-start ordering.
Use the existing `Watch::find` to search consumed history as well as future
events. `Watch::said` stores events read during the real fixture hold in
`seen`; `Watch::until` only reads future events. The execution, attempt and
kind filters remain unchanged. No retry or new watcher mechanism is added.

Test-audit gate: this still shows the selected cancellation clause in G8/G13,
where checked owner disappearance precedes the subsequent selected execution.
The test comment still names releasing before controller/box disappearance
or admitting a successor while reserved as sabotage. The distinct hardest
case remains the selected controller's own held host-route response and
externally held real teardown. The expected sequence ordering comes from the
spec; the observer is real daemon status history, qualified by external box,
process and lock observations. This correction adds no scenario or duplicate
assertion and weakens no guarantee. Binary lines removed: 0. Test lines
removed: 1, replaced by the existing history-aware lookup.

Boxed `cargo fmt --check` and
`cargo clippy --all-targets --locked -- -D warnings` pass. Host e2e was not
rerun in this box and remains the operator's landing gate. The supplied
failure does not establish the later publication ownership control, which
the test had not yet reached.
