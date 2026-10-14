# Y-151 required host scenario correction

Correctness review found that the required G3 scenario assumed two landing
children. The ordinary default plan schedules all four configured definitions
at landing, including the two candidate-stage gates. The earlier fixture
could not pass its child count, unknown command count or snapshot assertions.

The corrected scenario keeps that supported default graph. It asserts the
four fixture-selected landing nodes and their final queue selection. Both
closed children reference the original candidate pass on identical inputs.
After the target changes the merged tree, the first closed landing child
runs and has no reference. The later equivalent child references that fresh
passing landing check. Both children bind host Git's actual merged head,
tree and consumed base. The unknown command runs once at candidate and twice
at landing per attempt. Original candidate snapshots now exclude children.
Approval identity, immutable original rows, stdout-only command observation,
subprocess exit status and cleanup release waits remain covered.

## Test-audit change gate

- Keep the corrected assertion blocks under G3's existing actual prepared-input
  reuse and changed merged-tree rerun clauses. No contract changed.
- Sabotage remains candidate-head-only matching, unknown-input reuse,
  unconditional command replay or copied verdicts. The changed-tree control
  now observes the child that actually performs fresh verification.
- This repairs the existing required cross-stage scenario. It adds no twin
  restart or target-change scenario and removes no guarantee coverage.
- Expected node names and command counts come from the fixture's four
  definitions and default plan contract. Host Git supplies tree/base/head.
  Store snapshots and MCP observe original evidence and final selection.
  A later reference must name the independently observed fresh passing check.

Diff removals: 19 test lines beside 0 binary lines. The incorrect two-child
expectations are replaced; no scenario or guarantee clause is deleted.

Validation passed: `cargo fmt --check`,
`cargo clippy --all-targets --locked -- -D warnings`,
`cargo build --locked` and `git diff --check`.
Host e2e remains deferred to the operator's serial native gate as instructed.
The required first native case remains
`identical_prepared_inputs_reference_candidate_pass_and_changed_tree_reruns`.
