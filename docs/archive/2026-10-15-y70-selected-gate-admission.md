# Y-70 selected gate admission

Review3368 at fe8b98b identified the admission counterpart of review3363.
The scheduler observed B's execution columns and input readiness outside
the start transaction. An operator could select A before that transaction.
Execution creation then bound A's selection input while retaining B's
base, head and proof columns, which also determine the actual gate checkout.

Verification admission now rereads the live attempt inside the start
transaction. Like automatic gate repair, it requires the observed plan,
choice, candidate and proof, no pending continuation, no superseding ticket
edit and a current eligible choice when selected. It resolves the current
declared inputs before creating any execution row. For a selector graph,
the bound candidate value must equal the execution base, head and normalized
proof. A superseded observation or unavailable input returns without row
creation or dispatch. Execution creation uses that same Start value and
the same binding function under the same transaction.

The outside-transaction input-readiness check and its now-unused Boolean
wrapper were removed. Ordinary graph input eligibility still uses binding;
the explicit selected candidate comparison applies only to selector graphs.
No alternate input identity, reservation or lifecycle was added.

Read together: shared scheduler execution columns, declared role admission,
selected choice eligibility and immutable source inputs, gate startup and
checkout columns, execution binding and creation, current-check qualification,
and the just-fixed automatic gate-repair transaction. The two transactions
now fence the same observed choice/candidate/proof identities before effects.

No externally held point inside this read-to-transaction race is demonstrated.
No fabricated race, sleep, retry or binary barrier was added. No assertions
changed. Existing real five-profile selected-only gate and same-producer
repair controls remain intact, including exact source feedback and Git heads.
The operator's earlier failure-policy and empty-panel policy passes remain
historical evidence, not qualification of this source change.

Validation: cargo fmt --check, cargo clippy --all-targets --locked --
-D warnings and cargo build --locked passed. Native/e2e was not run as
instructed; affected native
qualification and the final full host gate remain operator-owned after review.
No .yard configuration changed.
