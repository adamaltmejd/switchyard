# Smaller test surface, 2026-10-15

Base: b817d7ae82e7de7aa3754f1b3bbf3604bfdf99aa. Adam authorized reducing
unique secondary coverage as well as duplicates. Product behavior, G1-G15,
threat model, box table and worker grants remain unchanged. Only automated
proof is narrowed. AGENTS Tests and test-audit now select a small risk-based
core; Shown by records that core instead of an exhaustive scenario inventory.
The existing operator AGENTS addition stays uncommitted in the original tree.

## Static surface

| Module | Before | Retained |
| --- | ---: | ---: |
| g01_only_the_queue_lands.rs | 2 | 1 |
| g02_judgments_bind_identity.rs | 13 | 5 |
| g03_landing_cas.rs | 6 | 2 |
| g04_capacity_holds.rs | 11 | 5 |
| g05_intent_precedes_effect.rs | 2 | 2 |
| g06_inputs_validated.rs | 1 | 1 |
| g06_plan_inspection.rs | 2 | 1 |
| g07_claude_usage.rs | 1 | 0 |
| g07_lands_end_to_end.rs | 3 | 1 |
| g07_operator_mcp.rs | 4 | 3 |
| g07_usage.rs | 4 | 1 |
| g08_audit_reports.rs | 5 | 2 |
| g08_durable_package.rs | 5 | 3 |
| g08_review_is_a_publication.rs | 12 | 6 |
| g08_selects_existing_candidate.rs | 2 | 1 |
| g09_implementer_starts_right.rs | 3 | 1 |
| g10_queue_one_at_a_time.rs | 5 | 2 |
| g11_only_sync_changes_yard.rs | 4 | 2 |
| g13_runs_where_its_row_says.rs | 2 | 2 |
| g14_worker_git_untrusted.rs | 2 | 1 |
| g15_plans_and_proposals.rs | 4 | 3 |

Total: 93 -> 45 existing test functions.

This is not a measured speedup. Actual removed work includes the second
five-writer edit lifecycle; two modern historical store builds; specialized
Claude/Codex accounting fixtures; four packaged archive/variant build calls;
packaged adoption/iteration matrices; repeated gate-error/owner/retry chains;
and multiple secondary planner, admission, target, stop and omission cycles.
No tests are bundled to lower the count and no replacements are requested.

## Retained core and accepted gaps

Retain unauthorized landing and worker/operator authority; stale candidate,
proof and choice identity; native CAS; real crash, unknown-owner and corrupt
checkpoint recovery; concurrency and exact join capacity; per-harness secret
and candidate-code isolation; actual full ticket, report and packaged use;
and five different implementers with own reviews, one replacement and native
landing. The product still supports behavior without a dedicated test.

Deliberately untested secondary coverage includes selected omission/failure
option matrices, quiescent plan-adoption combinations, independent-target
throughput and live-limit drain permutations, startup/stop error permutations,
review/report owner lifecycle variants, adapter-specific usage and modern
historical schema variants, pure inspection/configuration matrices, future
preset/one-off admission permutations, sibling-key collisions across planner
executions, default-selection overlaps, proof-symlink snapshot refusal, session
execution-limit cycles, dirty-work retry variants and packaged settings/staging/
image compatibility combinations. These are accepted coverage choices, not
missing guarantees or tickets to recreate the old surface.

## Changed retained observations

The mixed reader/worker race now compares the complete native box-name set
under pinfold's own project label with the two held actors, before release.
This preserves actual unrecorded-box exclusion while dropping the simpler
worker-only race. Fixture latches hold both actors; the literal configured cap
is two. Sabotage: launch another box before recording its reservation.

Pi's interrupted reopen compares usage and resume identity, allowing checked
cleanup to finish. The unchanged fixture supplies all numeric/provenance and
explicit-zero expectations. Earlier failed full gate3497 remains failed.

The edit/adoption scenario requires the fixture edit text to reach the model;
it does not demand a resumed-only diff marker when changed definitions cause
a fresh session. Failed Y158 gate3521's retained store proves resumed=NULL and
the whole current body in the actual prompt. ARCH's fresh-body contract was
met. The stopped/resumed same-project delivery control remains.

## Qualification

Formatting and compilation checks are in progress; native validation is
pending. No canonical import, push, release, installation or daemon change has
occurred. Preserve all preceding failed runs. Final measurements and committed
identities belong in a separate dated qualification record after checks pass.
