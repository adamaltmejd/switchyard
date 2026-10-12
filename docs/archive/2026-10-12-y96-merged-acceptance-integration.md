# Y-96 merged acceptance integration

Assessment base: `71f93eddbcd744ff7cb95f5ac8b8bf10a3772d9a`.
The complete Y-96 revision 2 body and phase-5 acceptance contract were read.
Both required implementation children have actually landed. Their heads are
ancestors of this base. This record maps the original contract to the landed
source and operator-recorded host observations. It does not close Y-96.

## Actual implementation and evidence

| Work | Actual landed head | Recorded external validation |
|---|---|---|
| Y-95 candidate decisions | `14f0dcb2d358c589737fd5aab6ffd785e2cf4f77` | [Operator record](2026-10-12-y95-operator-validation.md): landing 1866, host 1870, 72 scenarios passed. |
| Y-103 exact prepared continuation | `4f78ea237c721a904b4696784e36615a07b67af3` | [Operator record](2026-10-12-y103-operator-validation.md): review 1017, landing 1934, host 1938, 75 scenarios passed. |
| Y-105 selected failed merged decisions | `b9429b066e6acb1a0e442ec70c0cfff568f25a4c` | [Operator record](2026-10-12-y105-operator-validation.md): review 2090, landing 2093, host 2097, 75 scenarios passed; focused failed-input ownership control and sabotage. |
| Y-106 named merged omissions | `ca66d4b6cc2dad65ff7bf9990608b6b0643f3284` | [Operator record](2026-10-12-y106-operator-validation.md): review 2128, landing 2129, host 2133, 75 scenarios passed; focused omission control and ownership sabotage. |
| Y-104 integrated decisions and explicit repair | `5dc2ce99509ba84242b834632e6e76b81859ab8e` | [Operator record](2026-10-12-y104-operator-validation.md): review 2144, landing 2145, host 2149, 75 scenarios passed; focused complete G8 repair control, feedback-drop sabotage, G3 CAS control and sabotage. |

Each record names actual canonical landing, resolved intent and removed
cleanup. Y-104's host gate ran the complete candidate with all dependencies
present. Its source, tests, ARCHITECTURE.md and README.md are identical to
this assessment base. The only intervening change is its dated validation
record. This was checked with Git, not inferred from ticket status.

The host observations below come from those committed operator records.
Their raw host logs and private validation directories are not mounted in
this worker. No new runtime observation or independent reproduction is
claimed here. Earlier failed controls and gates remain failed evidence.

## Full original contract

Source paths below are relative to `crates/yard/src`. Scenario names refer
to the real host e2e guarantee files. Y-104 host 2149 passed their combined
landed source. Focused experiments establish only the sabotages they name.

| Original clause | Landed mechanism | Observed scenario and observer |
|---|---|---|
| Trusted operator; selected eligible failure or named omission; no mandatory exception | `jobs/admit.rs::candidate_answer` requires named attention and nonempty text. `jobs/queue.rs::failed_evidence` requires admitted operator handling and eligible completed failed judgment. `skipped_evidence` reads the approval's named omission request. | G8 `selected_failures_continue_without_rewriting_judgments` exercises ordinary repair before selected acceptance. G2 `selected_skipped_decisions_bind_identity_without_manufacturing_results` exercises explicit omissions and ordinary omission controls. CLI, operator MCP and store expose the decisions. |
| Reuse candidate decision contract; bind target old head, merged head, landing, child, node and result | `failed_evidence` includes the full check, result, inputs and original preparation. `candidate_answer` compares each supplied binding, current candidate/revisions/proof, active approval and complete reread evidence. `store/checks.rs::decision` matches the exact evidence in resolved attention. | G3 held-child scenarios compare prepared data to host Git and complete failed check/result/input snapshots. Their delayed old answer names expected old target and current advanced target, with unchanged attention and audit snapshots. G8 verifies a late duplicate after restart is stale by attempt identity. |
| Accept explicitly, then continue all remaining selected merged requirements and queue landing | `queue::next` serializes candidate questions on their target. Acceptance resolves attention; a new landing follows its existing `resumed` chain. `accepted_child` consumes only passing or exactly accepted failed evidence. The admitted complete route supplies remaining gates. | G8 selected-failures observes failed first and passing remaining landing gates on host Git's pinned head. G3 unchanged restart retains completed children and reruns the interrupted remaining child on that same head. |
| Preserve failed/skipped facts and actual evidence; no fabricated pass | Acceptance changes attention, not check verdict or execution result. Omissions have no child, check or judgment. Ordinary omission has no explicit decision. | G8 and G3 compare complete original failed rows through landing/reopen. G2 checks absent omitted rows and unchanged decisions, skipped selection and unreviewed panel through restart. Fixture commands supply the actual fail/pass expectations. |
| Same-project full default and reduced/empty successful controls | Ordinary selection is immutable admitted plan data. The queue prepares actual Git input and uses CAS even with no selected gate. | G1 `operator_reduced_verification_stays_skipped_and_queue_authorized` lands full, reduced and empty workflows in one project. Host Git supplies exact merge parents and tree. G2 omissions scenario also lands full/default, ordinary reduced, ordinary empty and explicit empty controls. |
| Restart preserves decisions, checked teardown and exact merged identity | `queue::history` follows existing execution provenance. Original `prepared` and pinned ref remain unchanged. `prove_predecessors` proves every parent and child ended, unreserved, under the current lock contract, without open intent, live box or inherited host lock. | G3 `prepared_input_and_completed_child_survive_host_child_restart` holds a real host child, proves process group, inherited lock, target, pinned ref and absent intent before restarting. After release it compares retained passing and failed evidence and exact landed head. |
| Changed target or merged input requires fresh evidence/acceptance; delayed answer is stale and writes nothing | Answer-time Git probes compare target and original pinned ref under the target lock. Continuation retires changed preparation. Final consumption rereads the ref, omissions and selected evidence. Fresh preparation gets a new original landing identity. | G3 `target_advance_retires_prepared_input_and_requires_fresh_children` advances canonical before restarting a held child; fresh children and acceptance are required, old child IDs are excluded and delayed acceptance writes neither decision nor audit. G2 does the same for omissions across held continuation/restart. A separately corrupted pinned ref is guarded in source, not independently exercised by these controls. |
| Unknown liveness, infrastructure error and undecided intent cannot be accepted as verification failure | `failed_evidence` requires completed infrastructure plus failed judgment. `prove_predecessors` refuses unknown inventory or retained ownership and open intent. `queue::next` treats reservations and unresolved intent as target ownership. | G8 `a_gate_errors_start_reruns_that_gate` observes real box-up infrastructure error, no judgment/check/repair and a stale acceptance with no audit, then successful same-prepared-input retry under the same approval. G3 held parent ownership and held final Git-owner controls require release before continuation. G3 held update-ref recovery leaves intent undecided until its owner releases. |
| Protected paths require trusted authority under auto; workers cannot select, accept, approve or land | `queue::holds` rereads current protected paths and manual/auto policy. Operator tools use the trusted socket surface; worker grants do not include them. | G1 full/reduced/empty control holds protected AGENTS.md work at operator approval under auto. G1 worker push/RPC/canonical refusal and trusted operator controls remain. G2 live policy withdrawal controls exercise current policy without replaying unchanged judgments. |
| One execution lifecycle, single daemon writer, shared capacity and target serialization | Continuations and children use `executions::start/end`, existing reservations and `capacity::effect`. Decisions use daemon `project.tx` and existing attention resolution. Target locks and queue ownership remain shared with sync and intent recovery. No feature store, lease or recovery lifecycle was added. | G3 ownership controls expose reservations and inherited locks. The full actual-candidate host gate also retains G4/G5 capacity/unknown-owner and G10 per-target serialization controls. No changed assertion or new concurrency setting is used to mask these controls. |
| Keep CAS and canonical-based intent recovery | Final admission persists exact intent then calls `git.update_ref` with old target. Mismatch retires the landing. `reconcile::decide_intent` waits for checked release and decides from canonical old/merged ancestry. `record_landing` consumes one open intent transactionally. | G3 `a_landing_retires_when_canonical_moves_before_update_ref` observes retired first and landed second rows. Y-104's focused sabotage falsely records the mismatch and fails this assertion. `a_landing_whose_update_ref_is_held_stays_undecided_until_released` covers restart with a surviving Git owner and exact target intent. |
| Eligible explicit repair remains distinct from acceptance | The same exact answer resolves repair. `queue::returned` retires approval and carries complete failed evidence into existing pending feedback; `jobs/mod.rs::advance` binds it to the next implementation. Fresh checks and approval are required. | Y-104's focused complete G8 scenario holds real Pi repair, observes original child/evidence feedback, no canonical mutation or intent, retired approval and unchanged failure; release takes fresh verification and successful landing. Dropping feedback fails directly. |
| Update G1/G2/G3/G8 and threat model with implementation | Current threat controls One merge path, Trusted verification selection and Exact identity describe explicit exact merged decisions. Verification identity, execution continuation, attention, Git and guarantee rows state the same contract. These changes landed with the implementation children. | Read current ARCHITECTURE.md against queue, admission and decision source. This integration introduces no design change, so it requires no new living-contract wording. |
| No .yard change, worker host e2e, operational switchover, model/reviewer change, new timeout or concurrency masking | This candidate adds only this dated record. It starts no development daemon and opens no construction store. | Git diff limits this candidate to documentation. Worker commands are fmt and locked all-targets clippy. Host runtime evidence remains the operator's isolated Linux x86_64 profile. |

The source and evidence review found no integration defect. No CLI
reproduction was attempted in this worker. No implementation change or
follow-up defect ticket is claimed. The Y-104
repair runtime gap recorded in its earlier integration assessment is now
covered by its later exact-head control and actual full landing gate.
Historical storage/cache diagnostics are not explained by that passing gate.

## Scoped test-audit assessment

This is a contract-to-scenario assessment, not a release sweep. No test or
assertion changed. No guarantee or Shown by clause was removed or weakened.
Test lines added/removed: 0/0. Binary lines added/removed: 0/0.

G1 owns authorized queue/default/reduced/empty and protected controls.
G2 owns named identities, omissions and stale/no-write observations.
G3 owns held restart, exact preparation, retained children, ownership and
canonical CAS/recovery. G8 owns failed facts, remaining checks, explicit
repair and infrastructure refusal with successful retry. The blocks have
different failure mechanisms; their within-project successful paths supply
the refusal controls. Snapshot agreement is the promised observation, not a
fabricated expected pass. Git, fixture commands and actual host ownership
supply the remaining expected values. Test comments name the relevant
failure rewrite, changed-input inheritance, teardown and CAS sabotages.

The selected assertion blocks use public CLI/operator MCP, store, Git,
host processes/locks and real pinfold boxes with the scripted model. Held
host child, Git or inventory replies establish deterministic boundaries;
history observers wait for events and final release after failed end.
No worker sabotage or host execution was run to settle these mechanisms.

## Fresh validation still required

The landed runtime implementations and combined Y-104 host evidence cover
the original behavior. They do not substitute for fresh Y-96 exact-head
operator validation, published review and approved actual merged-tree gates.
Before accepting this integration candidate, the operator must:

1. Read the complete candidate and original Y-96 contract, compare exact
   head/proof and all dependency ancestry, and validate this evidence mapping
   against the retained external records.
2. On separate operator-owned state/socket/runtime, run the unchanged G1
   reduced control, G2 skipped-decisions control, both G3 prepared-child
   restart scenarios, G3 surviving-Git-owner and CAS/intent scenarios, and
   G8 selected-failures and infrastructure-retry scenarios at the exact head.
3. Demonstrate direct assertion failures for rewriting accepted failed
   check/result evidence, inheriting acceptance after changed preparation,
   bypassing predecessor teardown and bypassing final CAS. Restore exact
   source after each experiment. Earlier Y-105/Y-106 ownership and Y-104
   feedback/CAS experiments cover only their stated mutations; no independent
   failure-rewrite or changed-identity sabotage is claimed by this record.
4. Publish exact-head review, then approve the concrete candidate through
   the supported queue. Run fmt, clippy, build and the full real host e2e
   gate on its actual prepared merged tree. Record the actual canonical
   landing and resolved intent before consuming it and completing Y-96.

Worker `cargo fmt --check` and
`cargo clippy --all-targets --locked -- -D warnings` passed before this
record's commit. They are compile/style checks, not host runtime evidence.
No worker `cargo test -p e2e` is authorized. This document does not declare
those fresh operator steps complete or request a manual completion override.

Y-61's original downstream dependency remains on Y-96. Full integration and
release work through Y-80 remains required, including release cleanup,
test-audit sweep and Linux arm64/Mac validation. Y-81 stays deferred.
No ticket, approval or installed configuration was changed by this assessment.
