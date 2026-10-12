# Y-108 read-only resolution repair

The operator reported host validation of candidate
571a3f839920e59e0f2d000d8222a2cdefab5b70 on 2026-10-12.
The unchanged live published-review teardown and held implementation/edit
scenarios passed. The unchanged
`a_plan_revision_hands_off_a_selected_read_only_implementer` failed with
cargo exit 101 after 5.08 seconds. Its revision call received
daemon/disconnected instead of plan_revision 2. The daemon panicked at
plan.rs:355, indexing a seat absent from the admitted definition view.
This failed result stands. Other controls after that failure, host sabotage
and the full suite have not validated that candidate.

Operator evidence is in
`/home/yard/validation/y108-571a3f-review-20261012`:
`operator-focused-controls-result.json` and
`operator-control-a_plan_revision_hands_off_a_selected_read_only_implementer.log`.
That host directory is unavailable in this clone; this record preserves
the operator's supplied result, not an independent replay.

Loading an admitted read-only plan retains its selected workflow and seat
definitions. Its seat map is empty. Other source workflows remain available
for matching, including an unused default that names an ordinary seat.
Resolution normalizes against those workflows before runnable validation.
The default's direct seat index therefore panicked even though the selected
read-only workflow was legitimate.

Workflow translation now returns its existing unknown-seat error rather
than panicking. Normalization and runnable matching already skip translation
errors. The selected workflow still requires exact ordinary plan matching;
a missing seat in a selected workflow cannot become runnable. No admitted
definition, worker grant, lifecycle, fixture or assertion changes.
ARCHITECTURE.md's live read-only handoff contract remains unchanged.

The existing real host scenario is the acceptance check for this repair.
It must still show refused old publication, unchanged old execution inputs
and results, the successor's revision diff and successful replacement report.
Local validation uses cargo fmt --check and locked all-targets clippy.
Host e2e remains the post-approval landing gate; it was not run in this clone.
