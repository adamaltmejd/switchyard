# Y-71 landing observer correction

The operator's focused host validation of 382e9d reports the conflict scenario
PASS. In the held-partial fixture, integration execution 5 was interrupted and
released, execution 6 published a candidate, selector 7 selected it, gate 8
passed, landing 9 landed, and cleanup 10 was removed and released. Exact
approval resolved. The test then waited for the nonexistent attempt.landed
event. At this repair request that native run was still live on its invalid
wait; its eventual failed terminal result must be retained, not accepted as a
passing scenario.

Reported outside evidence:
/home/yard/validation/y71-operator-20261015/382e9d-live-landed-wrong-event-diagnosis.json
and /home/yard/validation/y71-382e9d-integration-20261015-evidence/native-stores.

Disposition: replace the existing G2/G9 scenario's landing wait with
landing.recorded. ARCHITECTURE.md's audit list names that event, and the existing
selected-candidate scenario uses it after approval. Watch::event compares the
event name literally. The final exact approval-row assertion remains unchanged.
All other explicit event names in g09_integration match the same audit list:
attempt.ended, execution.started, execution.ended, execution.released and
attention.raised. No production event, behavior or spec change is needed.

This retains G2's C-only approval and landing observation after G9's held
partial-work restart. Existing sabotage comments name reusing A's approval or
candidate for C; the exact Git heads and final store assertion still reject
that sabotage. The expected event comes from the binding audit contract, heads
from host Git, and approved contents from the fixture. This corrects an existing
observer in its hardest-case restart scenario; it adds no assertion block,
scenario or twin and weakens no guarantee. Test lines removed: one invalid
wait, replaced with the supported event. Binary lines removed: zero. No operator
approval question is pending.

Validation: boxed cargo fmt --check, cargo clippy --all-targets --locked --
-D warnings, and cargo build --locked. No worker e2e run; native validation
remains operator-run.
