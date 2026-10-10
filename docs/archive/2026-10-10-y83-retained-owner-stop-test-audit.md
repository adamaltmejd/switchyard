# Y-83 retained-owner stop repair, 2026-10-10

The operator reproduced review1207/check641's P2 on exact candidate
59a05db12617da6ec327b942c4bbae5796177651 through the real CLI. The isolated
legacy host probe used the existing real old-binary held-child/restart
fixture. After child release and second startup, attempt stop Y2 answered
stopping=2 while execution 2 remained running, reserved, lock_contract=0,
with red:ownership and no current stop handle. The operator reports exit
101 in 13.04 seconds, log /tmp/switchyard-y83-owner-stop-probe.log; probe
source was restored clean. These are operator observations, not worker-run
e2e. The reported full 50-scenario pass in 174.19 seconds did not cover
this acknowledgement.

attempt_stop now captures the current notification handle and checks the
current execution row in the existing transaction. A reserved row without
that handle refuses with reason ownership, ticket, attempt, execution and
any open ownership-attention identity. Refusal precedes all stop mutations
and the successful attempt.stopped audit. It neither guesses quiescence
nor releases the reservation. An unreserved waiter still ends unchanged;
a reserved live controller still receives its existing preparation/worker
notification. No ownership proof, external-resolution rule, abandonment
check or lifecycle is changed.

Test-audit Gate: extend
an_upgrade_retains_a_legacy_pre_box_child_without_an_ownership_proof.

- G5's shown-by clause now includes truthful stop refusal on the retained
  legacy owner. Keep the real old-binary child held across daemon SIGKILL,
  surviving PID proof, external old row, initial candidate approval,
  three-lane admission, both startups and all existing ownership and
  model-exclusion assertions.
- The admitted unreserved waiter is the same-project allowed stop control.
  Observe its successful CLI answer, stopped:unchanged identity and ended,
  unreserved row. Deliberately start it again and observe a fresh execution
  waiting for capacity before releasing the old child. Stopped work or an
  unadmitted ready ticket does not stand in for the exclusion proof.
- After child completion and second startup, refuse attempt stop Y2. The
  expected execution identity is the fixture's old execution 2; attention
  identity agrees with the current supported status reading. External
  store snapshots prove execution and attempt unchanged. The audit has
  no successful attempt.stopped for execution 2. Existing current
  red:ownership and retained reservation observations remain intact.
- Acknowledge without a current handle fails the refusal assertion. End or
  release the retained owner on refusal fails the unchanged-row block.
  These sabotages are named in the comment. Existing G4 waiter/preparation
  stops and G9 acting-worker stops cannot expose a retained controller
  without an in-memory handle after legacy migration. No new scenario or
  duplicate exclusion block is added.

Correct the G3 held-update-ref sabotage comment without changing assertions.
Skipping decide_intent's lock_free alone cannot bypass the preceding
capacity::release proof. The comment now names omitting the landing Git
child's inherited host lock: box absence and the free lock then falsely
prove ownership gone while the fixture holds the real child. Both recovery
lock checks see that same absence. No legacy check is deleted. This
comment correction is source reasoning, not a worker-run sabotage probe.

Assertion lines removed: 0. Binary behavior removed: successful stop
acknowledgement/audit for a reserved implementation with no deliverable
handle. No new owner, timeout, clock wait, retry, binary hook, unit seam or
machine-wide limit is added. All Y58 integration, Y83 capacity, preparation,
abandonment and once-only integrity repairs are preserved. Repository
.yard and frozen daemon/reviewer settings are unchanged.

Worker validation: cargo fmt --check, cargo clippy --all-targets --locked
-- -D warnings and git diff --check passed. Host e2e was not run here, as
instructed. Fresh review and full exact-head host validation remain required
before approval.
