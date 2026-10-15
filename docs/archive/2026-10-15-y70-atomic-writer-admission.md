# Y-70 atomic writer continuation admission

The operator's preserved preliminary native 5b8bce run reached the held
CAS-intent refusal. Its full attempt comparison failed only on lane zero
to one and lane_since null to timestamp. The operator retained the field
diff, read-only store snapshot, log and binary. This is a product failure:
explicit workspace start committed take_lane before start_writer checked
intent. The writer-attention start path had the same split transaction.
No fixture assertion is changed or weakened in this repair.

Both supported writer continuation paths now hold the existing daemon
admission lock around one project transaction. Explicit workspace start
revalidates the expected plan and pending timeout/edit blockers inside that
transaction. Shared start_writer rereads the attempt, refuses ended state
and open intent, resolves current writer settings and checks exclusive
workspace identity through execution creation. The same transaction then
checks current lane ownership and free lane capacity, records the lane and
clock, and resolves only the eligible attention. An error at any point rolls
back writer, choice, approval, next, attention, lane and clock changes.
Only a committed execution is spawned.

The existing take_lane entry point and these transactions share the same
store-based lane operation. It rereads current ownership before counting
free lanes; an already-held lane needs no new reservation. The admission
lock still serializes competing admissions, and the project transaction
serializes intent creation, plan/attention changes, writer ownership and
lane writes. There is no preflight-only intent fence, second reservation
truth, new owner, lifecycle, barrier or retry. Capacity failure after a
provisional writer start rolls back all its store effects. A stopped item
resolved concurrently also rolls back the provisional writer and lane.

Read together: free/spare lane readings, take_lane, scheduled and explicit
new-ticket admission, both workspace/attention continuation paths, expected
plan and blocker guards, shared writer start, execution workspace exclusivity,
choice invalidation, attention resolution, set_lane clock behavior and
project transaction rollback. The preceding 7a12414 pending-repair choice
binding remains in the same candidate history, including its held real Git
control and exact same-B repair. No .yard configuration changes.

ARCHITECTURE.md records the atomic continuation contract. The original
five-profile G2/G8/G9 scenario retains the actual held Git CAS call, exact
intent refusal, complete before/after attempts, approvals and producer rows,
and successful pre-intent continuation and actual same-producer conflict
repair. Its existing comment names removing the start_writer intent fence
as sabotage; this bug left the fence intact but committed the lane before
it. The unchanged full attempt equality detects that split. No new scenario,
harness, assertion, sleep or immediate-command window was added. The same
transaction structure repairs the attention path; this box does not claim
separate native qualification of that path.

Validation: cargo fmt --check, cargo clippy --all-targets --locked --
-D warnings, and cargo build --locked passed. Native/e2e was not run here,
as instructed. The preliminary failure stays preserved, and reviewed native
qualification remains the operator's host gate.
