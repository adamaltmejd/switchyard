# Y-68 operator node schema repair — 2026-10-14

The full native gate on 787c1ab passed 86 scenarios, including the real
nullable migration. Four scenarios failed before daemon dispatch with
invalid argument node: a_landing_retires_when_canonical_moves_before_update_ref,
prepared_input_and_completed_child_survive_host_child_restart,
target_advance_retires_prepared_input_and_requires_fresh_children, and
selected_failures_continue_without_rewriting_judgments.

Y-68 added a node definition object to execution.start. The schema builder
selected that object schema by field name alone. Existing attempt.accept
and attempt.repair use node as a string identity. Their correct calls were
therefore rejected before authoritative ownership and decision validation.

The object schema now applies only to the explicitly object-typed node
field. Identity fields keep their declared string schema. The same schema
still drives tools/list and boundary validation. No daemon decision,
ownership, capability, scope or lifecycle predicate changed. The binding
architecture records these argument types in this commit.

Existing assertions remain unchanged. The G2/G3/G5/G8 scenarios retain
their exact acceptance, stale ownership, crash, fresh-child and landing
observations. Standalone G5/G6/G13 cases retain actual definition objects
and operator/worker controls. There is no new assertion block or twin;
the reported native failures already expose this regression. No test lines
were removed; one production line was replaced.

Validation: cargo fmt --check, strict locked all-target clippy, locked build
and git diff --check passed. The built binary's real stdio tools/list,
with isolated temporary state/runtime names and no daemon or project, declared
attempt acceptance and repair node fields as string and standalone start's
node field as object. This checks advertised schemas, not native behavior.
No e2e suite, runtime installation, dependency download or .yard change was
made. Run the four affected native cases serially and the required full host
landing gate on the repaired head. Prior-head passes do not qualify this head.
