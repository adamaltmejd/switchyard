# Y-141 blocked-review corrections

The review found three defects in the Y-141 candidate.

P0: skipping the unused ordinary prompt computation passed an empty string
to PackagedAudit::argv. It delegates to Pi::argv, whose check_prompt rejects
empty input before any box starts. The packaged branch now supplies a fixed
nonempty placeholder. Pi's native input hook reads audit.json before model
dispatch, so the controller still receives the one real bounded labelled
handoff there. No duplicate plan_handoff computation was restored.

P2: replacement instruction checks used contains, which could match the
new instructions in the handoff despite stale instructions opening the
controller prompt. All three held replacements now require the prompt to
start with the declared new instructions and their following newline.
The initial controller explicitly starts with the declared old instructions.
The separate handoff model/blob/context assertions remain intact.

P3: LATER_SELECTED_SKILL and LATER_REFERENCE were no longer written by the
fixture. Their negative assertions could not fail and have been removed.
The written hostile/unselected markers and independent phase-selected
old/new skill and reference checks remain.

## Test-audit Gate

- Row and clause: G2's held single-node packaged independent adoption
  clause observes actual selected instructions and revision handoff.
  G6 retains selected committed bytes, and G8/G12 retain explicit bounded
  publication and selected instruction authority. No spec clause changes.
- Sabotage: launch the replacement controller with predecessor instructions
  while retaining the current handoff. The independently held host request
  now fails its prefix assertion. The test comment names the fault.
- Distinctness and hardest case: strengthen the existing settings-only,
  staging-only and image-only replacements after a held publication.
  A contains assertion, model body assertion or recorded settings row
  cannot distinguish stale initial instructions from the new instruction
  text embedded in the diff. No scenario or twin was added.
- External expectations: old/new instruction prefixes are the exact host
  fixture values committed in the selected profile, with the newline from
  the documented launch composition. Host phases still choose the holds
  independently of model, instructions and staged values.

Disposition: strengthen the three instruction assertions, add their initial
positive control, and delete the two unwritten negative markers as directed
by the review. No remaining assertion or guarantee was weakened. This diff
replaces seven test lines beside one binary line; two of the test lines
are the dead markers. Prior archive records remain unchanged.

Validation: cargo fmt --check and cargo clippy --all-targets --locked --
-D warnings pass. The launch correction is verified statically against
Pi's prompt validation and native input hook. Host e2e and independent
runtime faults were not run in this box and remain post-approval gates.
Full packaged and parent integration obligations remain open.
