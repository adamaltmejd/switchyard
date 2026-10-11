# Y-98 approval attempt identity repair

Review 1698, check 892 reported a P3 identity integrity finding. Approval
read a live attempt before awaiting load_attempt. Inside its transaction,
skipped_decision read that old attempt by id, while approval_item read the
current live attempt again. Replacement between the initial read and the
transaction could therefore bind skipped evidence to an attempt different
from the authoritative approval row and use the old admitted policy.

The transaction now reads the ticket's current live attempt first. It must
have the id used to load admitted policy. A replacement returns stale with
identity=attempt, expected=the loaded attempt id and current=the live attempt
id. If no live attempt remains, current is null. This refusal precedes
skipped_decision and all approval writes. With the same id, skipped_decision
uses the transaction-current row. approval_item reads the same live attempt
inside the same synchronous store transaction. No mutation or await occurs
between these reads. The project holds its store mutex for the transaction,
so another daemon mutation cannot replace the attempt between them.

A ticket edit within the same attempt passes this id guard and reaches the
existing skipped decision checks. Their ordering remains unchanged: the held
edit refuses the old revision; after the edited candidate is ready, the old
head refuses. Target, base, proof, revision, plan, selected skipped nodes and
approval attention keep their existing exact checks. Live protected-path
policy, selected check revalidation, skipped and unreviewed semantics,
authoritative approval evidence and queue validation are unchanged. Ordinary
approval also refuses replacement before using the old loaded policy.

Test-audit disposition: retain all G2/G8 blocks and Y-97 acceptance controls.
No assertion, fixture or observer changes. The existing omitted acceptance
scenario observes both the persisted approval attempt and the exact attempt
inside skipped evidence; it retains the held-edit stale revision/head
refusals, all other stale/no-write refusals, current acceptance, remaining
gate evidence, empty panel, absence of fabricated results, landing and
quiescent reopen. Expected nodes/revisions/text remain fixture literals;
base/head come from host git, and attention/proof identity agreement comes
from the observed approval item. Its sabotage comments remain intact.
This narrow repair closes the source-level attempt replacement gap; it does
not claim that the existing scenario deterministically holds that race.
No test hook, sleep, retry, deadline change or new lifecycle is introduced.
Test assertion lines removed: 0. Binary lines removed: 1, replacing the old
attempt-id read with the transaction-current live-attempt guard.

The second P3 finding about missing-proof invalid/stale consistency was
read. It is not fixed by this repair or waived as a guard failure. Proof
validation is unchanged.

The operator reported host session 42838 at 27dd217 failed 69/1 in 451.32
seconds because yard init/pinfold box list timed out before an existing G8
crash case began. This identity repair is independent and does not fix that
timeout. A subsequent read-only label-list probe returned zero in 0.03
seconds and doctor was healthy; neither establishes a full-suite pass.
These are operator-reported host observations, not candidate test results.

Validation in this candidate: cargo fmt --check, cargo clippy --all-targets
--locked -- -D warnings and git diff --check. No e2e execution inside the
candidate. Fresh exact-head source/controls review, test gate, focused omitted
acceptance and full 70-test host runs, current published review and actual
landing remain required. Prior archives are unchanged.
