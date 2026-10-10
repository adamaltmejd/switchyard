# Y-83 initial landing target-load repair, 2026-10-10

Review1212/check645 passed candidate 3b4595373f02011c110c83b9788b6127520e3dc2
with one P3. Source inspection confirms queue::land called load_target
before its first capacity effect. That function validates the branch and
reads its ref even on a cache hit; a miss also reads committed config and
Dockerfile. These are landing-owned Git children. The earlier gate-loop
audit did not establish completeness: it omitted this initial call. Keep
that archive as dated history rather than updating its claim.

The operator reports exact 3b45953 passed all 50 host scenarios in 170.82
seconds, and the stop-omission sabotage failed in 14.01 seconds. These
are operator evidence, not worker-run host tests. Neither result proves
this initial target-read boundary.

Move load_target into the existing preparation effect, after that effect
reserves and installs HOST_LOCK and after acquiring canonical there.
Return loaded config and target_ref with the prepared target and merged
head for the unchanged later phases. There is no extra reservation,
lifecycle, owner or target policy. Capacity calculation still reads the
current default config before admission; that necessary calculation is
not renamed landing work. The landing's selected-target read is guarded.

Source audit of queue::land's execution-owned Git paths:

- Before preparation, only execution, approval, attempt and ticket store
  reads, host path construction and frozen head selection remain.
- Preparation guards branch validation, target ref/config/Dockerfile
  loading, target rev-parse, ancestry, merge-tree, commit-tree and the ref
  retaining the prepared commit. Conflict-return bundle creation shares
  this effect without a nested reservation.
- Each gate admission reserves the existing landing row before canonical
  locking, current target-config loading and automatic approval's
  protected-path diff. The transaction still binds current identities.
  Release proves ownership gone before the child gate reserves and acts.
- Failed-child return reserves the landing row before its target bundle
  Git call. Both returned call sites have an enclosing landing effect.
- Final admission and canonical mutation share the final effect: current
  config/policy loading, protected-path diff, intent binding, update-ref
  compare-and-swap, ancestry and recording remain unchanged.

The post-effect unresolved-intent path delegates to existing reconciliation.
Its canonical observations after ownership proof, and queue::next's
pre-execution approval screen, are unchanged. This audit distinguishes
those existing paths and capacity calculation from the initial
execution-owned selected-target load; it claims no new runtime coverage
of reconciliation or scheduler reads.

Test-audit Gate: extend the existing
landing_gate_policy_reads_hold_the_shared_execution_slot scenario.

- G4's shown-by clause now describes initial target-read reservation and
  inherited ownership before any prepared ref or child gate. The host Git
  wrapper compares its actual open file identities with the fixture
  project's landing locks, without assuming a descriptor number. It holds
  the first owned branch validation on a FIFO. Scheduler/config reads and
  implementation-owned reads do not meet that ownership condition.
- External store rows show exactly one running reserved landing, agreeing
  with the recorded execution identity. Canonical Git has no prepared
  landing ref, and no landing-child execution exists. These observations
  locate the read before preparation completes, not merely somewhere in
  the landing's lifetime. The lock is an external ownership observer,
  not a binary test hook or an asserted descriptor/file-layout convention.
- Moving the initial load back outside preparation makes the first owned
  target read occur at gate admission, after the prepared ref exists.
  The new observer then fails on Git state, not a timeout. The scenario
  comment names this omission sabotage; it was not run in this worker.
- Release permits the same project's first host gate, held policy diff,
  second gate and canonical landing at max_executions=1. All original
  gate-policy, parent identity, child absence, marker and landed-byte
  assertions remain. Initial and first-child parent identities agree.
  The fixture supplies expected phase ordering, gate names and bytes;
  Git, host FIFOs/open descriptors and external rows supply observations.
- The previous block could catch unreserved gate-policy work but never
  observed the earlier config read. Extending it adds this distinct
  omission boundary without another scenario or duplicate continuation.

Assertion lines removed: 0. Binary statements removed: the initial
unguarded load and target-ref construction, moved into preparation.
All 50 scenarios, retained-owner stop refusal/control, preparation and
abandonment guards, integrity repairs and Y58 MCP code/clauses remain.
No sleep, retry, timeout-as-pass, binary barrier, dependency or operational
.yard/frozen configuration change is added.

Worker validation: cargo fmt --check, cargo clippy --all-targets --locked
-- -D warnings and git diff --check passed. Host e2e was not run here, as
instructed. Fresh review and full exact-head host validation remain
required before approval.
