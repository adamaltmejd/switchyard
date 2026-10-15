# Y-158 fresh body observer test audit

The operator reported landing3521 at bbd627f: 93 passed, 1 failed in
1268.41 seconds. The raw failure is retained externally in
B/approved-y158-bbd627-gate-capture. The reported p.sqlite records execution
4 with adopted-model, ticket revision 2, plan revision 3 and NULL resumed.
Its prompt contains the full Rename it too body, Revised implementation,
and the labelled plan diff. This candidate does not alter that evidence.

Changed only the invalid resumed-only body observer in G9's
an_edit_reaches_the_next_prompt_and_stop_delivers_it_sooner. Work's contract
in ARCHITECTURE.md requires the whole body for a fresh session and a body
diff for a resumed session. The fixture explicitly changes the consumed
instruction/profile definition before this launch. It must expect a fresh
whole-body line, not +Rename it too. The expected body comes from the
fixture's ticket edit; the observer is the request received by the host
model. Matching the complete line rejects a diff-only delivery.

Gate answers:

- G9's edit-delivery clause is the existing scenario. This correction
  follows the explicit fresh-session contract after changed definitions.
- Sabotage: omit the body from the fresh prompt or deliver only a resumed
  diff. The test comment names both; the corrected assertion fails.
- The hard case remains an edit while a real worker is held, followed by
  productive work before any gate. The plan-diff assertion observes the
  separate definition handoff; it cannot prove ticket-body delivery.
  The q project retains its stopped/resumed body-diff control.
- The expected whole body is the literal fixture edit. Settings and Git
  blob expectations, actual plan-diff content and attribution, frozen rows,
  candidate Git observations and stopped/resumed controls are unchanged.

No scenario, retry, binary change or session-compatibility change was added.
No coverage clause or guarantee was removed. The operator's announced
test-surface reduction and coverage/policy reconciliation are separate work.
The operator list is empty. Test lines removed: 6 (one assertion replaced
and five comment lines revised). Binary lines removed: 0.

Validation: cargo fmt --check and cargo clippy --all-targets --locked --
-D warnings. No e2e suite ran in the candidate box; host qualification
remains the operator's landing gate.
