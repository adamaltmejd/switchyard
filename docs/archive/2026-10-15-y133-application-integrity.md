# Y-133 review2045 P2: finding application integrity

The finding-selector application transaction reloaded the attempt but relied
on an earlier runnable observation. An intervening abandon could therefore
leave a permanent applied fact on a graph that was no longer live.

The existing transaction now revalidates live and runnable state, admitted plan
revision, candidate and proof, ticket revision, absence of pending continuation,
and latest selector execution identity. It reloads that execution before the
existing finding_current ownership, eligibility, input and publication checks.
Only then may the same transaction record application. No lifecycle, schema,
raw report, candidate control or selection-validity rule changes.

The existing held-selector assertions remain unchanged. Their real inherited
owner establishes an unapplied publication through restart, followed by applied
selection after checked release. They do not hold the scheduler between its
outer runnable read and application transaction. Concurrent abandon at precisely
that interval is deliberately untested: there is no external held point there.
No timing race, sleep, binary barrier, fixture, box or model turn is added.

The operator reports the preceding repaired head be2c5bd passed the native
joined-review/nested held-writer case: 1/1 in 134.97 seconds. That evidence is
retained for that head; it is not a claim of a native run of this repair.
The writer outcome fix and its assertions are unchanged.

Candidate checks: cargo fmt --check, cargo clippy --all-targets --locked
-- -D warnings, cargo build --locked. Native host e2e remains the operator gate.
