# Y-70 obsolete candidate-gate repair

Review3363 at ab1c7dc found a read-to-transaction race. The scheduler could
read B's failed candidate gate, then an operator could select A. The repair
transaction checked only plan and ticket edits. Writer dispatch reread the
attempt and could start A with B's failure feedback.

The existing gate-repair transaction now rereads the attempt and requires
the observed choice, candidate base/head, proof and plan revision. The attempt
must still be live, without a pending continuation or superseding ticket edit.
A selected choice must still qualify against its declared current inputs.
The authoritative current-check lookup must still return the same failed
check and execution. That lookup checks result eligibility, checked teardown,
definitions and exact supplier bindings. The exact failure decision must also
remain unchanged. Only then does the same transaction dispatch the producer,
freeze feedback, invalidate the choice and withdraw its approvals. An obsolete
trigger returns without those effects.

Read together: scheduler gate outcome and decision routing; selected review
repair guards; current-check eligibility and execution qualification; graph
input binding; selection current/activation/invalidation; writer dispatch and
feedback capture. The repair guard precedes writer creation and selection
invalidation. It uses the existing project transaction and authoritative
check lookup, with no new lifecycle, owner or reservation.

There is no demonstrated externally held point between the trigger read and
this transaction. No timing race or binary barrier was added. No test or
assertion changed. The existing five-profile same-reviewer picker scenario
still holds B's actual automatic repair and observes B's workspace, model,
replacement Git head and exact failed gate execution/node/output feedback.
It then proves the repaired file and fresh candidate/review/choice/checks
before approval and landing. This remains the allowed repair control; it
does not claim deterministic coverage of the obsolete-trigger interleaving.
All earlier assertions and controls remain in place.

Validation: cargo fmt --check, cargo clippy --all-targets --locked --
-D warnings and cargo build --locked passed. Native/e2e was not run, as instructed; qualification
remains the operator's host gate after review. No .yard configuration changed.
