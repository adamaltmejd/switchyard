# Y-132 accumulated join Gate

Disposition: strengthen the existing multi-review scenario in
`g04_capacity_holds.rs`. Move operator handling from the blocking source to
the required empty join. The source keeps default automatic handling.
G2 and G8 require exact operator handling of an accumulated blocking result;
the picker cannot bypass it because the join's own verdict passes.

Sabotage: use only the join's own check verdict to decide operator handling,
or keep automatically repairing carried blocks after exact join acceptance.
The changed fixture requires candidate attention for the join, a passing own
check and a blocked result carrying exactly the required source check. The
existing stale-head refusal, exact repair, host-observed selected groups and
exact acceptance remain the same-project controls. Acceptance must reach
approval with one repair, retaining original failed evidence.

Expected source failure and empty join come from fixture publications. The
external store supplies the required source check identity. Attention's exact
result and input identity are observed through the host CLI history. Git
supplies the candidate heads, and the existing host model observes repair.

No new machine, build, project, box or model turn. This replaces the easier
own-verdict operator case within the existing hard case. The edit replaces
4 test lines beside 3 binary lines. No assertion block was pruned. No suite
sweep or runtime measurement is claimed. E2e and sabotage execution remain
deferred to the sequential host landing gate after approval, per instruction.
