# Y-158 admission repair test audit

Integrity review1783 identified standalone report admission accepting ordinary
agent selected inputs without freezing or staging them. This shape now refuses
as standalone_selected_inputs, with the selected field and paths, before an
execution is created. No alternate staging path was added.

G6's existing standalone scenario gains one refusal block. It submits an
existing Markdown skill and, independently, a nonexistent reference. Expected
input identities come from those literal requests. Execution and audit rows
must remain equal to their pre-call snapshots; the host model receives no
request. The existing unselected report succeeds in the same project and
remains the allowed control. Removing either selection guard makes its call
succeed, failing the block. Candidate-output and worker-authority refusals
exercise different guards and cannot detect ignored selected report inputs.
No existing assertion or acceptance was removed.

G13's selected ordinary worker scenario now uses the supported planning
lifecycle from c7b8556. The real planner publishes a report. Status history
waits for its ended attempt, cleanup and checked cleanup release before
explicit writable admission. There is no parked start or unpark race.
The returned writable attempt identity scopes all session and verification
queries. Model openings exclude the preliminary report. All selected byte,
mount, admission, adoption and fresh-session assertions remain. The ordinary
round counter starts only for writable workers.

The operator list is empty. No guarantee was weakened. The spec states the
standalone restriction and G6's new observation. No crash, sleep, retry,
timeout wait, test-only binary path or fixture barrier was added.

Binary lines removed: 0. Test lines removed: 7, replacing the parked admission
and unscoped observations without removing an assertion block.

Validation is boxed cargo fmt --check, cargo clippy --all-targets --locked --
-D warnings and cargo build --locked. Native host e2e qualification remains
the operator's landing gate; it was not run in the candidate.
