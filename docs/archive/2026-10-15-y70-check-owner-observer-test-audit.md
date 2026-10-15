# Y-70 check ownership observers

The operator reports native b27716 core selection PASS 1/0 in 84.74 suite
seconds, 91.676 wall seconds, including pending-start refusal and held-intent
full-state preservation. Its selected failure-policy scenario failed 0/1 in
42.13 suite seconds, 42.206 wall seconds: SELECT head from check referenced
an absent column. Logs, exact binary, terminal and read-only database evidence
are retained by the operator under the reported b27716 failure-policy path.
The adjacent selected empty-panel policy scenario was not run because its
judgment projection had the same missing execution-head observation.
This candidate does not transfer the core pass to its own exact source.

Disposition: correct only the two fixture SQL observations. The current
schema's check row contains id, execution, kind, verdict and created_at.
Its migration deletes copied candidate identity columns. Execution owns
the head. The selected failure-policy query now joins check.execution to
execution.id and projects e.head with c.verdict for the exact check id.
The G2 selected empty-panel query already joined that owner; it now projects
e.head alongside c.* and e.reason. Its existing candidate/review head and
verdict assertions and both later immutable judgment comparisons stay intact.

Read all check-table queries added by this attempt against the schema and
migration: these two observations, selected failure history snapshots, raw
five-profile check history, selected review-limit verdict lookup, adjacent
historical-store observations and existing joined landing observers. Other
new queries use actual check columns or compare complete raw history rows;
none requires a copied check head. No production schema, compatibility
path, helper, harness or binary change is introduced. The binding contract
is unchanged; this repair corrects its external observer only.

G2/G8's selected operator failure repair/acceptance and selected empty-panel
policy-lapse evidence remain the clauses shown. Existing comments name
sabotage: copy a losing review, rewrite failed evidence, treat repair as
acceptance, or count every agent verdict as the selected panel. These are
the existing hard cases and no duplicate scenario is added. Git in the
actual fixture clones supplies the expected candidate heads; fixture model
and gate scripts supply passing/blocked outcomes. The read-only store join
observes the exact check's owner instead of a nonexistent copy. Every
identity, history, failure-reason, repair, acceptance and positive control
assertion is preserved. Test lines replaced: 2 SQL strings. Assertions
removed: 0. Binary lines changed: 0.

Validation: cargo fmt --check, cargo clippy --all-targets --locked --
-D warnings, and cargo build --locked passed. No native/e2e suite was run
in this box, as instructed. Both affected scenarios and the full native gate
remain operator-owned qualification for the new exact candidate. No .yard
project configuration changed.
