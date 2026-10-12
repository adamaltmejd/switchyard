# Y-110 remaining host staging stop guard

Review 2254/check 1176 passed with a P3 guarantee finding. The operator
identified remaining synchronous pre-turn staging errors that propagated
before consuming an acknowledged stop. This corrects the existing
preparation-stop contract; it introduces no new guarantee or runtime seam.
No runtime reproduction of these synchronous paths is claimed.

## Static source coverage

Two local result groups now pass through the existing preparation_result:

- Proof preparation: writable proof create_dir_all, read-only empty-proof
  create_dir, and read-only proof snapshot validation and copying.
- Packaged preparation: trusted-directory create_dir; selected-input array,
  path and blob validation; real Git blob retrieval; selected Markdown
  UTF-8 validation; selected parent create_dir_all and file write; audit
  prompt and serialized launch bounds; and audit.json write.

The packaged result is one cohesive staging handoff. Its real Git children
finish before that result is checked. No store operation or ownership
operation is enclosed in either new group. Their original Fail conversions
and messages are unchanged. Without stop, the same error propagates. With
acknowledged stop, the existing guard records the error diagnostic and
ends unchanged. Abandonment still uses the existing guard's abandoned path.
Guard/store failures still propagate directly.

Inspection covered implement from the initial load through worker startup.
Clone/audit-directory preparation, agent environment, harness staging,
prompt, argv, image, route and egress already pass through the same guard.
There are no remaining direct host filesystem staging error exits in that
range. Frozen workflow/plan/provider/harness resolution is unchanged;
store reads, resume recording and frozen-target lookup remain outside the
new staging groups. Neither group includes a returned acting owner.
The startup error arm remains guarded, while successful LiveBox still
goes through run_harness and normal teardown. Nothing after the turn,
grant revocation, image handling, capacity proof or containment changes.

## Existing controls and evidence limits

No test or assertion changes. The existing success and failure preparation
scenarios remain distinct, isolated parallel callers. They exercise the
shared stop-versus-error mechanism without one fixture per filesystem call.
Their comments still name propagating preparation Err before honoring stop
as sabotage. The startup mutation and surviving-owner controls remain.
G4/G5/G9 shown-by scope and the prior test-audit gate answers are unchanged.
Expected exits, identities, ownership and effects still come from the
fixture, real Git/Pinfold, host, CLI history and read-only store.

The operator reports Y-112 landed as
938144caf46223217ccbf447063c5feb087451ba, with host gate 78/0 in 584.39s.
Its canonical archive produced 513eca2fbf450af15e764b9b38c6037050e2954e.
Y-110 27ef855 integrated without conflict, yielding tree
49e73b81f077445847eb001b783505fa2b567310. Both preparation scenarios
passed together 2/0 in 46.81s scenario time and 61.687s total. Restoring
only startup Err-before-stop failed directly at G4:1078, red versus
stopped, cargo 101 in 17.36s scenario time and 17.434s total. The operator
restored that merged source and cargo build passed. Evidence is under
/home/yard/validation/y110-27ef85-integrated-20261013.

These are operator results for the prior integrated tree, not this repair
or an actual landing. Earlier failed and stalled predecessors and their
unreached later blocks remain recorded in the unchanged dated archives.

This workspace's parent is 27ef855; it has no remote or object for the
reported Y-112/canonical commits. No prerequisite implementation is copied,
undone or changed here. The patch touches only supervise.rs and this record;
it must integrate with the landed close-only inherited-lock fix on the host.
No unrelated AGENTS edit or checkpoint document is changed.

## Validation

cargo fmt --check, cargo clippy --all-targets --locked -- -D warnings,
and cargo build --locked passed. No e2e, synchronous fault probe or source
mutation run was performed here. Static review identifies routed results;
it is not runtime validation. The final operator gate must rerun both
existing preparation scenarios concurrently on integrated source and the
meaningful error-first mutation before landing.
