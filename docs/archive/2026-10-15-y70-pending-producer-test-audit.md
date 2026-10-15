# Y-70 pending repair producer binding

Review1721 identified a choice replacement after a valid landing return.
The return delivered B's target bundle and recorded its repair in next.
A quiescent operator selection could replace B with A without clearing that
repair. Dispatch then resolved A from the new choice and delivered B's
instructions to A, whose input mount lacked the bundle.

Choice activation now rereads the attempt inside its transaction. While
next exists, a replacement must reference the same exact immutable candidate
as the existing choice. A different candidate refuses with
pending_continuation and both references before invalidating anything.
The operator call rolls back its provisional selection execution and all
audit writes. Same-candidate selection retains the pending instructions
and target bundle. Unknown absent prior choice evidence stays absent and
cannot authorize a replacement. Dispatch also rereads the attempt inside
its writer-start transaction before resolving the selected producer.
No new stored field, owner, lifecycle or compatibility mechanism is added.

Read together: operator expected-identity and quiescence guards; selection
publication/current/activate/invalidate; writer steering; queue initial
return validation, immutable approval-choice producer, target bundle staging,
final repair transaction; runnable and lane reacquisition; implementation
node resolution, writer feedback freezing and choice invalidation. The
superseded-return and post-intent fences remain unchanged.

Disposition: extend the existing five-profile G2/G8/G10 host scenario's
valid B conflict, retaining every original assertion and control. Its Git
wrapper executes the real target bundle creation before arming a hold on
the scheduler's next real symbolic-ref read. Return holds canonical while
installing repair; approval released the attempt lane, so dispatch must
read canonical configuration before reacquiring it. The held Git call
therefore prevents dispatch even if the scheduler had an older attempt
snapshot. Landing ownership release needs no Git or canonical lock and
can finish while the scheduler is held. The fixture waits for checked
release after the exact conflict end, then proves from the read-only store
that B's choice, repair target and branch remain pending with lane zero.
No crash, new binary barrier, sleep, retry or immediate-command race is used.

While that call stays held, selecting A refuses with the exact candidate
references and no audit, attempt or bundle change. Selecting the same B
succeeds with a fresh operator choice while preserving next and bundle
bytes. Releasing the real Git call reaches the existing held B producer,
which fetches the actual target, resolves the conflict, republishes and
earns its own fresh review, selection, checks, approval and landing.
Both old and new choice execution rows remain immutable alongside the
original raw evidence. The original same-producer target-fetch control,
pre-intent supersession, post-intent refusal, omission, review-limit,
foreign/stale choice, five-writer and same-reviewer assertions remain.

The test comment names sabotage: activate A while retaining B's pending
instructions. The new block catches that change before it can dispatch;
existing controls began after B's repair execution started and could not
exercise a quiescent pending replacement. This repeated choice on a real
returned conflict is the hard case, within the same scenario. Observers
are CLI history, operator MCP, read-only store, host bundle bytes and Git
in the producer clone. Expected targets and heads come from external Git;
identity and byte comparisons establish agreement and preservation rather
than copying a verdict. The allowed same-B choice and actual fetch/landing
supply the positive control. ARCHITECTURE.md records the refusal and G10
shown-by evidence. Test assertions removed: 0. Binary lines replaced: 3
writer-start lines and the prior activation invalidation call.

Validation: cargo fmt --check, cargo clippy --all-targets --locked --
-D warnings, and cargo build --locked passed. Native/e2e was not run in
this box, as instructed; the new host scenario remains unqualified until
the operator's reviewed landing gate. The preliminary core 5b run covers
prior corrections only. No .yard project configuration changed.
