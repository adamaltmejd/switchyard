# Y-143 source-only handoff Gate

This corrects candidate 1321751's G9 qualification. Its executed predecessor
and successor used different instructions. Equality of unexecuted intermediate
plans did not establish a source-only handoff. The earlier archive records that
candidate; this file records the correction.

The same `an_edit_reaches_the_next_prompt_and_stop_delivers_it_sooner` scenario
now adopts sources directly from revision 1 to revision 2. No profile or ticket
edit precedes those two executions. While the real host model route holds the
source successor, store rows establish both actual execution revisions and the
unchanged ticket revision. Their recorded plan payloads compare equal, including
instructions, topology and grants. The predecessor starts after the original
sole-capacity hold with original model, thinking level and staged marker.
The successor starts with adopted model, thinking level and staged marker.

A native Git diff observer requires deleted original model/blob identities and
added adopted model/blob/gate-definition identities in the same operator-labelled
diff. The same observer retains the original instruction-change attribution
control in the later phase. Only after the source successor's opening is observed
does the operator revise its profile and ticket. That held worker makes a real
`adopted.txt` commit. The third worker makes a distinct `renamed.txt` commit;
host Git observes both fixture literals. All gates run on the third candidate.
The separate project's stop and deliberate start controls stay unchanged.

Gate answers:

- G9: frozen startup and checked source-only successor handoff. Its shown-by
  clause now names actual equal-plan executions and the later instruction phase.
- Four independent binary sabotages must fail: return the original attempt
  implementer from `implementation_settings`; load the current attempt at held
  old startup; delete the consumed-definition map from `plan_handoff`; return an
  empty handoff when actual old/new plan objects are equal. The fourth must fail
  the source-only diff observer even though the later instruction diff survives.
  Deleting definitions and suppressing equal-plan handoffs are distinct bugs.
- The sole-capacity delay is the existing hardest startup race. G2's quiescent
  adoption and instruction-only G9 edits cannot catch an equal-plan handoff
  guard. No twin scenario is added. The previous intermediate-plan equality
  block is replaced with actual-execution binding, rather than retained as proof.
- Model/effort, system markers, file bytes and gate command are fixture literals.
  Blob and image identities come from host Git. Execution revisions are the
  admitted fixture sequence; expected ticket revision is the initial contract.
  Equal plans and preserved rows compare recorded outputs because those clauses
  promise agreement and immutability. The ended predecessor result, original
  admission, implementation settings and input rows and adopted revision row
  remain unchanged through the later phase. Checked predecessor release is
  observed while the source successor is held.

No promise is dropped. The instruction-only diff and stop controls remain.
The observer parser is shared between source and instruction phases; each call
observes a different handoff mechanism. No binary lines are changed.

Host e2e and all four named runtime sabotages remain post-approval gates.
No host suite or sabotage is run in this box, as the ticket instructs.
Y-140 integration and mandatory packaged activation remain outstanding;
this correction does not complete Y-137, Y-136 or Y-62.

Validation: `cargo fmt --check` and
`cargo clippy --all-targets --locked -- -D warnings` passed.
The test diff replaces 95 lines with 116 lines, including the shared observer
and moved controls. Binary lines removed or added: 0. No guarantee assertion
was dropped; the invalid intermediate-plan proof was replaced.
