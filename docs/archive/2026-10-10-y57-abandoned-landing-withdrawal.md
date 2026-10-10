# Y-57 abandoned landing withdrawal repair

The operator's 2026-10-10 host validation of exact head
4e89bd10eacc2c859e44273e560f98760b73922c reported a failure in the unchanged
G2 scenario an_attempt_abandoned_while_its_landing_rereads_policy_raises_nothing.
Boxed checks 678/679/680 passed. The host suite was still running when the
operator reported the failure; no terminal total is claimed here. The
operator reports that c4014a8 passed this scenario. Earlier archive files
remain historical. No live host fixture or operational state was touched.

## Diagnosis through the existing fixture

The real fixture in g02_judgments_bind_identity.rs changes approval policy
through sync, then holds the landing's protected-path Git read with its
wrapper and FIFOs. After the wrapper announces the held read, the fixture
abandons the attempt through the real CLI and releases the child. Its
execution.ended observation requires the landing outcome withdrawn. Its
store observation requires no open attention. The assertions at lines
577-584 remain unchanged, as do the fixture and its allowed control.

The operator's log reports execution no longer eligible to act. The new
queue policy admission loop tested attempt eligibility after the held
policy read and returned Fail::refused for the now-ended attempt. That
return bypassed queue::withdraw. Source inspection identifies the precise
outcome difference: spawn sees the attempt's abandoned outcome and ends
the still-running landing with outcome abandoned, rather than withdrawn.
That branch suppresses attention for abandonment, but still violates the
fixture's required landing outcome. This is source diagnosis using the
reported real fixture failure, not a new worker runtime observation or a
claim to have captured the host event independently.

## Repair and retained controls

Queue policy admission still refuses an execution row that is no longer
running. An attempt that ends during the policy read or publication wait
now returns the existing superseded-approval lapse instead of an effect
error. Each caller routes that lapse to queue::withdraw, which records
withdrawn and performs the existing approval cleanup. A superseded lapse
raises no attention and never invokes the admitted effect callback.
The ended attempt cannot start another gate, prepare a ref or write intent.

The eligibility check occurs before the unavailable-publication wait, so
abandonment also ends that wait without needing to restore uncertain input.
The existing reservation and inherited host lock remain owned until the
enclosing effect completes and proves external ownership release. No slot
is guessed released. Live attempts still wait on project events when
publication is unavailable. Ordinary capacity admission eligibility checks
are unchanged. Target locks, persisted intent refs, exact CAS, shared cap,
unknown-owner exclusion and all other review repairs are preserved.

## Validation

cargo fmt --check, cargo clippy --all-targets --locked -- -D warnings and
cargo build -p yard --locked pass. No test source or assertion changed;
all 52 scenarios remain required. Host e2e and runtime sabotage were not
run inside the worker, as instructed. Fresh exact-head review and renewed
full host validation remain required before approval. The Git I/O heap
pin and bounded proof remain unchanged. No test hook, dependency, runtime
setting, .yard file, construction state or operational configuration changed.
