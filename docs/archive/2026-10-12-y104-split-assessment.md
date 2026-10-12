# Y-104 merged decision split assessment

Source reviewed: `722ad5a`, including the actually landed Y-103 prepared
continuation. Y-104 is not implemented by this assessment. The worker
instruction permits proposing a split and stopping when the ticket is too
large. The yard-file skill requires one mechanism per ticket. This scope
has two distinct operator decision entry points, with different evidence
and scheduling requirements. Both reuse attention as the decision truth.

## Source boundary

- `jobs/queue.rs::land` sends a failed merged child through `returned`.
  That ends the landing, retires approval and schedules repair. Waiting for
  an operator answer must preserve approval instead. `next` currently
  resumes only error or interrupted landings, and `passing_child` retains
  only eligible passing evidence. Failed acceptance changes all three
  readings and the final selected-child validation before intent.
- `jobs/admit.rs::candidate_answer` validates candidate `checks::Input`.
  Its base and head are the candidate's, not the prepared old target and
  merged head. It performs no canonical target or prepared-ref reading.
  Reusing this validator without extending its identity would authorize
  the wrong input. The trusted CLI and operator MCP currently require a
  check identity for both accept and repair.
- `store/checks.rs::decision` reads resolved candidate attention as the
  sole persisted answer. Exact merged decisions can extend this store
  contract without another lifecycle or decision table.
- `plan.rs::ordinary_selection` permits `operator_failure` only on
  candidate gate and review nodes. Retained landing failure handling needs
  an admission change, distinct from accepting an already omitted node.
- `jobs/mod.rs::skipped_identity` filters candidate and review omissions.
  `jobs/admit.rs::skipped_decision` runs at candidate approval, before a
  prepared merge exists. Its explicit decision is stored on approval.
  It cannot bind landing omissions to a future exact merged head.
- `jobs/queue.rs::land` traverses retained landing nodes only. An empty
  landing selection proceeds directly from preparation to mutation intent.
  Explicit omission acceptance needs an operator decision opportunity at
  that prepared boundary. Ordinary omission must still proceed without
  an explicit acceptance. No omitted child or check exists to answer.
- Y-103's continuation proves predecessor ownership, validates the pinned
  input and preserves passing children. Its final selected-child check
  retires invalid evidence before intent. Both proposed changes must retain
  those controls, shared capacity and the final canonical compare-and-swap.

These are source observations, not reproduced CLI bugs. No development
daemon, project, socket, box or runtime was started.

## Proposed partition

Proposal **240**, key `y104-failed-merged-decision`, covers admission and
trusted acceptance or repair of selected failed merged gates. It depends
on Y-103. It binds the original landing, candidate, target old head, pinned
merged head, revisions and exact node/child/check/result. It preserves
failed evidence, runs all remaining selected requirements and revalidates
decisions before consumption. Its host evidence includes held-child
restart, stale refusal with no write, fresh acceptance and the existing
infrastructure-error retry control.

Proposal **241**, key `y104-skipped-merged-decision`, depends on proposal
240. It covers explicit named landing omission decisions after preparation,
including an empty selection. It reuses the first child's merged decision
validation but binds admitted skipped identities rather than inventing a
child or check. Ordinary omission remains without explicit acceptance.
Its separate host evidence proves the prepared decision boundary and
absence of fabricated evidence through actual landing and restart.

Both are parked proposals for operator review. Proposal **242** requests
that Y-104 depend on the second child, transitively on both. Y-104 retains
its entire original integration and evidence contract. Neither proposal
was accepted or started by this worker. No live contract was weakened or
changed. Y-104, Y-96 and Y-61 remain incomplete until exact-head operator
host evidence satisfies their full original contracts.

## Test-audit disposition

The test-audit skill and its sweep policy were read. No assertion block
changed, so this assessment claims neither an assertion gate pass nor a
whole-suite sweep. The proposed test boundaries are:

- G2/G3 prepared continuation scenarios: retain their real host-child and
  Git holds, outside ownership/input proofs and host-Git merged identities.
  Extend the failed-evidence continuation case without duplicating ordinary
  passing-child restart or canonical CAS controls.
- G8 `selected_failures_continue_without_rewriting_judgments`: preserve
  candidate acceptance evidence; merged acceptance needs a distinct input
  oracle and a remaining-child hold. Compare complete immutable failure
  snapshots through continuation, actual landing and restart.
- G8 `a_gate_errors_start_reruns_that_gate`: retain its broad no-check
  publication and same-project successful retry controls. Extend the
  refusal at the merged boundary; infrastructure errors never gain a
  judgment or an acceptance exit.
- G2 `selected_skipped_decisions_bind_identity_without_manufacturing_results`:
  preserve candidate omission controls and extend to prepared landing
  omissions. Observe the absence of omitted execution/check/result rows;
  a fabricated pass must fail this block. Full default and ordinary
  reduced/empty landings remain same-project positive controls.

Each child must answer the skill's four assertion gate questions in its
own audit: guarantee clause, discriminating sabotage, hardest distinct
case and outside expected value. Refusals must assert exact stale
identity/expected/current and unchanged decision/audit snapshots, followed
by a fresh successful control in that project. Restart requires a held real
child or Git point and proof before restart. No sleep, retry, clock wait,
public endpoint or test-only binary barrier is admitted.

Test lines removed: zero. Binary lines removed: zero.
`cargo fmt --check` and
`cargo clippy --all-targets --locked -- -D warnings` passed before commit.
Host e2e and sabotage were not run under the worker instruction. They remain
the operator's exact-head approved landing gate; compilation is not host
evidence. No `.yard/` file was changed.
