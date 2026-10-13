# Y140 gate consumer digest observer Gate

The operator reports Y144 source `9b98a707` landed through resolved execution
2811. Native 2815 passed 79 scenarios in 562.23 seconds, raw SHA256
`919f5a31d6540b51f122277f8d62ef6d8ee6fe6ce53a687dad63122bebcafc6b`.
The independent `original-gate-digests` mutation survived focused G2 with
exit 0 in 65.69 seconds. That survivor remains unqualified evidence despite
the full native pass. Six other mechanisms have direct detections or remain
in progress; this report claims no completed all-fault qualification.

Retained operator evidence:
`/home/yard/validation/y144-9b98a7-host-controls-20261013-evidence/original-gate-digests/`
contains `source.diff`, `native.log`, `terminal.json` and `operator-verdict.json`.
Earlier failed results and the survivor are not replaced by this correction.

## Disposition

Extend only the existing revision 5 gate-only adoption block in G2
`quiescent_plan_cas_retains_only_exact_node_and_producer_evidence`.
Read the prior gate digest from its existing execution snapshot. Reuse the
revision 5 execution query to read the new gate consumer digest, require a
string, and require it to differ. Preserve the exact one-node observation,
invocation counters, selective reuse, typed suppliers, stale no-write controls
and immutable history assertions. No new scenario or helper is introduced.

| Gate question | Answer |
| --- | --- |
| Guarantee and Shown by clause | G2 exact consumed gate binding: independent committed gate adoption changes its gate check consumer digest while retaining completed unrelated checks and immutable old evidence. The G2 row now names this observation. |
| Sabotage | `original-gate-digests`: make `executions::check_digest` use original admission for gate consumers instead of selected current admission. The test comment names this exact mutation. |
| Why existing coverage misses it; hardest case | The operator's native survivor demonstrates that actual command invocation, rerun identity and unchanged historical rows alone do not establish the new digest binding. Adoption follows completed checks on the same candidate with fixed proof, image and context, isolating the changed consumed command. |
| Outside oracle | The fixture changes the committed command from `original` to `adopted`; host Git supplies its source commit, and the actual host counter observes both commands. The spec's per-consumer digest contract requires unequal identities for this changed definition. The store supplies the two observed digests, not an expected digest calculation. No Yard hashing logic is copied. |

`jobs/mod.rs` supplies the ordinary command gate's digest to
`executions::start`. `store/executions.rs::start` passes that supplied digest
through `check_digest` and inserts the result into the execution row.
`check_digest` binds the selected consumed gate definition. Thus the observed
gate digest is populated. Implementation digests and optional image IDs are
not oracles for this assertion.

Static Gate disposition: keep the corrected block. No sweep tag applies.
No weakening or deletion is proposed to the operator. Test lines removed: 3.
Binary lines removed: 0. Product behavior, harness holds, deadlines and
concurrency are unchanged. This is a Gate review, not a suite sweep.

## Required qualification

Before commit, `cargo fmt --check`,
`cargo clippy --all-targets --locked -- -D warnings`, `cargo build --locked`
and `git diff --check` passed. Reviewer,
real host landing, the independent surviving mutation against this correction,
and restored G2 positive control remain required. No host e2e or sabotage
runs inside this box. A compile pass is not behavioral fault qualification.

Review1409 remains the latent current/recorded evidence carry-in. This fault
changes the check consumer digest source; it authorizes no speculative
historical evidence path or product change. Full ordinary and packaged Y140
parent scope and mandatory dependent packaged activation remain unchanged.
