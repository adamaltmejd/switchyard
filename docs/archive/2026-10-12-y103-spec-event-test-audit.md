# Y-103 landing gate: G7 event-list correction

The operator's host landing gate reported 74 passing tests and one failure
on candidate a87130de9cbdf3f7d75c063924aa2d17a0566c98. The failing G7 block
rejected `landing.prepared`. Y-103 had added that event and
`landing.continued` to ARCHITECTURE.md's Store contract but omitted them
from the test's fixed list. The new continuation scenarios and the extended
G8 same-approval control passed in that reported run.

## Test-audit disposition

Retain and correct the closed-event-list block in
`g07_lands_end_to_end::a_ticket_lands_end_to_end_and_leaves_only_rows`.
Add both specified landing events. Add its missing sabotage comment.
No assertion or guarantee is removed or weakened.

- Guarantee and clause: G7 says every event is one the Store section names.
  The existing spec already names both events; no spec change is needed.
- Sabotage: emit an unlisted handle or usage audit event on completion.
  The closed-list assertion still rejects it.
- Distinct mechanism: this existing block checks the entire audit stream
  after landing and cleanup while another attempt remains live. The
  continuation blocks check prepared identity and selected child evidence,
  not the closed audit-event vocabulary. No new scenario is added.
- Observer and expected values: the observer reads the real store file.
  The expected names come from ARCHITECTURE.md's Store event list, outside
  the binary. The correction preserves the same assertion and observer.

Test lines removed: 0. Binary lines removed: 0. No binary change.
Worker validation: `cargo fmt --check` and
`cargo clippy --all-targets --locked -- -D warnings`.
Host e2e and sabotage execution remain at the approved operator landing
gate; neither runs in the worker. No successor daemon was operated and no
`.yard` or construction state was changed.
