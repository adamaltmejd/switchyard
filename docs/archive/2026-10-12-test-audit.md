# Y-103 prepared landing continuation: test-audit gate

The audit covers the changed assertion blocks in
`g03_landing_cas.rs` and `g08_review_is_a_publication.rs`.
This is a ticket gate, not a release whole-suite sweep.
No existing guarantee or positive control was removed.

## Dispositions and the four gate answers

- Extend G8's existing `a_gate_errors_start_reruns_that_gate` control.
  A host landing child passes before the boxed child has an infrastructure
  error. The supported same-approval retry lands the exact pinned head and
  selects the original passing child. Its complete row stays unchanged.
  The failed child stays an error with no judgment and reruns.
  Sabotage: regenerate the merge or replay the passing child. The old block
  checked only merge parents and could accept a different merge commit.
  Host Git supplies the pinned head and parents. The fixture's successful
  host command supplies the expected pass. The retained store snapshot is
  the expected value for the promised before/after equality.

- Keep `prepared_input_and_completed_child_survive_host_child_restart`
  as distinct G3 recovery coverage. The second host gate holds on a FIFO.
  Before restarting, host Git proves the prepared ref and unchanged target,
  the store proves no intent, and the host proves a live group and held
  inherited lock. Restart kills that recorded group. Continuation retains
  the completed first child and reruns the interrupted second child on the
  exact prepared head. Sabotage: regenerate the prepared merge, replay the
  first child, or promote the interrupted child. G8's normal infrastructure
  error does not exercise reconciliation. Host Git, the fixture's head log,
  the host process/lock probes and the retained store snapshot supply the
  expected values. The fixture declares the second command's allowed pass.

- Keep `target_advance_retires_prepared_input_and_requires_fresh_children`
  as distinct G2 invalidation coverage. It uses the same held-child crash
  point, then advances canonical with host Git before restarting. The old
  result stays truthful but is absent from the final selection. Both gates
  run on the fresh head. Sabotage: reuse prepared input or child evidence
  across the changed target. Existing CAS coverage moves the target after
  intent; this block moves it before continuation admission. Host Git
  supplies the new target, and the host head log observes fresh verification.
  The unchanged-input scenario is its separately runnable retained-evidence
  control; this project itself supplies successful fresh verification.

- Keep `prepared_input_waits_for_its_surviving_git_owner` as distinct G3
  ownership coverage. A real final prepared-ref Git read completes and its
  wrapper holds before returning to the daemon. Before restart, the host
  proves the surviving process, inherited lock, prepared ref and unchanged
  canonical target; the store proves no intent and a retained reservation.
  Restart reports ownership on that execution and creates no successor.
  After releasing the command, history shows checked release and supported
  start lands the exact pinned head in the same project. Sabotage: consume
  preparation before teardown or regenerate it afterwards. The existing
  held update-ref block has an open mutation intent; this one has none.
  Host Git and host process/lock probes supply the expected identities.

- Keep the existing CAS block, default/reduced/empty selection controls and
  the G8 unstarted-child ownership control. They cover different boundaries.
  CAS sabotage remains recording a landing after the expected-old mismatch.
  The new continuation path still uses that same final CAS.

No new block waits out a timeout, asserts reason prose, or adds a binary
barrier. No test imports Yard internals. Fixtures use the existing separate
Machine state, socket and runtime. No successor daemon was installed or
operated and no construction state was opened.
The shared fixture supplies different author/committer dates to the first
and later real merge creations. A regeneration sabotage therefore produces
a different head even if both commands run in the same second. This supplies
Git input; it does not fake a command result or add a binary test path.

## Validation boundary

Worker checks: `cargo fmt --check` and
`cargo clippy --all-targets --locked -- -D warnings`.
Host e2e and sabotage execution are reserved for the approved operator
landing gate, as required by Y-103. They were not run in the worker.

The changed test files remove 6 lines while the binary files remove 7;
these are replacements, not deleted assertion blocks. No control was weakened.
Y-103 adds no merged failure/skipped acceptance API and does not complete Y-96.
