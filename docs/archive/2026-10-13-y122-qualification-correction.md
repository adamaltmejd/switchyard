# Y-122 qualification correction

The operator qualified candidate 316b756 on the real host using the existing
authorized isolated FUSE profile. These were not native-overlay trials.
Evidence is in
`/home/yard/validation/y122-316b75-host-20261013/operator-report.md` and the
phase receipts and logs beside it. The results below are operator-reported.
They qualify the earlier candidate and temporary mutations, not this repaired
head. Earlier dated records remain unchanged.

The pending-repair control passed, 1 passed and 0 failed. Removing only
`(current.next.is_some() && !implementing)` from `attempt_revise` failed
directly at g04:278: refusal was Null instead of `refused`. The later identity,
unchanged snapshots, lane-priority and running-repair blocks were unreached.
Exact source restoration followed by unchanged control passed, 1/0. The
candidate head observation excludes masking by the adjacent eligibility
guard. The pending-repair observer and additive G4 table clause stay unchanged.

The independent first-race qualification removed only its weak results,
refusal-vector, count and code block. With production intact it passed, 1/0,
in 9.61 total seconds. With that same fixture, dropping exactly the
`attempt_one_live` unique index, the live-attempt clause of `tickets::BLOCKER`
and the live check in `admit::attempt_start` failed directly at the retained
first live count, temporary g04:99: 2 versus 1, exit 101, in 13.02 total
seconds. The later capacity-race assertions were unreached. All sources were
restored exactly. Unchanged control passed, 1/0, in 16.21 total seconds.
Runtime inventories were empty.

This correction deletes the qualified weak first-race observer and its orphan
locals. Both concurrent starts of the same ticket and the first external
live-attempt count remain. The fixture-held control and complete separate
last-lane refusal, capacity reason and external count remain. Four executable
lines become one invocation that discards the response: net three test lines
removed, zero binary lines removed. The sabotage comment names all three
qualified guards and the direct retained count failure.

Test-audit Gate: G4's Shown-by clause says a ticket never has two live
attempts. Racing two starts of one ticket with both lanes free isolates that
boundary from capacity. The read-only store supplies the actual live rows;
the spec supplies the maximum of one, and the real fixture holds the work.
The last-lane race observes project capacity and cannot prove per-ticket
uniqueness. A generic refusal response does not independently prove this
clause, and the contract specifies no duplicate-start reason token. The
qualified three-guard mutation makes the retained oracle fail directly.
No guarantee or coverage-table clause is removed or weakened.

The correct board dependency is Y-111 depends on Y-122. This correction makes
no board or production change. Boxed fmt, locked all-targets clippy and build
are required before commit. No e2e, sabotage, sleep or retry runs in this box.
The operator will review the repaired exact diff and selected checks before
its real landing gate; this repaired head has no claimed host validation.

All three boxed checks and git diff --check passed for this correction.
