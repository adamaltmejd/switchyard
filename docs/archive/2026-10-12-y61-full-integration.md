# Y-61 full operator-decision integration

Assessment base: `62cbd859766b679acf222b8262970bee89dc2168`.
Scope: the complete original Y-61 contract, the phase-5 contract in
`2026-10-09-switchyard-redesign-plan.md` and the operator's partition.
The chain Y-60 -> Y-94 -> Y-95 -> Y-96 has actually landed. This assessment
does not substitute for approval, actual landing or ticket completion.

## Landed inputs and external evidence

Every head below is an ancestor of the assessment base, checked with Git.
The linked records distinguish failed candidates from actual landings.

| Ticket | Actual landed head | Operator-recorded host gate |
| --- | --- | --- |
| Y-60 ordinary graph | `e9e68ca546a46483b7ed969827d80860c16f4e34` | [Validation](2026-10-11-operator-y60-validation.md): landing1548, host1552, 62/0; resolved intent, cleanup1553 removed. |
| Y-94 reduced verification | `da5e3ccb99f81f59a8aecfa464279eb95300cf99` | [Validation](2026-10-11-operator-y94-validation.md): landing1601, host1605, 63/0; resolved intent, cleanup1606 removed. |
| Y-95 candidate decisions | `14f0dcb2d358c589737fd5aab6ffd785e2cf4f77` | [Validation](2026-10-12-y95-operator-validation.md): landing1866, host1870, 72/0; resolved intent, cleanup1876 removed. |
| Y-103 prepared continuation | `4f78ea237c721a904b4696784e36615a07b67af3` | [Validation](2026-10-12-y103-operator-validation.md): landing1934, host1938, 75/0; resolved intent, cleanup1943 removed. |
| Y-105 merged failure acceptance | `b9429b066e6acb1a0e442ec70c0cfff568f25a4c` | [Validation](2026-10-12-y105-operator-validation.md): landing2093, host2097, 75/0; resolved intent, cleanup2103 removed. |
| Y-106 merged omission decisions | `ca66d4b6cc2dad65ff7bf9990608b6b0643f3284` | [Validation](2026-10-12-y106-operator-validation.md): landing2129, host2133, 75/0; resolved intent, cleanup2134 removed. |
| Y-104 combined decisions and explicit repair | `5dc2ce99509ba84242b834632e6e76b81859ab8e` | [Validation](2026-10-12-y104-operator-validation.md): landing2145, host2149, 75/0; resolved intent, cleanup2150 removed. |
| Y-96 full merged integration | `84c13600214453a84a1bd299cde90b6b46a61915` | [Validation](2026-10-12-y96-operator-validation.md): review2160/check1126, landing2161, host2165, 75/0 in 555.79s; resolved intent, cleanup2166 removed. |

Git reports no difference between either the actual Y-104 or Y-96 head and
this base in crates, Cargo.toml, Cargo.lock, ARCHITECTURE.md, README.md,
AGENTS.md or .yard. Intervening changes are dated records. Thus the latest
actual full host gate exercises the implementations and assertions read
here together. This candidate adds only this record.

Runtime observations below are inherited from committed operator records.
Their raw host logs and private validation checkouts are not mounted here.
No independent host run or CLI reproduction occurred in this worker.

## Complete original contract

Source paths are relative to `crates/yard/src`. Scenario names are in
`crates/e2e/tests/guarantees`. Host2165 ran all 75 scenarios on the unchanged
combined implementation. The table maps the original contract, including
its safeguards, rather than treating child ticket completion as evidence.

| Original clause | Actual mechanism read | External scenario and observation |
| --- | --- | --- |
| Trusted operator selects reduced/no review and candidate/landing checks through the resolved plan | `plan.rs::ordinary_linear` validates retained ordinary definitions, phase order and complete visitation; `ordinary_routes` rewires retained panel predecessors. `jobs/admit.rs::admit` persists the selected plan, skipped nodes and unreviewed status. The full ordinary chain remains default. | G1 `operator_reduced_verification_stays_skipped_and_queue_authorized` lands full, reduced and empty workflows in one project. The reduced panel omits its middle seat, carries the first block through its passing final seat and repairs once. CLI/store observers compare fixture node sequences. |
| Only the queue lands an operator-authorized candidate; selected verification may be empty | `jobs/mod.rs::advance` derives approval from selected eligible evidence and live policy. `admit.rs::attempt_approve` binds it. `queue.rs::holds` revalidates it; `land` prepares and records actual Git input even with no selected children. | The same G1 scenario advances the target for each selection and compares merged parents/tree with host Git. Empty selection records resolved old/merged intent and no gate/check rows. G2's omission scenario also lands ordinary reduced/empty controls without explicit decisions. |
| Explicit acceptance names failed candidate gates, blocking seats or accumulated blocks and continues remaining selected checks | `advance` reads admitted `operator_failure` and exact `checks::decision`; acceptance changes routing, preserving immutable judgment. `candidate_answer` resolves the existing attention with nonempty text. `limit_checks` names retained checks for ordinary approve-at-limit. | G8 `selected_failures_continue_without_rewriting_judgments` first lands ordinary repair/limit approval, then accepts a failed gate and blocking seat. Remaining gate and final seat run; the final passing publication retains its blocked result. A separate accumulated-block decision names its original failed check. |
| Explicit acceptance names skipped results without manufacturing passes | `admit.rs::skipped_decision` binds candidate omissions and full skipped selection to approval, with actor/text and candidate/revision/plan/proof. `queue.rs::skipped_evidence` binds requested landing omissions to original preparation through existing attention. Ordinary omission records no explicit acceptance. | G2 `selected_skipped_decisions_bind_identity_without_manufacturing_results` lands a full control, retains a real candidate gate, accepts named candidate and landing omissions, and observes no omitted execution/check/result. An explicit empty landing exposes its prepared decision before intent; empty panels stay unreviewed through landing/reopen. |
| Selected merged-tree failures may be explicitly accepted; every remaining selected landing requirement still runs | `queue.rs::failed_evidence` requires selected operator handling, completed infrastructure, failed judgment and exact original preparation. `accepted_child` accepts only passing or exactly accepted eligible evidence. `land` follows every retained complete-route node and rereads children before intent. | G8 selected-failures observes an accepted failed first landing gate followed by a passing remaining gate on host Git's pinned merge. G3 `prepared_input_and_completed_child_survive_host_child_restart` retains completed passing and accepted failed children, reruns the interrupted remaining child and lands the same exact prepared head. |
| Failed/skipped records remain truthful before and after landing/restart | Decisions resolve attention or write approval; `checks.rs::decision` is the sole failure-decision reading. No decision updates a check verdict or execution result. Omissions have no judgment. | G8 compares original failed gate/review and complete merged check/result/input snapshots through canonical landing and quiescent reopen. G3 compares complete failed rows across held-child restart. G2 compares retained candidate judgments, skipped decisions, absent omitted rows and unreviewed admission across landing/reopen. |
| Actor, nonempty text and exact candidate/ticket/plan/check/node/execution/result identities | `candidate_answer` compares supplied bindings, current attention, candidate and revisions, then rereads complete eligible evidence in the daemon transaction. Attention payload names operator; resolution audit retains text and attention identity. Approval stores candidate skipped actor/text and audits accepted evidence. Merged decisions additionally bind approval, original landing, child, old target and pinned merge; omissions bind no child/result. | G2 omission decisions compare fixture text/actor and exact identity. G8 refuses another plan and a preceding gate check on fresh review attention, with exact reason and no audit; corrected decisions succeed. G3 compares prepared identities to host Git and complete failed evidence snapshots. |
| Stale acceptance names the changed identity and writes nothing | `candidate_answer`, `skipped_decision`, `stale_part` and `selected_checks_current` compare current identities before resolution. Merged answers hold the target lock and probe both canonical target and prepared ref. New preparation inherits no answer. | G2 `delayed_candidate_acceptance_names_the_changed_identity_and_writes_nothing` holds an edited worker before head change, refuses old revision, then old head, with unchanged audit. The omission case adds wrong plan, attention, retained node and omitted/mismatched required proof refusals. G3 `target_advance_retires_prepared_input_and_requires_fresh_children` and G2 merged omissions refuse old target with exact expected/current Git heads and unchanged attention/audit, then accept fresh evidence. |
| Infrastructure uncertainty cannot become test failure or proof of teardown | `checks::current` requires eligible ended/unreserved checked-teardown results. `failed_evidence` excludes infrastructure errors. `prove_predecessors` checks all parent/child owners, box inventory, inherited host locks, current lock contract and absent open intent. | G8 `a_gate_errors_start_reruns_that_gate` induces real box-up errors with null judgment, no output/check/repair and unchanged canonical. Acceptance refuses by attention identity without audit; supported same-input retry succeeds under the same approval. G8 held publication has no acceptance before teardown. G3 held final parent ownership refuses acceptance until release; G2 omission parent-lock refusal has the same-project released control. |
| Landing uses CAS and actual merged evidence, including empty selection | `land` pins preparation, verifies selected children/omission decisions, persists exact old/merged intent and calls `update_ref` with old target. `record_landing` consumes only the open intent. `reconcile::decide_intent` waits for checked release and reads canonical. | G1 compares empty/reduced merged parents and tree with host Git. G3 `a_landing_retires_when_canonical_moves_before_update_ref` observes retired first and landed second rows. `a_landing_whose_update_ref_is_held_stays_undecided_until_released` holds real Git ownership across restart, retains undecided intent and records feature/a once after release. |
| Protected paths require trusted operator authority under auto; workers cannot approve/accept/select/land | `advance` and `queue::holds` read current protected paths and auto/manual policy. Trusted operator MCP forwards to the socket. `mcp.rs` lists only execution-scoped worker tools and refuses others. Selection adds no grant. | G1 empty protected AGENTS.md work under auto waits for operator approval. G1 `a_worker_cannot_land_by_push_rpc_or_canonical` preserves push/socket/canonical refusals and queue control. G7 operator MCP covers initialized worker-session operator-tool refusal and trusted stdio control. G2 live protected/manual policy withdrawal preserves judgments while withdrawing auto authority. |
| One execution lifecycle, single daemon writer, checked teardown, shared capacity, containment and target serialization | Children/continuations use existing `executions::start/end`, reservations and `capacity::effect`; decisions use daemon `project.tx`. `queue::next` treats running/reserved owners and open intent as target ownership; a landing question holds later approvals on its target. No added lease, owner, receipt, store or recovery lifecycle. Worker/box grants stay unchanged. | G3 held-host-child and surviving-Git-owner cases expose process groups, locks, reservations and absent intent before restart. Full host2165 retains G4 shared-capacity, G5 unknown-owner/intent, G10 target serialization and G13/G14 launch/containment controls. These are inherited observations, not new dedicated decision/concurrency experiments. |
| Preserve explicit repair, exact identity and intent recovery | `candidate_answer` distinguishes repair from accept. `queue::returned` retires approval and binds exact failed evidence through existing pending feedback to a new implementation. Fresh candidate verification/approval is required. | The complete G8 selected-failures case holds real Pi repair, observes retired approval, original child/complete feedback, unchanged failure and no canonical mutation/intent, then fresh selected checks and successful landing. Y-104's focused feedback-drop mutation fails directly. Existing G3 intent/CAS cases remain in the full gate. |
| Revise threat model and G1/G2/G3/G8 in implementation commits; preserve exclusions | Current One merge path, Trusted verification selection and Exact identity controls agree with plan, admission, attention, verification identities and Git sections and all four guarantee rows. README states selected failure/omission behavior. These changes landed with the implementation slices. | Source/spec comparison finds no future feature claimed by this candidate. No living contract or assertion changes here. Mandatory policy exceptions, unknown-liveness bypass and worker landing privileges remain excluded. |

## Assertion assessment and limits

The test-audit skill was read for this scoped assessment. This is not the
release sweep. No assertion, guarantee row or Shown by clause changed.
Test lines added/removed: 0/0. Binary lines added/removed: 0/0.
No proposal is needed: no concrete remaining integration defect was found.

The mapped mechanisms are distinct. G1 owns selected scheduling and actual
empty-merge evidence; G2 owns delayed identity and explicit omission
decisions; G3 owns held ownership, restart, changed prepared input and CAS;
G8 owns failure routing, immutable judgments and repair feedback. Their
shared failure snapshots observe different boundaries. No twin was added.
Expected nodes/text/verdicts come from fixtures, merged parents/tree from
host Git, ownership from real processes/boxes/locks, and immutable/no-write
claims from external store snapshots. Refusals have same-project controls.
Crashes hold real model, host-command or Git calls and prove outside state.

The requested sabotages are named in existing tests: rewrite failed evidence
to pass (G8/G3), or run hidden omitted landing checks (G1). Those independent
mutations were not run here, and the operator records do not claim they ran.
Recorded focused mutations instead prove answer-time ownership (Y-105 and
Y-106), feedback retention and CAS (Y-104). Their direct failures and restored
source identities remain the evidence for those specific experiments only.

Several narrower branches remain source-only: independently corrupting the
pinned prepared ref, replacing an admitted immutable plan, both invalid-child
guards noted in Y-103 and timeout classification. They are not new runtime
claims or missing observed original acceptance scenarios. No clock test is
introduced. Earlier failed gates/controls and unresolved image-cache/native
listing diagnoses stay failed or unexplained; later greens do not explain them.
Inherited Linux x86_64 FUSE-profile evidence proves neither native-overlay,
Linux arm64 nor Mac behavior. Host2165 exceeds the historical five-minute
budget; no scenario was deleted or weakened. Release budget revision remains.

## Candidate gates and remaining operator work

Worker `cargo fmt --check` and
`cargo clippy --all-targets --locked -- -D warnings` both passed before
committing this record. They are static checks, not runtime evidence.
No worker `cargo test -p e2e` ran.

Before approval, the operator must read this full original-clause map and
compare the concrete candidate head/proof/diff and relevant inputs with the
recorded validated heads. Unchanged focused experiments may be reused for
their stated mechanisms; changed relevant inputs need fresh evidence.
Publish fresh exact-head review, then approve through the supported queue.
Actual prepared merged-tree fmt/clippy/build and the full real host e2e gate
remain mandatory. Record actual canonical landing, resolved intent and checked
cleanup before consuming it and completing Y-61. This worker records none of
those future gates as complete and requests no manual completion override.

This candidate changes no .yard file, installed configuration, model,
reviewer or frozen daemon. It starts no development binary or daemon and
opens no construction backlog. Operator validation uses separate
state/socket/project/runtime names. Original downstream dependencies stay
on Y-61. Full redesign/release scope through Y-80, including release cleanup,
test-audit sweep and platform validation, remains required. Y-81 stays deferred.
