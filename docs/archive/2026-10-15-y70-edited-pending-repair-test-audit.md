# Y-70 edited pending repair

Review3387 at 40e50fa identified two stale observations. Steering could
invalidate a newer operator choice by validating an earlier attempt snapshot.
A ticket edit could invalidate the choice but retain pending B repair. A
scheduler waiting on real Git before lane reacquisition could then consume
that repair through the default producer, or retained instructions could
prevent new readers and selection.

Steering now reads the live attempt inside its existing transaction. It
retires pending instructions when an edit is pending and validates the actual
current choice before invalidation. Attention still resolves against current
execution reads and only after owners drain. Writer scheduling stops when
steering changed its observed choice or pending instructions.

Pending dispatch now holds the existing admission lock and revalidates the
live attempt, plan, choice, candidate, proof, lane, pending instructions,
ticket revision, edit precedence, current choice eligibility and runnable
ownership/attention inside its existing project transaction. Only then does
it reserve the lane and start the selected producer. An obsolete observation
records neither a lane/clock write nor a producer/choice/approval effect.
Current selection remains the producer authority. There is no fallback from
an invalidated observed B choice to A in this dispatch path.

Read together: ticket editing and steering; live selection eligibility,
invalidation and activation; pending queue return and bundle delivery;
runnable ownership and attention; default refresh and lane capacity;
selected producer resolution, feedback freezing, writer creation and choice
invalidation. The same execution lifecycle and admission lock remain.

The test-audit gate applies to the extension of
`only_the_declared_choice_is_checked_and_landed`:

| Question | Disposition |
| --- | --- |
| Guarantee and clause | Keep. G2/G8 bind new judgments to the edited ticket and exact source. G10 now names the held scheduler across a body edit, retirement without dispatch, and fresh selected-producer repair. |
| Sabotage | The test comment names retaining old next after an edit or dispatching it after the lane wait. The fixture then sees retained pending state, a stale repair execution, wrong producer, absent fresh candidates, or no fresh approval. |
| Distinct hardest case | The existing same-B automatic repair and explicit-start refusals remain. They hold the same real scheduler Git read but never edit its ticket. This extension edits while that read holds the old dispatch before lane reacquisition. No micro-window timing race is asserted for steering. |
| External expected values | The fixture supplies the revision-two body, five profiles, B's version bytes and target markers. Host Git supplies target, clone head, FETCH_HEAD and merge parent. Supported MCP performs the edit and approvals. Store rows and history events observe dispatch, exact inputs, immutable history, new bound reviews and approval identities. |

The second pending conflict waits for an ended event after its own audit
boundary and for checked release after that event. It cannot reuse the
earlier conflict or preparation release. While Git holds, the edit must
clear choice and next, preserve the released lane/clock, create no writer,
and preserve bundle bytes. After release, all five new implementation rows
must read revision two as edit executions without obsolete feedback. Fresh
selected B still conflicts with the external target. Its actual producer
fetches that target and publishes a new candidate. Its own review, picker
source, required check, approval, omission acceptance and held CAS reach
landing. The complete earlier execution history stays unchanged.

No assertion block was deleted. The final exact approval ledger includes
both additional retired conflict approvals and the new landed approval.
Selected-only gate observations include the two externally observed new B
heads and still exclude every losing source. All earlier five-profile,
same-reviewer, failed-picker, stale/foreign choice, review-limit, independent
base, pending refusal, automatic repair, omission and open-intent controls
remain. No sleep, retry, binary barrier, new lifecycle or harness was added.
Test lines: +197/-8. Binary lines: +64/-12. Removed lines are replacements
or the pending dispatch moved into its guarded transaction; no assertion
block was removed.

Validation: cargo fmt --check, cargo clippy --all-targets --locked --
-D warnings and cargo build --locked passed. Host e2e was not run as instructed;
affected native qualification and the full landing gate remain operator-owned
after boxed review. No .yard configuration changed.
