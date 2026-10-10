# Y-83 sync invalidation boundary repair, 2026-10-10

Review1232/check660 passed exact c2a3697dcc177239abc3bdaa30925f1be8bc42b0
with P2 sync invalidation before consume preconditions and unrelated P3
transient reap race. Source inspection confirms sync cleared the published
default before checkout_branch, dirty_checkout and checkout_changed refusals.
Those consume failures leave canonical unchanged. A capacity waiter could
then acquire canonical, see unavailable input and end red:error. This is
source evidence for a G4 current-input omission, not a host reproduction.
The reap finding remains outside this focused repair.

The unconditional invalidation is removed. Import invalidates immediately
before canonical update-ref. Default selection invalidates immediately
before symbolic-ref. Current and checkout-only consume paths preserve the
published input throughout. Successful sync still refreshes the validated
actual default under the existing canonical lock before returning. Explicit
start's c2a3697 publication is unchanged.

Failure boundary audit:

- Parameter, branch, missing-history, divergence, config and orphaned-workflow
  refusals precede any canonical target/default mutation and preserve input.
  Fetch imports objects without naming a canonical target; it does not
  invalidate the published default.
- Consume's branch, cleanliness, fetch and fast-forward refusal/error paths
  affect only the operator checkout. They never clear canonical's input.
  Current and selection-only preparation likewise do not clear it.
- A completed update-ref returning false revalidates the actual current
  default while canonical is still held before returning refusal. It does
  not restore the old Arc or guessed limit. Failed revalidation leaves input
  unavailable. A command error with an uncertain mutation outcome returns
  without publication; it does not assume the target stayed unchanged.
- Successful import followed by an audit failure leaves input unavailable.
  Default-selection command/audit failures likewise leave it unavailable.
  If import precedes selection, no earlier input is restored between them.
  Final refresh publishes actual canonical state, not the incoming config
  assumed to be selected. No allocator Git, fallback, second authority,
  retry, lifecycle or control exemption is introduced.

Coverage disposition: all 50 scenarios and their assertions are unchanged.
The existing G11 named-sync scenario observes wrong-checkout, dirty-checkout
and divergence refusals with external heads/files, then successful consume
and import controls in the same project. Default-selection scenarios retain
held workers, frozen identities and successful selection controls. G4 retains
its held worker/default/limit and manual-start controls. These boundaries do
not deterministically order a waiting allocator ahead of normal scheduler
refresh after a refused consume. No suitable supported held point was found
that exposes this narrow publication gap while excluding that refresh.
No ordinary suite pass is claimed as proof of the race, and no timing,
sleep, retry, timeout-as-pass or binary barrier is added. Test-audit Gate
adds no assertion block to judge; assertion lines removed: 0.

The operator reports repair1233 ended timeout without code or scenario
evidence, and attention143's edit renews the clock through the supported
exit. This audit claims no validation from that attempt. Prior dated archives
remain history. Y58 integration, capacity/ownership guards, schema, .yard,
operational daemon and frozen reviewer/configuration remain unchanged.

Worker validation: cargo fmt --check, cargo clippy --all-targets --locked
-- -D warnings and git diff --check passed. Host e2e is
not run in this worker as instructed. Fresh exact-head review and full host
validation remain required before final approval.
