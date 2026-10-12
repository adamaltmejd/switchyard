# Y-104 integration and explicit merged repair test audit

Assessment base: `f9ab66930e31f8be808583fac6e702b5814c546e`.
This attempt integrates actual implementations, not parked proposals.
Y-96 and Y-61 retain their complete integration contracts. Y-80 release
scope is unchanged and Y-81 remains parked.

## Actual dependency landings

| Dependency | Actual landed head | Operator evidence |
|---|---|---|
| Y-103 | `4f78ea237c721a904b4696784e36615a07b67af3` | 2026-10-12-y103-operator-validation.md: review1017, landing1934, host1938, 75 scenarios passed. No independent sabotage claimed. |
| Y-105 | `b9429b066e6acb1a0e442ec70c0cfff568f25a4c` | 2026-10-12-y105-operator-validation.md: review2090, landing2093, host2097, 75 scenarios passed; focused predecessor ownership control and sabotage. Explicit selected merged repair remained outstanding. |
| Y-106 | `ca66d4b6cc2dad65ff7bf9990608b6b0643f3284` | 2026-10-12-y106-operator-validation.md: review2128, landing2129, host2133, 75 scenarios passed; focused omission control and predecessor ownership sabotage. |

These records establish actual landings on Linux x86_64 with the operator's
existing isolated validation profile. Their host runs precede this attempt.
They do not execute this changed repair control or its sabotage. Prior
failed gates remain failed evidence. No archive record was rewritten.

## Clause-to-evidence assessment

The source references below name current mechanisms. Runtime references are
inherited operator records, not new worker runtime observations.

| Original clause | Current source and owning control | Evidence limit |
|---|---|---|
| Trusted pre-admission selection; default fail/repair is the control | plan::ordinary_selection admits operator handling only on retained candidate, review and landing nodes. G1 default/reduced/empty and G8 selected_failures_continue_without_rewriting_judgments retain ordinary controls. | Inherited Y-105/Y-106 suite evidence; unchanged earlier blocks. |
| Exact candidate base/head/proof, target/old/merged, revisions, original landing, node/child/check/result, actor and nonempty text | queue::failed_evidence and admit::candidate_answer bind and reread full evidence. attempts::resolve writes the existing decision and audit; checks::decision is the sole decision reader. | Source plus inherited acceptance controls. New repair answer uses these same named identities; its host execution is unrun. |
| Failures stay failed; acceptance continues remaining gates on pinned input | queue::accepted_child requires exact failed evidence and an accept decision. G3 held remaining-child scenarios and G8 compare complete failed snapshots, host Git and remaining child heads. | Inherited Y-105 suite evidence. |
| Explicit eligible merged repair follows truthful return; fresh successful path | queue::land reads repair, calls returned, retires approval and schedules implementation. This change carries its complete evidence into existing bound feedback. G8 now holds the real repair turn, observes no canonical move or intent, then requires fresh verification, approval and remaining merged gates. | New source and compiled scenario only. Exact-head host control and sabotage are required. |
| Named admitted omissions bind pinned input without child/check/pass; ordinary omission has no decision | queue::skipped_evidence, candidate_answer and final omission reread use the existing attention resolution. G2 selected_skipped_decisions_bind_identity_without_manufacturing_results preserves skipped/empty and ordinary controls. | Inherited Y-106 focused and full suite evidence. |
| Delayed answers or changed target/ref/evidence are stale, exact expected/current, with no writes; fresh merge inherits nothing | candidate_answer locks the target, probes target and original ref, checks current admission and complete evidence transactionally. G2/G3 bracket stale calls with decision/audit snapshots and obtain fresh acceptance in the same project. | Inherited dependency evidence. No new claim for every possible changed field mutation. |
| Restart retains decisions and evidence, including held remaining child | queue::history/prove_predecessors qualifies original preparation and retained children. G3 proves pinned Git, store, host process and inherited lock before killing at its held host child. | Inherited Y-103/Y-105 controls; new repair feedback persistence has not been independently run. |
| Infrastructure error, unknown liveness, intent or unchecked teardown cannot be accepted | failed_evidence requires an eligible completed failed judgment. candidate_answer proves predecessors; queue final consumption and capacity ownership retain checked release. G8 infrastructure refusal retains its successful same-input retry; G3/G2 ownership controls remain. | Inherited Y-105/Y-106 controls and named ownership sabotage. |
| Protected paths still need trusted approval; workers cannot select/accept/approve/land | queue::holds enforces auto protected-path policy; operator tools remain on trusted stdio, outside worker grants. G1 worker authority and protected-path controls remain unchanged. | Inherited suite and source; no grant change. |
| Daemon-only writer, one execution lifecycle, shared capacity and target serialization | Existing daemon transactions, execution rows, capacity::effect and target_lock remain the path for answers, continuation and return. No new lifecycle, store, lease or recovery path. G4/G5/G10 remain unchanged. | Source plus inherited suite; no operational changes. |
| Final CAS and canonical-based intent recovery | queue final admit rereads selection and input, persists intent, calls update_ref with the old head, and recovers from canonical. Existing G3 final Git-owner and CAS scenarios remain unchanged. | Inherited suite evidence. Independent exact-head CAS sabotage remains required by the parent. |

## Integration change

The selected repair branch previously passed only a detail string to returned.
advance started its implementation without bound feedback. The new execution
therefore did not retain the complete named failed evidence in its inputs.
This is a source observation, not a CLI-reproduced defect claim.

returned now carries that evidence into the existing pending repair and
advance binds it through start_implementation's existing feedback seam.
The feedback names the original child and node, and holds the complete failed
check, result, merged inputs and original landing identity. It is historical
input, not another decision store. Ordinary red/conflict returns carry no
new evidence. The branch comes from the attempt's frozen target, already
required by admission. No schema, dependency, worker grant or exception changed.

## Scoped test-audit gate

Disposition: extend G8's existing selected-failures scenario in the same
project. G2/G3/G8 Shown by clauses now name explicit merged repair. No test,
assertion or promise was removed. This is a scoped gate, not a whole-suite
cleanup sweep.

The new assertion blocks answer the four gate questions:

1. G8's explicit selected merged repair clause owns the return, bound feedback
   and fresh successful path. G2 owns unchanged original failure rows; G3 owns
   no consumption of failed input and the fresh canonical landing.
2. Sabotage advance by replacing next.get("feedback") with None. The held
   repair still starts, but its exact feedback source/evidence assertions fail.
   This is a direct assertion failure, not a timeout. Treating repair as
   acceptance would instead select remaining landing work and move canonical;
   the event observer checks repair versus landing before waiting on the latch.
3. Candidate repair and automatic landing repair never read this merged repair
   decision. Acceptance does not retire approval or bind repair feedback.
   The harder case advances target after candidate verification, fails only on
   the merged input and holds the real resumed implementer before repair acts.
   All earlier acceptance, restart, stale/no-write and infrastructure blocks stay.
4. Fixture commands supply failed/pass verdicts, failure output and repaired
   files. Host Git supplies old, pinned merged and fresh candidate heads.
   Supported operator MCP supplies the explicit decision; CLI and store expose
   execution inputs, retired approval and absent mutation intent. Comparing
   original snapshots and carried evidence establishes promised agreement,
   rather than deriving success expectations from Yard's output.

The new latch is a real local model reply holding Pi in its real box. No
restart is added here; existing held restart controls remain intact. No sleep,
retry, public endpoint, deadline/concurrency change or binary barrier was added.
Test lines added/removed: 176/2. Binary lines added/removed: 31/15.
Test removals are fixture edits, not removed assertion blocks. Binary removals
replace argument plumbing and pending repair construction.

## Validation and remaining gates

Worker `cargo fmt --check` and
`cargo clippy --all-targets --locked -- -D warnings` passed on the final source.
Clippy compiled the changed e2e target; this is not runtime evidence. No worker
host e2e, sabotage, development daemon or operational runtime/configuration
change is authorized or claimed. No .yard file changed.

The operator must run this complete G8 scenario unchanged at the exact candidate
head, demonstrate the feedback-drop sabotage and restore the exact source,
then run the approved actual merged host gate. Published review and actual
landing remain required. The original failure-rewrite, changed-identity
inheritance, unchecked-consumption and CAS sabotage obligations remain;
inherited ownership experiments cover only their stated mechanisms.
This attempt does not declare Y-104, Y-96 or Y-61 satisfied.
