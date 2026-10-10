# Y-59 final integration evidence — 2026-10-10

This candidate audits the complete original Y-59 contract after its three
implementation slices landed. It changes no runtime, assertion, binding
contract or project configuration. No concrete remaining integration defect
was established by this source pass. The inherited implementation and host
evidence cover the original acceptance scenarios below. This record is not
an operator approval or a replacement for the fresh exact-head host gate.

## Actual landed inputs

| Ticket | Landed implementation head | Inherited operator evidence |
|---|---|---|
| Y-84 | `6f70b68204f51d1cd88fd2b04f0846010436a588` | `2026-10-10-operator-y84-validation.md`: exact candidate 53/53 in 231.93 s; landing 53/53 in 225.38 s |
| Y-85 | `ec434d7f78ff129650acebf60c10f9727fbd70c8` | `2026-10-10-operator-y85-validation.md`: exact candidate 55/55 in 214.86 s; landing 55/55 in 243.89 s |
| Y-86 | `95924b770f4481c114e5471d8abf4d591986ee2b` | `2026-10-10-operator-y86-validation.md`: exact candidate 56/56 in 218.06 s; landing 56/56 in 247.40 s |

All three heads are ancestors of this checkout's starting head,
`20a9b291b5a2294223cee6337def08549c7090e5`. Y-85 also includes the landed
Y-87 admission-provenance prerequisite,
`92764a50d5b8eab7cc33fbaacfe1d95fa0becce8`, as recorded in
`2026-10-10-operator-y87-validation.md`. Its initial candidate,
`12cae4ba89b4e78778652c0a2d84878d2607d992`, failed host validation and is
not the landed head. Y-57 and Y-58 precede Y-84. The abandoned Y-59 split
proposal is not implementation evidence.
These pass counts are reports from the committed operator records, not
tests run by this worker. Y-86's complete suite includes the earlier slices'
scenarios on the combined implementation.

The operator's source-gate correction prompted a recheck of every named
landed head against its committed validation record and Git ancestry. All
named scenarios below exist in the stated source files; those guarantee
files are unchanged from landed Y-86. The source assertions and operator
records support the mapping and reported counts below. This recheck adds
no runtime evidence and preserves every source-only and unrun label.

## Shared source and lifecycle

`config.rs` parses typed profiles and presets and validates their named
definitions. `plan.rs` is the shared representation and resolver for target
inspection, validation and admission. It expands profiles, applies targeted
field overrides, checks typed input/output references, declared outcomes,
required results and bounded cycles. There is no expression evaluator.
Runnable matching requires equality with an existing native workflow chain.
Valid graphs outside that subset remain inspectable and cannot run.

`jobs/admit.rs::admit` resolves before inserting an attempt or its first
execution. The existing transaction records revision 1, the actual matched
workflow, selected agent/seat/gate/review definitions, routes and selection
evidence. Provider, login and profile references are names; the configuration
blob identifies the preset/profile definitions. No credential value belongs
to this record. There is no second run lifecycle or run row.

`jobs/mod.rs::read_config` records source commit, configuration and Dockerfile
blobs, build-context tree and trusted staging paths/blobs/modes.
`load_attempt` reads that immutable source and verifies its inputs and native
plan against the admission record. `resolved` consumes persisted definitions.
Implementation, review, candidate gates, repairs and queue landing use this
same reading. `supervise.rs::image` builds the admitted context; trusted
staging uses its source rather than a repaired candidate base. Runtime image
IDs remain observations on executions, separate from admitted input identity.

`store/checks.rs` matches the nullable plan revision as part of current
results and records it on approvals. `jobs/mod.rs::stale_part` and
`queue.rs::holds` preserve exact candidate, proof, ticket, plan, digest and
selected-result bindings. Template sync does not update recorded admission.
Schema migration leaves old admissions and plan bindings unknown;
`load_attempt` retains their explicitly specified current-template behavior.
Recovery does not manufacture an admission from the current template.

The implemented boundary refuses selection on live work and stopped/red
retries with `active_plan`. It provides no active-plan adjustment operation.
No revision-CAS adaptation or general graph runtime is claimed here. A
template edit cannot serve as an implicit adjustment. The later adjustment
contract remains separate from this slice's immutable admission behavior.

## Original acceptance mapping

Scenario names below are independently runnable real host e2e tests under
`crates/e2e/tests/guarantees/`. Their inherited runs use real daemons,
pinfold boxes and scripted host models. This worker did not run them.

| Original clause | Inherited scenario and external observation |
|---|---|
| G6 unresolved references, output type mismatch, undeclared outcomes and unbounded repeats name the input; corrected control in the same project | `g06_plan_inspection.rs::typed_plans_inspect_without_admission_and_refuse_unsupported_runtime`: operator MCP asserts exact input/reason tokens for absent node/output, wrong type, outcome, missing repeat bound, zero bound and missing required result. Corrected overrides resolve to the literal fixture graph in that project. |
| Named profiles/presets, targeted overrides and one-off graph share the representation; inspect without work | The same scenario compares preset expansion and one-off resolution to fixture declarations, distinguishes feature/a from main using host Git, and observes unchanged audit and empty ticket/attempt/execution rows after inspection. |
| Unsupported runnable shapes are refused; equivalent legacy plan remains supported | The same scenario refuses a valid bounded repeat, then validates and executes the literal default chain. Real boxed Git inputs distinguish candidate from merged-ref gates, including the second instance of candidate-stage gates at landing. `selected_admission_refuses_without_work_and_freezes_operator_inputs` refuses unsupported admission with `input: plan`, unchanged work rows and unchanged model request count, followed by supported admission in the same project. |
| G2 frozen resolved plan survives template sync | `g06_plan_inspection.rs::selected_admission_refuses_without_work_and_freezes_operator_inputs` holds a real model request, syncs workflow/profile definitions and compares the complete persisted admission before and after. Live selection is refused. `g02_judgments_bind_identity.rs::a_new_head_revision_or_proof_rejudges_but_template_sync_keeps_checks` observes stable judgment counts and revision 1 after gate/workflow/seat sync, while head, ticket and proof changes still rejudge. |
| Selected gate/reviewer/Dockerfile/tool/skill inputs stay frozen through the native lifecycle | `g10_queue_one_at_a_time.rs::a_ticket_binds_feature_inputs_repairs_and_lands_on_its_target`: fixture v1/v2 declarations, Git blobs/tree, trusted skill text in real model requests, host gate counters and boxed image marker distinguish admitted inputs through target conflict repair, changed base, quiescent daemon restart, approval and landing. Stored check execution/image/digest/plan identities survive restart. Main stays fixed and feature/a lands. |
| Future attempts resolve changed templates | The same G10 scenario observes v2 gates, reviewer, Dockerfile, skills and image on a future feature/a admission. `selected_admission_refuses_without_work_and_freezes_operator_inputs` separately admits the changed preset and a literal one-off graph through trusted MCP after the old attempt ends. |
| Held automatic approval plus protected-path-only sync withdraws on the same candidate without judgment rerun | `g02_judgments_bind_identity.rs::a_sync_that_protects_an_auto_approved_path_withdraws_it`: real held landing host gate, withdrawn auto row, same head and revision 1 attention, exactly one model opening and one candidate gate. Manual approval of that same head then lands in the same project. |
| Held automatic approval plus approve=manual-only sync has the same behavior | `g02_judgments_bind_identity.rs::manual_policy_withdraws_auto_approval_without_rejudgment`: same external observations and same-head manual landing control, with only approval policy changed. |
| Manual decisions remain exact; old digest-change expectations adopt the new revision contract | The G2 identity scenario refuses old head/proof answers with expected/current identities and no row; the current answer succeeds. `g08_review_is_a_publication.rs::a_seat_that_always_blocks_gets_max_rounds` binds the admitted blocking check and revision after gate-template sync. G10 also refuses the wrong target for a feature/a decision. These are the landed Y-85 replacements, not the old template-invalidation assertions. |

The required sabotages are already named in these inherited test comments:
skip output-reference validation; read late workflow/gate/reviewer/staging
inputs; or read safeguards from the admitted plan rather than current target
policy. No sabotage execution is claimed for this pass.

## Live constraints and preserved controls

Automatic approval in `jobs/mod.rs` reads current selected-target policy
under its target lock. `queue.rs::admit` reads current policy separately from
the frozen execution inputs before landing children and intent. Withdrawal
does not supersede unchanged checks. The abandonment-during-policy-read G2
scenario also proves that an ended attempt receives no new attention.

Capacity continues to use the published current default limits, not the
admitted target's limits. `g10_queue_one_at_a_time.rs::same_target_worker_and_landing_drain_at_one_slot`
holds a real landing gate and reserved pre-box clone, lowers the live limit
from two to one and proves no landing intent appears while usage drains.
Release reaches the real host model and permits both candidates to land.
`g04_capacity_holds.rs::landing_gate_policy_reads_hold_the_shared_execution_slot`
distinguishes current-default capacity from the frozen target and observes
the reservation and inherited lock on post-approval policy reads.
Lane admission remains serialized by the existing admission lock.

Source-only evidence for live credential availability: implementation and
review call the harness route against `daemon.machine` before issuing the
execution bearer. Frozen provider/login names supply no stored secret and
cannot make a missing credential available. This pass does not claim a
post-admission credential-removal runtime scenario. Inherited G8/G12
Pi/Claude/Codex credential scenarios establish host route injection and
absence of upstream secrets inside the real boxes.

Source-only evidence for post-admission containment narrowing:
`supervise.rs::egress` intersects admitted hosts with current selected-target
hosts at launch; review grants no general egress. The plan cannot widen
mounts or worker tools. Inherited G13 checks observe read-only seats and
proof, and gate refusal of model/MCP access. This pass does not claim an
egress-removal runtime scenario. `mcp.rs` issues tools by execution kind,
never graph node declarations. G1 and G7 prove worker/operator separation;
G4's `an_execution_ending_revokes_only_its_own_project_grant` proves revocation
is project-scoped. G8's own-session controls retain registration and
publication proof.

G15's `an_upgrade_preserves_ended_planner_proposals_and_limit_overrides`
retains truthful unknown admissions and legacy exits.
`an_upgrade_retains_a_legacy_pre_box_child_without_an_ownership_proof` and
G4's `an_old_host_child_retains_capacity_across_restart` retain unknown-owner
exclusions. No migration, containment, writer or lifecycle control changes
in this candidate. G11 retains worker `.yard` refusal and future-admission
operator-sync control.

The two below-blocking findings in the Y-85/Y-86 operator records remain
findings: a concurrent admission can make a scheduler tick fail closed until
its source is loaded; graphs matching several workflows can select different
native lifecycle limits. The actual matched workflow and limits are recorded.
This pass established neither an identity bypass nor a new reproduced CLI
defect, and does not claim either finding fixed.

## Test-audit disposition and current validation

The test-audit skill was read for this focused acceptance review. Keep the
inherited blocks mapped above: spec clauses, external observers, literal
fixture/Git/pinfold expected values and named sabotage are present. The
typed-resolution, selected-admission, frozen-repair and policy-withdrawal
scenarios exercise different mechanisms. No duplicate scenario is added.
Assertion changes: zero. Test lines removed: zero. Binary lines removed:
zero. No guarantee or shown-by clause is weakened; no operator waiver is
requested. This is not a whole-suite cleanup sweep.

Current candidate checks: `cargo fmt --check` and
`cargo clippy --all-targets --locked -- -D warnings` passed. Host e2e and
sabotage runs were intentionally not executed in this box. The operator
still owns full exact-head host validation, fresh review and landing gates.
No development daemon was started or construction backlog opened. `.yard/`
is unchanged. Y-59's downstream Y-60/Y-66 dependencies and the useful
redesign objective through Y-80 remain unchanged.
