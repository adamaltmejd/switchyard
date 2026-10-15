# Y-71 Pi usage reopen observer repair

Preserved failure, as reported by the operator: full native gate 3497 at exact
head 5207870323ebee029a0be9e5fd67a2701ce88c54 finished 94 passed and one failed
in 1319.11s. The failure was
g07_usage::pi_unknowns_and_interrupted_resume_usage_remain_execution_local at
g07_usage.rs:143. Raw evidence remains on the host at
/home/yard/validation/y111-operator-evidence-20261013/approved-y71-520787-gate-capture/y-71-gate3497.log.
This worker cannot read that host path and does not replace the evidence or
claim the failed gate passed.

The whole-row reopen comparison differed only in reserved changing from true
to false and teardown changing from false to true. Usage and result agreed.
Stopped attention precedes checked release, so those cleanup observations can
legitimately change across restart. No product behavior changes.

Test-audit disposition: rewrite the existing interrupted Pi reopen block to
compare only usage, outcome and execution/harness/version/model/session/resumed
identity. Retain the fixture-derived numeric totals, unknown fields, partial
harness provenance and the later resumed explicit-zero completion control.
No Y-71 integration or containment block changes.

The exact G7 Shown by clause is: "Real Pi frames with no usage stay null; a
resumed turn whose box ends while its next host reply is held retains only its
reported new-message tokens and null cost after reopening, and a later resumed
explicit-zero completion records zero with complete harness provenance."
The binding clause already matches this observer and needs no edit.

Sabotage: discard observed increments on reopen, count resumed history, replace
null cost with zero, change partial/unreported provenance, or rewrite the
interrupted execution's harness/model/session/resumed identity on reopen.
The test comment names these mechanisms. Usage expectations originate in the
scripted host model's token/cache counts, the wrapper's removed real Pi fields
and the spec's completeness/provenance contract. Retained identities are the
same real execution's recorded identities before reopening; the promise being
tested is that these observations agree across reopening. The pre-reopen
numeric/provenance assertions remain, so equality alone cannot bless incorrect
values. The successful resumed zero turn remains the positive control.

This block is distinct from completed Pi landing, fresh Codex and historical
migration scenarios: a resumed Pi execution loses its terminal frame after a
reported cache-inclusive message. The real host model holds the next reply;
real box down drains the reported stream. Before restart, the test observes
failed stopped attention, the held fixture and the exact failed execution's
partial usage and predecessor identity. It does not require cleanup to remain
held or add a cleanup wait.

Ownership/recovery coverage remains unchanged. G4/G5's
an_old_host_child_retains_capacity_across_restart retains the exact reservation
while a real inherited child holds its lock and permits release only after it
exits. G5's a_daemon_killed_mid_execution_interrupts_it_and_start_continues
checks interruption, box removal and retained work. G8's completed judgment
ownership checks retain reservation/teardown and ineligibility until checked
release. The existing Codex usage scenario explicitly waits for release before
its full-row snapshot; no such wait is needed for Pi's G7 usage promise.

Test lines removed: three (one sabotage comment, one reopen comment and the
whole-row assertion), replaced with scoped assertions and precise intent.
Binary lines removed: zero. No ownership assertion is removed or weakened.

Worker validation: cargo fmt --check, cargo clippy --all-targets --locked --
-D warnings and cargo build --locked. This workspace has no pinfold or container
runtime. Before another full native qualification, the operator must run the
affected real Pi usage scenario and both g09_integration scenarios at the new
exact candidate head. Those native runs are pending, not passes.
