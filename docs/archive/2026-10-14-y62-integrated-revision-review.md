# Y-62 integrated revision review

Reviewed source: `1fc4e9c1c9379a8354bff3d304c76abaf9e96c0e`.
This is the parent integration review against the original Y-62 clauses,
the 2026-10-09 plan and the 2026-10-12 operator partition. The sizing-only
attempt 103 supplies no implementation or acceptance evidence.

No missing implementation clause was found in this source review. The
binding architecture already describes the integrated behavior. This commit
changes no binary, assertion, guarantee, project configuration or lifecycle.
It does not authorize landing, close Y-62 or move Y-63/Y-65 dependencies.

## Mechanisms and acceptance observers

| Original clause | Integrated mechanism and existing observer |
| --- | --- |
| Remaining node/profile/routing adjustments use expected plan revision | Operator MCP forwards structured selection and overrides to `attempt_revise`. Resolution uses recorded definitions; the transaction rechecks live attempt identity and revision. G2 `quiescent_plan_cas_retains_only_exact_node_and_producer_evidence` changes final and nonfinal profiles and routing, observes actual selected model requests and older producer identities, and checks stale CAS writes nothing. |
| Preserve history; invalidate only changed inputs | `append_plan` inserts revision evidence without replacing original admission. `executions::qualifies` compares node policy, consumed definitions and actual typed suppliers. `checks::current` requires exact candidate/ticket/digest, registration, eligible result and checked release. The quiescent G2 scenario retains an unrelated completed seat, changes downstream suppliers, and compares historical admission, execution, result, check and revision rows through reopening. |
| Freeze running inputs; stop affected judges before replacement | Review and implementation startup use `attempts::for_execution`; gate startup uses the same historical reading. Revision collects only superseded running judges for stop. G2 `a_live_plan_revision_preserves_publication_and_waits_for_checked_teardown` revises a published held reviewer, refuses the stale second edit, observes its live box and locked host owner without replacement, then observes old box absence and lock freedom before the new judgment counts. Its original failing publication remains in the store. |
| Implementer steering is a new execution with revision diff | Revision records the running implementer's handoff. Pending edit takes precedence over routed repair; `plan_handoff` compares historical plans and consumed definitions. G9 `an_edit_reaches_the_next_prompt_and_stop_delivers_it_sooner` holds startup behind real capacity, observes original launch inputs, then the actual successor's adopted settings, staged bytes and labelled diff after predecessor release. Ticket and instruction edits also reach new productive executions. |
| Explicit revision alone adopts workflow/gate/reviewer/image/tool/skill definitions | Independent definition, staging and image commits default to their recorded sources. Definition adoption re-resolves selection; staging and image adoption preserve omitted definitions. The quiescent G2 scenario observes template sync before adoption, a new gate consumer and digest at revision 5 with unchanged review and withdrawn approval, changed reviewer settings at revision 6, separate tool and skill inputs at revisions 7/8, and exact older image context at revision 9. Host counters, native model requests and Git identities provide the external observations. |
| Stale decisions refuse after adoption | The same G2 scenario retains withdrawn approval after a stale decision naming revision 1 against revision 9, observes exact expected/current revision and no audit write, then approves the current identity and lands in the same project. Earlier stale edit controls run while a reviewer is held. No new twin is added. |
| Current safeguards remain outside the plan | Admission and queue policy reads use the current target configuration. Existing G2 protected-path and manual-policy sync scenarios withdraw automatic approval on the same plan without new judgments. These observers are reused unchanged. |
| Revisions cannot retarget, bypass grants or reverse execution history | Revision retains admitted workflow/runtime and compares implementation access, agent, outputs and publication grants. Explicit source adoption preserves harness, connection and egress. Open intents and unsupported pending work refuse revision. Target edits retain their live-attempt refusal. G3 intent controls and G9 target controls remain; G2 historical snapshots cover returning routes without relabeling producers. |

Consumer identities include committed staged paths, blobs and modes, selected
agent settings and harness pins, review policy and boxed image context. Host
gates do not consume a boxed image. Unknown consumed evidence does not qualify
cross-revision reuse. No new worker tool grant is introduced by an adjustment.
The existing operator API, daemon transaction and execution reservations remain
the only mutation and ownership paths.

## Y128 stopped-owner revisit

Packaged committed revisions are now supported, so the earlier unsupported
revision trigger has occurred. `capacity::release` already reads
`attempts::for_execution(tx, &released)` in `stopped_implementation`. That
helper selects `execution.plan_revision`; it never substitutes the latest
admission. A missing recorded revision refuses, and an unbound execution has
no admission proof. Neither case infers packaged ownership.

The predicate remains limited to a lock-contract implementation ended as
`unchanged`, on node `implementer`, with recorded packaged runtime and an
existing lock file. Reservation release first requires box absence and host
lock freedom. Reconciliation matches only the exact execution's open
`red:ownership` item. Ordinary unchanged preparations retain their operator
exit; unknown ownership retains capacity.

G8's existing held-stop segment in
`a_packaged_audit_bounds_iterations_and_requires_native_publication` explicitly
adopts a definition while the old execution's same-inode host lock and release
probe are held. It compares the old execution and both questions before and
after adoption. Checked release resolves ownership alone, preserves the raw
unchanged result and ineligibility, leaves stop open and starts no execution.
Explicit start then consumes the adopted instructions and publishes natively.
Both revisions remain packaged by the supported shape constraint. This
scenario does not claim to distinguish a latest-admission runtime lookup or
prove missing-revision behavior through mutation; the source trace establishes
which historical lookup is used. No artificial store migration or unsupported
runtime transition is added to manufacture that distinction.

## Validation boundary

`cargo fmt --check`, `cargo clippy --all-targets --locked -- -D warnings`
and `git diff --check` passed. No e2e suite or development daemon was run.
No construction state, socket or runtime was opened, and `.yard` was unchanged.

The mapped assertions are existing host scenarios, not fresh runtime evidence.
The Y109 integration archive and earlier child archives retain their own
qualification boundaries and failures. The Y128 report named by the operator
is not available in this workspace and was not independently read. Host
landing validation and independent sabotage/restoration qualification remain
required after approval. This source review does not turn those pending gates
into passes or substitute for the operator's landed-dependency check.
