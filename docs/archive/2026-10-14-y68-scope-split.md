# Y-68 scope review — 2026-10-14

Baseline: fb3f727, after Y-67. No runtime behavior changed in this review.
Y-68 is incomplete. The proposed children preserve its original scope.

## Evidence

`store/schema.rs` declares `execution.attempt` non-null. Checks, attention,
approval and execution self-references must survive its migration.
`store/mod.rs` enables foreign keys before transactional forward migrations.
Changing the initial schema alone would not upgrade an existing store.

`store/executions.rs` requires an attempt in `Execution` and `Start`.
`start` loads it unconditionally to obtain plan revision and input bindings.
`end` also resolves attempt admission for typed results. Historical rows
must retain unknown scope rather than acquire evidence from current config.

`jobs/capacity.rs` puts most effect locks beneath the attempt directory.
Reservation eligibility, waiting events, ownership attention and checked
release consume attempt records. `jobs/reconcile.rs` releases old owners
and records interruption. Nullable storage alone cannot make these paths
safe for a standalone execution.

`jobs/supervise.rs` resolves gate and implementation inputs through
`attempts::for_execution`. `jobs/report.rs` uses that same resolver for
publication policy and attempt edits for supersession. The operator stdio
MCP and daemon dispatch have no standalone creation operation. Worker HTTP
grants have a separate dispatch. Optional association must not make an
individual result eligible for ordinary candidate control.

Existing historical-store observers are in `g08_audit_reports.rs` and
`g05_intent_precedes_effect.rs`. Held publication, stop and inherited lock
observers exist in `g08_durable_package.rs`. Future assertion changes must
use test-audit and read each affected scenario in full before qualification.
No assertion changed here, and no host e2e run was attempted.

## Proposed boundaries

The yard-file skill requires one reviewable mechanism per ticket. The user
explicitly authorized proposing a split and stopping when the ticket is
too large. Three parked proposals were submitted through `yard_propose`:

1. Proposal 453, key `y68-scope`, depends on Y-67. Migrate nullable execution
   association and frozen scope, preserving historical records and existing
   attempt behavior. Do not expose standalone admission yet.
2. Proposal 454, key `y68-standalone`, depends on `y68-scope`. Expose both
   boxed command and read-only report through trusted operator MCP. Reuse
   capacity, deadlines, ownership and checked restart. The first capability
   milestone is the required independently runnable zero-ticket scenario
   exercising both runners. Include immutable results/usage, worker refusal,
   stop/restart and no candidate authority. A report-only result is incomplete.
3. Proposal 455, key `y68-attached`, depends on `y68-standalone`. Add optional
   existing-attempt association without approval, check, route or landing
   authority. Prove frozen identity and results through relevant attempt
   changes and retained ownership, with ordinary attempt controls.

These are proposals awaiting the operator's decision, not landed ticket IDs.
No proposal removes Y-68 acceptance requirements. Each implementation must
update ARCHITECTURE.md for behavior actually delivered in that commit.
Native cases run serially after approval; the full host landing gate follows
successful affected cases. No `.yard` or installed runtime change is needed.
The binding architecture stays unchanged because this review implements no
new contract. Progress was recorded through `yard_progress`.

Validation before commit: `cargo fmt --check` and
`cargo clippy --all-targets --locked -- -D warnings` passed. The e2e suite
was not run, as instructed. No development daemon opened project state.
