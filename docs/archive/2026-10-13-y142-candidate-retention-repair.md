# Y-142 completed candidate retention repair

The host landing gate on candidate
`2c492521173051d0eba41d980ba417025ead6c32` passed 78 scenarios and failed G2's
`quiescent_plan_cas_retains_only_exact_node_and_producer_evidence`.
Its staging-adoption assertion observed an implementer, two candidate gates
and two reviews in the new revision, instead of only the two affected reviews.
This is a product failure. The assertion remains unchanged.

`attempts::edit_pending` delegates policy comparison to
`executions::plan_superseded`. That comparison included the old implementer's
staged sources. Adopting new staging therefore scheduled an implementation
edit, changed the candidate and forced new gates. The candidate supplied by
the completed implementation should have survived this boundary.

Qualification now ignores staging differences only for an eligible
implementer candidate result that exactly matches the current base, head and
proof. Node policy comparison precedes this exception. Agent settings, runner,
image and all other consumed definitions still compare. Running work and
read-only reports do not satisfy the exception. The existing unread live
handoff check still requires a successor; ownership must be checked before
the candidate result is eligible. No stored row or digest is rewritten.

The existing G2 block already proves the correction at the hardest boundary:
staging adoption after completed judging, with the exact image and context
fixed. Its fixture consumer list requires only the affected reviews; host
gate counters and host model opening counters detect implementation replay.
Immutable row snapshots and typed supplier bindings retain their assertions.
Restoring staging comparison for this completed producer is the sabotage that
reproduces the reported host failure. No new scenario or twin is needed.
Test lines removed: 0. Binary lines removed: 2 replaced declarations.

Validation: `cargo fmt --check` and
`cargo clippy --all-targets --locked -- -D warnings`. Host e2e and independent
runtime sabotages remain post-approval gates and were not run in this clone.
The original Y-142 audit and all parent integration obligations remain intact.
