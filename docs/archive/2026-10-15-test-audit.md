# Y-73 classification assertion audit

The operator requested complete optional command or read-only agent
classification, bounded named continuation, exact failed evidence and unchanged
Y132 remediation groups. This change extends existing host fixtures. No new
fixture machine or historical build is added. Candidate compilation checks do
not establish host behavior; the host e2e landing gate remains pending.

## Dispositions and external expectations

- G2/G8, `candidate_reuses_original_pass_after_bookkeeping_with_unknown_rerun`:
  retain the existing failure and authorized acceptance case. A real boxed
  command publishes a same-node retry, bounded to one dispatch per plan.
  Attention must report that bound exhausted, including after a ticket edit.
  Expected counts come from the fixture's failed commands and declared bound;
  verdicts come from the externally read store. Sabotage is unbounded retry,
  resetting the bound on edit, or recording continuation as pass. This protects
  runaway execution and false verification at the cost of three command
  classifier boxes and one additional failed-check box, with no model calls.
- G2/G8, `delayed_candidate_acceptance_names_the_changed_identity_and_writes_nothing`:
  retain the existing stale operator acceptance case. Its scripted host model
  now requests original repair, a named different fixer, then acceptance.
  The store must retain three failed checks and the configured repair models;
  the host model counts its three classification openings before attention.
  No approval may exist until the explicit operator decision. Undeclared and
  stale publications must refuse before valid same-turn publications succeed.
  A later text-only classifier must leave no report or eligible output; its
  operator retry starts a new execution and publishes successfully. Expected
  identities come from the frozen prompt and host Git; revisions come from
  the operator edit. Sabotage is accepting undeclared/stale publications,
  ignoring the fixer, granting classifier acceptance authority, or treating
  missing publication as a route. The added work is five classifier boxes and
  model openings, two repair writer boxes and one review box, plus two model
  tool-result turns for rejected publications. It protects unauthorized
  acceptance and wrong-profile dispatch in the existing hard case.
- G2/G8, `concurrent_readers_share_capacity_and_join_exact_results_across_restart`:
  retain its Y132 contributor and blocking-verdict assertions. The scripted
  read-only classifier observes the immutable selected groups; the existing
  repair prompt must carry exactly those groups. Expected identities and
  contributor sets come from the fixture's original reviewer publications and
  picker selection. Sabotage is reconstructing groups from raw reviews,
  dropping contributors, or treating an excluded required finding as accepted.
  This adds two classifier boxes and model openings to the existing machine.
  It reuses the difficult multi-model join instead of duplicating its setup.

All three fixtures use real binaries, daemon, boxes and harnesses with the
scripted host model. No mock, public model, sleep, retry loop, timeout proof or
binary barrier is added. The existing allowed decisions and successful
publications remain the positive controls. The original reports and failed
verdicts remain externally inspected evidence.

Deliberately untested here: the topology cross-product, native classification
crash permutations, and the plan-preauthorized acceptance variant. Those
contracts remain exact. Runtime has not been measured in this boxed candidate.

The diff replaces eight test lines and 135 binary lines; this is a new
guarantee implementation, not a consolidation or coverage sweep. No assertion
block is deleted. The user requires one integrated implementation commit and
explicitly reserves execution of the host e2e suite for the landing gate.
