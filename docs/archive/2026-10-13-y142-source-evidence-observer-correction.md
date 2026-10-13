# Y-142 source-evidence observer correction and test-audit Gate

Retained host gate2717 failed with 78 passed and one failed in 551.48 seconds
on merged ref `fb518906c5d89428b948ba7898e05296c89e224b`. G2 reached the source
evidence block after actual shared-consumer reruns, then failed at line674.
The observer read current evidence at the response root. This correction
preserves that failure, result2691 and all previous archives unchanged.

`jobs/admit.rs::attempt_show` places `attempt.to_json()` under `attempt`.
`store/attempts.rs::to_json` places current resolved evidence under `plan`.
The test now reads `show["attempt"]["plan"]`. Product output and consumed-input
qualification are unchanged.

Sibling accesses were checked against the constructors. Current evidence's
`inputs` comes from `Loaded::inputs` and `read_sources`: image, definition
and staging commits, whole context tree, Dockerfile blob and selected staged
path/blob records are direct fields there. `attempt_show` exposes
`plan_revisions` at the response root; `attempts::revisions` returns evidence
objects whose `plan` contains the graph. Those history accesses remain valid.
The CLI fixture parses the returned JSON without moving those fields.

Test-audit Gate disposition: retain the existing G2 source-evidence block
and correct its observer, with no added scenario or assertion removed.

- Clause: G2's Shown by requires exact selected source blobs while the image
  commit and whole context stay fixed across separate staging adoptions.
- Sabotages: resolved overwrites adopted definitions, stale tool identity,
  stale skill identity, whole-panel invalidation and the completed-producer
  staging exception remain named runtime controls. The existing test comment
  names these mechanisms. No qualification exception is restored.
- Hardest case and distinction: this is the same quiescent boundary after
  completed gates and reviews. Independent gate-only and reviewer-only
  retention controls cannot establish correct shared staged-input evidence.
  There is no new twin, retry or scope partition.
- Oracles: source commits, selected blobs, Dockerfile blob and whole context
  come from host Git. Settings and bytes come from the fixture. Actual typed
  supplier, immutable-row, approval and external model/gate counter assertions
  all remain unchanged. The response constructor establishes where to observe
  evidence; it does not supply the expected source identities or counts.

Test source lines removed: 1, replaced by the corrected observer path.
Binary source lines removed: 0. No runtime seam, deadline, concurrency,
lifecycle or product contract change is introduced.

Validation: `cargo fmt --check` and
`cargo clippy --all-targets --locked -- -D warnings`. Host e2e and all five
named independent runtime qualification controls remain post-approval gates
and were not run here. Every original parent acceptance and the obligations
recorded in the preceding integrity correction remain mandatory; this
observer repair completes no parent or dependent runtime child.
