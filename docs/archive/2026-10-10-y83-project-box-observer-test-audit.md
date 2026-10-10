# Y-83 project box observer audit, 2026-10-10

The operator reports that the valid full host run at initial candidate
400f717 passed 43 scenarios and failed racing_workers_share_one_execution_slot.
The run took 191.66 seconds; its log is
/tmp/switchyard-y83-host-after-space.log. The presence-only project label
selected five boxes across concurrent fixtures and construction workers.
That observation exceeded this fixture project's scope. This worker has
not rerun the host suite or inspected construction state.

Keep the G4 race assertion block. It shows the existing shared-slot clause:
two admitted tickets race at limit one, only one worker acts, and the other
reaches approval after release. Find the held box by the fixture store's
recorded handle, then obtain its project label independently from pinfold.
Count boxes with that exact label. This follows the existing G5 observer.
The store-to-box identity comparison remains, as does the expected count
of one. Unrelated projects remain free to run concurrently.

Test-audit gate:

- Guarantee: G4's two-ticket shared-slot race. Its binding shown-by clause
  now names the project-scoped pinfold observation.
- Sabotage: reserve after spawn or count only attempt lanes, as the test's
  comment already names. Either permits both racing workers to act.
- Distinction: the attempt-lane race cannot expose an execution limit below
  max_lanes. The held worker and concurrently admitted waiter remain the
  hard case; this observer repair adds no scenario or twin.
- External expectations: the project's configured limit supplies one;
  pinfold supplies the label and actual boxes; the scripted host model
  supplies the independently held turn and request count. The execution
  handle only anchors the explicit row-to-box identity comparison. No
  project-key hashing logic is copied from Yard.

No existing assertion or guarantee is removed or weakened. No tests are
serialized, no retry is added, and no unrelated work is stopped. Binary
lines removed: 0. Assertion lines removed: 0. One observer assignment is
replaced by the scoped selection.

Validation: cargo fmt --check and cargo clippy --all-targets --locked --
-D warnings. Host e2e remains the landing gate, per the ticket instruction.
