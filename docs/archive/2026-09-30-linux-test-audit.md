# Linux test load audit, 2026-09-30

A cost-driven test-audit sweep from `9f5a82a`. The operator asked for less
test load on Linux, meaning fewer or smaller scenarios. The binary is
unchanged.

## Where the time went

Host: yard-sthlm, a 4-CPU LXC, rootless podman, pinfold 0.0.9. The runs
used the default four test threads.

- Before this pass, the suite at `77d032d` ran 47 scenarios in 152 to
  160 s. The tests summed to 581 to 613 test-seconds. The longest single
  test took 31 s.
- The suite is throughput-bound. The sum divided by four is about equal to
  the elapsed time, and CPU averaged about 80% busy (vmstat, 2 s samples).
  More threads cannot help much, so per-test work is the lever.
- A bare `pinfold box up` reaches `ready` in 0.38 s on an idle host (four
  runs). It is not the cost. The cost is the work inside boxes (implementer
  and seat Pi runs, boxed gates) plus a daemon and project per test.
- G2 took 129 s and G8 124 s, 43% of the total.

## What changed

The operator approved the shrinks, which keep every Shown-by clause, and
the sweep deletions. They also approved four cuts. Four reviewers edited
in separate worktrees, one commit per test file.

| Commit | File | Disposition |
|---|---|---|
| 152944f | G3 | Cut: the after-`update-ref` crash test. The retire test's landing gate runs on the host. The `red` count no longer filters on a reason token the spec does not name, and the empty-attention twin is gone. |
| 3d0ec7b | G2 | The new-commit, ticket-edit, synced-change and proof-only tests are one chain. Cut: the manual-sync withdrawal test. The abandon test drops its unused candidate gate. Five twins are deleted. |
| df5fee9 | G8 | The linked-skill test merges into the Codex seat test. `max_rounds` goes from 2 to 1. The limit test's early gate is dropped, so the synced change adds a gate. The gate-error test runs with review `none`. Cut: the P2 below-`blocking` round. Two blocks under the bar are deleted. The abandoned gate's "records no check" is now read from the store. |
| 1dfe4e4 | G14 | Cut: the proof entry-bound case. The gate runs on the host. |
| 34b7111 | G1 | Commit and probe run in one model turn. |
| 98a4e1c | G5 | Two twins are deleted. The host-gate crash test ends at the re-queue. |
| 187380b | G6 | The two tests share one machine, and both proposals go in one turn. |
| 8db79cc | G7 | The watch test has no gate and no seat. Its drain is armed at the abandon. The weak idle case, a taut comparison and two racy negatives are deleted. The route check now requires the key only the injecting route adds. |
| aada7b8 | G9 | The edit and stop tests share one machine. The gate runs on the host, and the held worker holds its opening. The sessions test has no seat. Two twins are deleted. |
| 20c59c1 | G4 | Two twin live-attempt reads are deleted. |
| e7ae2fe | G10 | Cut: the third candidate. The conflict test's gate runs on the host. |
| faf4108 | G13 | The box-gate/seat and host-gate proof tests are one project. The superseded landing is observed at its end. |
| 2c6d9ea | G11 | The diverged-sync test folds into the yard test. Gates run on the host, and the synced-change check asserts only the new gate. |
| 49ad8c2 | G15 | The mint-order test folds into the plan test. The dependent child is the write control. The parked twin is deleted. |

## Operator decisions

The operator approved each of these, and each spec row now matches what
remains.

- **G2.** A sync that sets `approve = "manual"` is now shown only through
  the abandon scenario: the approval is withdrawn and nothing is raised.
  The raise and no-rerun half is shown by the protect-path sync. Nothing
  now fails if `approve` is folded into the gate digest.
- **G3.** "The daemon killed after `update-ref`, before the landing is
  recorded …" is removed. The held-`update-ref` restart takes the same
  record-once branch.
- **G10.** The row now says "Two approved candidates". "The third lands on
  the moved target with its own gate run" is removed; G3 and G13 land a
  gate over a moved target.
- **G8 and G14.** The P2 boundary goes; P3 findings still pass in G7 and
  the seat scenarios. The G8 row text is unchanged. "Or passes the proof
  entry bound" is removed from G14.

The reviewers proposed further cuts rated MEDIUM or HIGH risk, and cuts
that break the positive-control rule. None was offered to the operator or
taken.

## Result

| Measurement | Before (`77d032d`) | After |
|---|---:|---:|
| Scenarios | 47 | 36 |
| Linux suite, 4 threads | 152 to 160 s | 123 s, 125 s |
| Sum of test times | 581 to 613 s | 452 s, 460 s |
| Longest test | 31 s | 52 s (the G2 chain) |
| E2E test lines | | +691 / -1071 |
| Binary lines | | 0 |
| ARCHITECTURE.md lines | 814 | 814 (4 rows edited in place) |

Both after-runs were green, back to back, on a host whose load average was
about 5 at the start. The G2 chain now bounds any gain from more threads.

## Doubts carried forward

- **G7.** The "armed on an idle board keeps waiting" block catches its
  sabotage only if the watch polls before `ticket unpark`. The race
  predates this pass. The drain sabotage fails only at the watch's 600 s
  deadline.
- **G15.** Pi can issue parallel tool calls, so proposal order is not
  fixed. The test accepts by title. Accepting in attention order failed
  once.
- **Sabotages.** The G8 abandoned-gate check, the G7 route key, the G15
  scoping and the G13 superseded-landing sabotages were checked by reading
  the code, not by running them.
- **macOS.** Not run on the Mac. The Mac's recommended four threads should
  be re-measured there.
