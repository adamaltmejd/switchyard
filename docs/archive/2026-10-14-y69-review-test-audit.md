# Y69 review correction audit, 2026-10-14

The correctness review found two lost-progress paths: multiple exact start
answers overwrote one attempt-wide continuation, and a stopped writer's
attention blocked an edited ticket after another writer drained. This change
keeps continuation on the existing execution lifecycle. It adds no queue,
owner, lease or state machine.

## Assertion dispositions

The strongest five-profile concurrent scenario in g04_capacity_holds.rs is
extended. All original isolation, immutable publication, exact reader,
retention, exclusive-owner, stop/retry and cleanup observations remain.

| Disposition | Spec clause and sabotage | External expectations and hardest case |
| --- | --- | --- |
| Extend the existing crash with a second held writer | G9: each accepted declared continuation records an independent execution; sabotage by writing both start answers into attempt.next | Real host-model requests hold both boxes at daemon death. Host locks, partial files and store rows establish the crash point before restart. After checked release, exact stopped-item answers must each produce a distinct execution. Both successors reuse their own recorded session and clone, commit fixture partial files, finish with exit zero, and preserve the earlier immutable publication. One successor remains held while the other starts. A single interrupted writer cannot expose overwrite by a second answer. |
| Add the edited-ticket sequence in the same five-profile fixture | G9: edited stopped attention discharges after old owners drain and all writers read the edited ticket; sabotage by returning on open attention before reconciling the edit | An initially held writer returns unchanged while another real writer remains held with a partial file. The operator edits revision 1 once. No successor starts while that owner remains active. After release, all five revision-2 producers commit fixture revision bytes, exact named readers pass with exit zero, and the original stopped item resolves through edit. Git supplies committed bytes; host inode and partial-file observations prove retained workspace continuation. Existing single-writer edit checks lack this stopped/held interleaving. |

The same-project allowed controls remain: five independent writers, named
stop, explicit declared-workspace continuation, and successful starts after
exclusive ownership is released. No refusal becomes a timeout pass. The
existing host child wrapper still preserves original stdin in a closed
descriptor, forwards actual child status and retains inherited locks through
its FIFO hold. Its source and historical archive inputs are unchanged by this
correction.

No assertion block or guarantee clause was removed or weakened. The candidate
count changes from five to six because the second recovered writer publishes
an additional immutable candidate; the original five-profile requirement and
all original publication checks remain. Test-file deletions are 9 lines of
replaced setup/start/count statements and comments. Binary-source deletions
are 26 lines, chiefly the superseded attempt-wide continuation scheduler path.
There is no operator weakening decision to request.

The boxed checks are cargo fmt --check, cargo clippy --all-targets --locked
-- -D warnings and cargo build --locked. E2e remains unrun in this worker by
the user's instruction. These observations are runnable host scenarios,
pending serial affected-case qualification and the full landing gate after
review; this report does not claim passing host evidence.
