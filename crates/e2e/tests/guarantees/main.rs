//! The end-to-end guarantee scenarios in one binary, so test threads stay
//! busy across guarantees.

mod g01_only_the_queue_lands;
mod g02_judgments_bind_identity;
mod g03_landing_cas;
mod g04_capacity_holds;
mod g05_intent_precedes_effect;
mod g06_inputs_validated;
mod g07_lands_end_to_end;
mod g08_review_is_a_publication;
mod g09_implementer_starts_right;
mod g10_queue_one_at_a_time;
mod g11_only_sync_changes_yard;
mod g13_runs_where_its_row_says;
mod g14_worker_git_untrusted;
mod g15_plans_and_proposals;
