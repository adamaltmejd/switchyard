# Y-156 host admission correction, 2026-10-15

The operator's focused host run of candidate
`9e81c6c897f5d00a8b93f8c9259402070b01cee2` failed before handoff admission:
0/1 in 32.75 seconds (44.92 seconds wall). The fixture selected both graph
and workflow, correctly refused as `exclusive_selection`. Remove only the
workflow field, matching the existing graph-only settled-planner admission
in G4. Keep the explicit three-role graph and the planner cleanup/release wait.
Product admission and scheduling are unchanged.

Audit of other selection calls in this scenario: initial ticket creation
selects only the planner workflow; the competing start names only the existing
workspace; rejection and approval name the candidate head and target. None
combines graph, preset or workflow selections.

Test-audit disposition: retain the same integrated G2/G9/G13 survivor.
The four gate answers remain: lost work, concurrent ownership, unauthorized
publication/landing and stale judgment justify the fixture; its existing
sabotage comment names those failures; no extra scenario, box, model turn,
retry, deadline or barrier is added; fixture bytes, Git, filesystem inodes,
host model requests and externally observed execution/check identities supply
the expected values. Every retention, authority, ownership, selected-settings,
fresh-session, producer/judgment and native-landing assertion is preserved.
Test lines removed: 1 (replaced). Binary lines removed: 0.

Host evidence is at
`/home/yard/validation/y156-9e81c6-handoff-20261015-evidence/`:
`native.log`, `terminal.json` and `native-stores`. That host path is absent
from this candidate box and was not modified. The operator will rerun the
entire focused scenario on the repaired candidate before full prepared native
qualification. No e2e run was made inside the box. Candidate checks are
formatting and all-target Clippy with warnings denied.
