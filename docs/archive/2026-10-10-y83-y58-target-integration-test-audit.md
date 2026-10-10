# Y-83 and Y58 target integration, 2026-10-10

The ordinary landing queue returned the architecture conflict and supplied
/yard/input/target.bundle. Fetching its refs/heads/main supplied exact
f9f4602f1b1f7df7abc0019c9ab55e8e4231d1db. Merge that committed target
into documentation-corrected candidate edec60257ff43cd24e73113dc1c623f7fd83b8d4.
No bundle was fabricated or installed by hand.

Only docs/ARCHITECTURE.md conflicted. Resolve its guarantees-table hunk
with the candidate's complete G4/G5 rows and the target's complete G6/G7
rows. Keep the auto-merged target G1/G2 stdio clauses and the candidate's
G9 reserved-preparation stop clause. Compare all 15 resolved rows with
their required parent rows; no guarantee clause is removed.

The target's operator_mcp module, its binary and test module registrations,
io-std feature, client fixture and host scenarios merge unchanged. The
existing capacity mechanism, inherited-owner proofs, preparation-stop and
abandonment guards, and once-only landing transaction repair remain.
The merge changes no .yard configuration or frozen reviewer setting.

Test-audit Gate for the inherited target scenarios:

- Keep operator_wait_cancellation_keeps_work_and_worker_grants_separate.
  G1 observes an initialized worker-session operator refusal with the
  trusted stdio operation as control. G2 observes stale revision/head
  identities and unchanged rows after successful edits, then approves the
  current candidate. G6 observes a schema-invalid edit and its valid
  control. G7 cancels a read wait and closes another client's stdin while
  real boxed work is held, then observes approval and queue landing.
  The existing comment names operator dispatch, identity forwarding,
  strict schema and work-stopping sabotages. Fixture revisions, host Git,
  external rows and the held model request supply the expected values.
- Keep accepted_operator_mutations_finish_after_transport_failure.
  G7 holds a real sync Git effect, observes the fatal frame error before
  release, then observes its completion and canonical at the independently
  read checkout head. A successful same-project sync is the control.
  The existing comment names aborting accepted forwards as sabotage.
  This mutation-drain boundary differs from the read-wait cancellation
  boundary; neither scenario replaces the other or the Y83 capacity tests.

Both scenarios and their assertions are identical to the supplied target.
The existing Y83 G3/G4/G5/G7/G9/G10 host assertion files are identical to
the pre-merge candidate. Assertion lines removed: 0. Binary lines removed
by manual conflict resolution: 0. Existing dated Y58 audit evidence is
preserved. No new scenario, test hook, sleep or retry is added.

Combined worker validation: cargo fmt --check, cargo clippy --all-targets
--locked -- -D warnings and git diff --check passed. Host e2e was not run
in this worker, as instructed. The previously reported 47-scenario pass
on cec10d6 does not establish this combined head. Fresh candidate checks,
review, host validation and exact approval remain required before landing.
