# Y-67 scope partition

Source inspected: c87a253. No implementation or binding contract changed.
Y-67 is not complete. The ticket explicitly permits proposing a split and
stopping when the work is too large. Proposals 449 and 450 are parked for
Adam. They retain the original acceptance and the required first milestone.
They introduce no prerequisite outside Y-67's implementation scope.

## Traced mechanisms

`Config::normalize_ordinary` and `Plan::ordinary_selection` in `plan.rs`
normalize selected plans back into the ordinary linear panel. The selection
walk accepts one complete successor and visits each retained node once.
`Plan::ordinary_routes` follows the first complete edge and installs the
linear panel input and bounded final blocking route.

`jobs/mod.rs::runnable` excludes an attempt while any execution is running
or reserved. `advance` follows the first complete edge, schedules one
missing check and returns. Removing only the running-row exclusion would
allow duplicate starts. Following more edges without replacing linear
panel accumulation would not establish exact joins or safe repairs.

`store/executions.rs::binding` assigns ordinary node identities from job
kind and configured name. A non-implementer supplier must be a review seat.
Its value comes from the current panel check. It has no generic command
result supplier path. `qualifies` reconstructs those inputs and compares
the admitted producer policy and definitions; complete routing alone may
change without invalidating equivalent existing evidence. Joins must retain
exact supplier evidence and reject results invalidated by revised inputs.
They must not replace qualification with publication presence.

`supervise.rs::box_gate` mounts the command checkout writable.
`review.rs` mounts each seat's checkout read-only. Admitting concurrent
read-only command nodes requires launch enforcement alongside scheduling.
The existing host/box containment contract must remain exact.

`capacity::effect` reserves actual external work under the validated project
default's `max_executions`. A denied reservation writes no reservation.
Reservations remain until checked release. `reconcile::project` tears down
old boxes and proves each running child's ownership before recording its
interrupted end. Published reviews can become eligible without replay
after checked release. Reuse these mechanisms; introduce no branch owner,
lease, recovery path or capacity allocator. Attempt lanes and per-target
landing serialization remain separate constraints.

## Proposed implementation boundaries

Proposal 449, key `y67-concurrency`, implements the candidate graph
mechanism through admission, advancement, input binding and read-only
launch. It depends on the original Y-63, Y-66 and Y-83 tickets. Admission
still requires original Y-63 integrated acceptance actually complete.
Its first normal candidate includes the required independently runnable
real-host scenario: held concurrent selected readers, an independent ticket
racing the last existing execution slot, and an exact required-result join
that waits before succeeding in the same project. Safe ownership, restart
and revision behavior cannot be weakened to make this slice smaller.

Proposal 450, key `y67-recovery`, depends on `y67-concurrency`. It completes
the original restart, stale-plan/result and different-target obligations
through the existing hardest held-owner and capacity scenarios. It includes
implementation corrections where needed and the integrated acceptance;
it is not a report-only completion ticket. Y-67 remains open until all
original obligations pass on the integrated candidate.

Both proposals require a binding architecture update with each implemented
behavior, test-audit for every assertion change, boxed fmt/clippy/build,
correctness review before the operator's serial affected host scenarios,
and the required full landing gate afterwards. Both remain parked.

## Fixture review requirements retained

The ordinary default graph includes candidate-stage command definitions
again as landing consumers. The Y-151 host-scenario correction records an
actual failure from assuming fewer children. Read every authored selection
with its dependent expectations before allocating a native slot. Enumerate
branch/join nodes, selected candidate and landing consumers, exact supplier
results and frozen revisions, failed/incomplete routes, held owners,
counter increments and cleanup release. Use supported read-only plan
inspection to check selection, with expected values from fixture/spec/host.
Trace retained versus future history waits and preserve subprocess exits.

The existing `racing_workers_share_one_execution_slot` proves two attempts
sharing a slot, not concurrent children in one attempt or an explicit join.
`one_slot_lands_with_child_gates_and_excludes_a_worker` already proves a
landing child and worker share capacity without reserving an idle parent.
Extend the hardest applicable mechanisms; do not copy their assertions
into parallel scenarios. The first required graph scenario remains new
because its multi-execution attempt and exact multi-supplier join differ.

No assertions, source, `.yard/` configuration or construction state changed.
No daemon was launched and no host e2e suite ran. Dependency acceptance is
an operator admission condition, not inferred from source presence here.

Worker validation passed: `cargo fmt --check`,
`cargo clippy --all-targets --locked -- -D warnings`,
`cargo build --locked` and `git diff --check`.
