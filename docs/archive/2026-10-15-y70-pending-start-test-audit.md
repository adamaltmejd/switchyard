# Y-70 explicit start cannot consume pending repair

Review3353 found that the pending choice guard did not cover explicit
writer starts. A named A start could consume B's feedback and clear next.
A named B start could consume the same feedback under retry instead of
repair, omitting the target-fetch instructions. The operator requires both
starts to refuse while automatic continuation remains pending.

Workspace and writer-attention admission now reread attempt.next and
refuse pending_continuation inside the existing atomic admission transaction.
The refusal names the attempt, current choice and complete pending value.
It precedes writer creation, choice invalidation, lane/clock allocation and
attention resolution. There is also an early read-only check at start entry:
a pending dispatch needs no configuration read to refuse a competing start.
The transactional checks remain necessary if pending work appears while
configuration is being read. Both use the same stored next; no new owner,
reservation or lifecycle is introduced.

Read together: explicit start entry and attention selection; workspace and
writer-attention transaction guards; shared atomic lane operation; scheduler
pending reason and feedback dispatch; current producer lookup; writer feedback
freezing and choice invalidation; queue return next installation; the target
fetch prompt. Automatic dispatch bypasses the operator-only guard and retains
repair, the selected producer, target and failure source. Same-candidate
operator choice replacement remains allowed and preserves next.

Disposition: extend the existing five-profile G2/G8/G9/G10 scenario's held
pending conflict. The real bundle creation arms the real scheduler Git read
before lane reacquisition. That read holds canonical, so the early refusal
also prevents explicit starts from waiting behind the scheduler. Checked
landing release and stored lane zero establish the pending point. After
existing different-choice refusal and allowed same-exact-B choice replacement,
the host tries explicit A and B workspace starts. Each must name the exact
pending attempt, new choice and unchanged instructions. Complete attempt,
execution, approval, attention and audit snapshots, plus host bundle bytes,
remain equal after each refusal. The host releases the actual Git call and
the existing automatic B execution fetches the target, repairs and earns
fresh own review, selection, checks, approval and landing.

The added comment names sabotage: let an operator writer start consume or
relabel pending repair. The choice-only control cannot expose that route;
existing post-intent refusal reaches a different fence after next is gone.
A pending returned conflict with B's actual bundle is the hard case, and
both a different writer and B can break it in distinct ways. Observers are
operator MCP, CLI history, read-only store, host files and producer Git.
Expected target/head content comes from external Git and fixture scripts.
Complete-row and byte comparisons prove agreement and absence of mutation;
no expected verdict is derived from Yard. The existing allowed same-B
choice and actual automatic B fetch/landing are the same-project control.
ARCHITECTURE.md records the explicit refusal and its G10 shown-by evidence.
The attention path shares the guarded transaction; no separate synthetic
attention or new scenario was manufactured to claim native coverage.

No assertions, original controls or guarantees were removed or weakened.
Test lines removed: 0. Binary lines removed: 0. The atomic lane fix, exact
held-CAS refusal comparisons and immutable evidence controls remain intact.
Validation: cargo fmt --check, cargo clippy --all-targets --locked --
-D warnings, and cargo build --locked passed. Native/e2e was not run here,
as instructed; reviewed host qualification remains operator-owned. No .yard
project configuration changed.
