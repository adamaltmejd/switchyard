# Y-97 landed target integration and test audit — 2026-10-11

Merged the operator's actual canonical target
`870ba9d1876fccde2a79783633114823169555e5` into this attempt's candidate
`993fa1cb9a6a24e5db558df3b0c3bf07a26bcb2d` through worker Git. The public
incremental bundle came from the ticket body, decoded in scratch space.
Its 28649 bytes have SHA256
`e010347f6e579a5b13c0c1a3a040d6b7cd4f93da3ef71063f0f6997905a63671`.
Git verified both prerequisite objects and the advertised HEAD before merge.
No operational store, daemon, host installation or repository `.yard` file
was changed. No runtime or model selection was changed.

The operator reports Y-91 actually landed as
`e65730bc84ab99974e65966b32d3c181751524a3`, landing.recorded 5700,
with host gate 1647 passing 67/0 in 325.40 seconds and cleanup 1648 removed.
The later operator evidence commit is the imported target above. These are
historical operator observations, not validation of this combined candidate.
Y-92 packaging is separate; no packaged runtime selection is claimed here.

## Integration resolution

The concrete conflicts were the schema migration tail and G8 guarantee row.
The landed execution.report migration stays immediately after the existing
node/inputs/result/teardown entry. The unchanged Y-97 attention-kind rebuild
follows it. Source comparison confirms every landed migration entry remains
an exact prefix, and the attention migration is unchanged. A store already
at the landed report version therefore executes the attention migration;
an older store executes both in order. Existing daemon writer ownership,
transactional migration/version updates and foreign-key enforcement remain.
The attention copy preserves IDs, payloads, resolutions, timestamps and
history references, its closed allowed kinds, foreign keys and open index.
No isolated candidate store is promoted or manually rewritten.

The combined G8 row retains the report, legacy continuation refusal,
publication binding/event, supersession, teardown and successful control
clauses together with selected omission and exact acceptance coverage.
Its historical upgrade clause keeps Y-97's fuller preservation observations
and the existing approve-at-limit landing. A source comparison of all fifteen
guarantee rows against both parents found their coverage retained. The G2
acceptance identity and unchanged-failure clauses remain intact.

Read the combined report validator, capability grants, implementer
settlement, admission/start and readiness guards, execution result binding,
candidate decision path and approval/queue check revalidation. The report
column is retained, read-only publication still needs immutable admission,
and unbound recovery remains abandonment followed by fresh admission.
Acceptance still reads exact eligible evidence and persists a decision
without rewriting checks or results. Landing still revalidates selected
identities and live policy under existing ownership requirements.

## Test-audit gate

Keep all existing assertion blocks. The only fixture changes made during
integration beyond the carried parent changes are two Y-97 planner closures in G2's delayed
acceptance and G8's selected-failure scenario: use planned(request) to call
the real yard_publish_report tool rather than return bare terminal text.
The helper publishes the literal fixture report on the opening model turn
and terminates after the tool reply. It supplies no frozen binding itself.
Planner completion, cleanup and reservation-release observations remain.
The real historical-binary upgrade fixture intentionally keeps its legacy
proposal behavior, including the open and resolved preservation controls.

- G2 retains revision and head stale refusals with expected/current values
  and unchanged audit rows, followed by the successful repair and landing.
  Expected revisions come from the fixture briefs; heads and ancestry come
  from host Git. Sabotage remains omission of exact identity comparisons.
  A held edit is the hard case. Existing approval blocks do not subsume
  this persisted candidate decision.
- G8 retains default bounded repair and approve-at-limit, selected failed
  gate and blocking review acceptance, remaining checks, final carried block,
  final approval, held publication/restart and unchanged judgment comparisons
  through landing and reopen. Named plan/result refusals and infrastructure
  refusal with successful same-project controls remain. Fixture node order,
  verdicts and explicit decisions supply expectations; store equality is the
  promised immutability observer; Git observes actual landing. Sabotage
  remains rewriting fail/block to pass, ignoring selected result identity,
  skipping checks or consuming publication before teardown. These accepted
  failures cannot be established by default repair alone.
- Landed G8 report tests and Y-94 omission/CAS controls remain. No report
  assertion or test helper contract changes. Migration preservation still
  uses a real older daemon and read-only row/audit observers across two
  reopens, followed by successful proposal acceptance and limit landing.
  No fixture modifies a schema, and no second upgrade scenario is added.

This is a focused integration gate, not a whole-suite sweep. No assertion,
control or guarantee clause was removed or weakened; no twin scenario,
sleep, retry, clock guarantee, mock or binary barrier was introduced.
Test assertion lines removed: 0. Binary lines removed by conflict resolution: 0.
Existing archive records are preserved; this record is new.

## Validation

`cargo fmt --check`, `cargo clippy --all-targets --locked -- -D warnings`
and `git diff --check` pass. Clippy compiles the combined e2e targets.

Fresh exact combined focused G2/G8, infrastructure and upgrade tests,
the full host suite, fresh published review and actual landing gates remain
required. No e2e or sabotage ran inside this candidate. Old isolated source
or host results do not establish combined correctness. Ticket dependencies
and unparked status remain unchanged.
