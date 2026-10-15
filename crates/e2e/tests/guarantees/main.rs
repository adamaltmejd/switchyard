//! The end-to-end guarantee scenarios in one binary, so test threads stay
//! busy across guarantees.

mod g01_only_the_queue_lands;
mod g02_judgments_bind_identity;
mod g03_landing_cas;
mod g04_capacity_holds;
mod g05_intent_precedes_effect;
mod g06_inputs_validated;
mod g06_plan_inspection;
mod g07_claude_usage;
mod g07_lands_end_to_end;
mod g07_operator_mcp;
mod g07_usage;
mod g08_audit_reports;
mod g08_durable_package;
mod g08_review_is_a_publication;
mod g08_selects_existing_candidate;
mod g09_implementer_starts_right;
mod g09_integration;
mod g10_queue_one_at_a_time;
mod g11_only_sync_changes_yard;
// G12 is established by G8's Pi, Claude and Codex seat scenarios.
mod g13_runs_where_its_row_says;
mod g14_worker_git_untrusted;
mod g15_plans_and_proposals;
