# Y-134 landing release observation repair

The host landing gate for c636696 reported 78 passing scenarios and one
failure: G4's `a_returning_repair_takes_a_freed_lane_before_a_new_ticket`.
The supplied failure includes audit-row comparisons through the landing's
terminal red event and approval retirement.

The trace shows landing execution 3 releasing at sequences 19 and 22 while
running its child gate. It reserves again at sequence 26 and ends red at
sequence 27. The fixture used `Watch::find` to select a release by execution
identity alone. That search includes previously observed events, so it could
select sequence 19 instead of waiting for final checked release. Its audit
snapshot could then race that final release.

Disposition: retain every G4 assertion. Require the selected release to have
a sequence after the observed terminal landing event. This changes only the
fixture's synchronization. It adds no sleep, retry, runtime seam or binary
barrier. The provenance implementation and current architecture stay intact.

Test-audit Gate answers:

1. G4's existing returning-repair and pending-adjustment clauses remain the
   contract. Their assertions still observe the real held lane, refusal
   without store writes, repair priority and successful adjustment control.
2. Existing sabotage remains: schedule ready admissions before returning
   repairs reacquire, or remove the pending-return adjustment guard. The
   repair also documents why an intermediate release cannot settle ownership.
   Selecting any release without terminal ordering restores the snapshot race.
3. No new assertion block or scenario is added. This fixture already holds
   the sole lane while a failed landing returns for repair. Other revision
   scenarios hold executing work rather than this pending return. The host
   trace exposes the intervening parent slot releases that its old observer
   confused with final release.
4. The existing expectations still come from fixture latches, host box
   inventory and externally read store rows. Status history supplies the
   supported wait seam; terminal-before-release ordering selects the settled
   point before comparing unchanged audit rows. No digest or copied Yard
   computation supplies an expected value.

One test line is replaced; no assertion or binary line is removed. This is
the Gate for a synchronization repair, not a release sweep. No guarantee or
Shown by clause changes.

Validation: `cargo fmt --check`,
`cargo clippy --all-targets --locked -- -D warnings` and `git diff --check`
pass. Per Y-134, host e2e is left to the post-approval landing gate; this
clone has not rerun it. That gate must exercise the repaired observation.
