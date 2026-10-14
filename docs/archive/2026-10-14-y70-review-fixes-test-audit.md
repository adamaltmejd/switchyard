# Y-70 review fixes assertion audit

Keep the additions to G2/G8's five-producer selection scenario. No existing
assertion or guarantee is removed. The two review findings expose distinct
failures after publication and after landing admission.

The first picker publishes B, then receives a nonretryable HTTP 400 from the
scripted host model. The real Pi harness must end in failure. Store and history
observers require ordinary red:review attention, retained raw publication, no
selection output, no native gate or approval, and only one picker execution
before explicit attention start. That supported exit succeeds in the same
project. Historical row comparisons retain the failed publication afterward.
The expected candidate comes from the fixture's declared B input; the error
comes from its host response. Sabotage: suppress failure whenever publication
exists. Ordinary reviewer publication tests intentionally retain verdicts after
runner failure and cannot expose this selector-specific eligibility failure.

The final landing targets feature/a while main remains the configured default.
A host Git wrapper holds the real update-ref before running it. Outside the
binary, the FIFO proves the child is held, the store proves its open intent,
and canonical Git proves the target has not moved. Replacement of B must refuse
naming that intent without changing attempts, approvals or implementation rows.
Release permits the selected candidate to land. The earlier replacement of B
is the allowed control in the same project. Expected heads and content come
from Git and fixture bytes. Sabotage: remove the transactional intent fence
from start_writer. Existing intent tests fence edits and plan changes, while
the original selection scenario replaces B only before an intent exists.

Test assertion lines removed: 0. Binary guarantees removed: 0. Five producer
concurrency, same frozen reviewer, immutable history and selected-only native
checks remain asserted. No sleep, mock, runtime stub or binary barrier is added.
The wrapper runs the real Git command and only controls when it proceeds.
Host qualification is deferred to the normal landing gate, as instructed;
fmt, clippy and build are the candidate's local checks.
