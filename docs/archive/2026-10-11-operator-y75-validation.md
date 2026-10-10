# Operator validation of Y-75, 2026-10-11

Y-75 normalizes Pi usage from the new execution stream, including partial
observations retained through interrupted resume. Claude and Codex advanced
accounting remain separate tickets; no bill is inferred from provider tokens.

## Source and assertion judgment

Attempt 70, ticket revision 3. Base
`37f61cacb389c626a956c5166463028390b35984`.
Candidate `5dcb76cbd48b54cf85865f051098ddf43e4f8e59`, empty proof snapshot.
The operator read the complete 553-line diff, protected architecture and
48-line assertion audit. Source and test-audit gates passed.

Pi counts assistant message_end usage once, excluding turn_end copies and
session history. Each reported field accumulates an optional increment with
harness provenance and partial completeness. Missing coverage is sticky;
settlement marks complete only on success with full coverage. Absent fields
stay null. The daemon persists observations through the existing execution
row before settlement. Resumed Pi completeness is preserved; the existing
unknown guard still applies to resumed Claude and Codex.

One new G7 scenario exercises real Pi against a scripted host model. The
host pinfold wrapper removes measurement fields from real harness frames,
preserving child behavior, stderr, registration and exit status. The first
completion has no usage; a resumed turn reports one message then waits on
the held model response. Real box down drains the reported frames and the
ended record retains fixture tokens with partial coverage and null cost.
The daemon reopens after observing the ended record, with the reply still
held, and preserves the record. A later resumed execution with nonzero
session history reports literal zero and complete provenance.

Expected measurements come from the host fixture, removed fields and real
Pi frames. This exposes distinct Pi omission/interrupted-resume mechanisms
not shown by ordinary fresh Pi, Codex omission or historical migration.
No assertion is removed, no lifecycle or ownership control is weakened,
and no binary barrier, sleep, retry, mock or public endpoint was added.

## Exact-head validation

Candidate checks 741-743, executions 1408-1410, passed fmt, clippy and build.
An isolated clean checkout at the exact candidate ran
`cargo test -p e2e --locked`: exit zero, 59 passed, zero failed, 236.02s.
Log: `/tmp/switchyard-y75-5dcb-host.log`. Post-run inspection verified the
same clean candidate head. No isolated host suite remains active.

## Initial review, decision and repair

Correctness review 1411/check744 passed round 1 with no findings.
Protected attention 166 named the exact candidate, proof, revision 3 and
checks 741-744. Fresh full diff byte-matched the read candidate. Approval
waited for the larger Y-88 slice to land; that slice was actually consumed
and its operator record imported at `b2be134dac3f3805752dd4b910f8abf14ac3d7f0`.
The operator reread current checks and identities, then approval 82 used
the read head/proof. The next command rearmed watch.

Landing 1418 ended conflict in `docs/ARCHITECTURE.md`. Yard retired approval
and started repair 1419. The first candidate's validation is not evidence
for the repaired head. Read the full repair delta, rerun exact-head host
validation and read fresh review before the next decision.

## Repair source judgment

Repair 1419 produced `f5e3d6363697638a71a71ef5555e73af71bd391d`, based on
actual target `b2be134dac3f3805752dd4b910f8abf14ac3d7f0`.
The operator read all shared repair hunks: architecture preserves both
usage and execution evidence clauses; execution storage retains Pi's
resume coverage with Y-88 bindings and result columns; supervision keeps
partial observations alongside infrastructure outcome evidence.
Git blob comparisons confirm all exclusive adapter files are unchanged
from the first candidate, and all exclusive inherited files match the
landed target. Source and assertion gates passed; no assertion was lost.

The isolated clean checkout at the exact repaired head ran
`cargo test -p e2e --locked`: terminal exit zero, 59 passed, zero failed,
298.31 seconds. Log: `/tmp/switchyard-y75-f5e-host.log`. Post-run Git
inspection confirmed the same clean repaired head. No isolated host suite
remains active. Fresh checks 746-748, executions 1421-1423, passed fmt,
clippy and build. Review 1424 remains active; its verdict is pending.

## Fresh review and protected decision

Fresh review 1424/check752 passed with one P3 coverage finding: the Pi
fixture uses zero cache tokens, so it cannot detect removal of cache reads
and writes from the input sum. The source correctly includes both; no
runtime parser failure is demonstrated. This is carried coverage, not a
fixed finding or permission to leave a release clause unshown.

The operator attached the exact finding to the existing final Y-80 audit,
preserving its full original migration/release body and dependencies at
parked revision 6. Full byte readback verified that change. The required
final audit must prove the existing G7 sum clause with real pinned Pi,
supported nonzero cache fixture values and external expected totals, while
keeping omission/interruption/resume/zero controls and avoiding twins.

Fresh full diff matched the read repaired candidate. Protected attention
168 at seq 4951 named f5e, empty proof, revision 3 and checks 746-748/752.
Approval 83 used that exact payload; the next command was watch. The
below-blocking default applies. No failed test was accepted or relabeled.

## Fresh landing and consume

Landing 1433 ran against exact merged head
`f5e3d6363697638a71a71ef5555e73af71bd391d`, identical to the repaired
candidate, with old target `b2be134dac3f3805752dd4b910f8abf14ac3d7f0`.
Fresh executions 1434, 1435 and 1437 passed fmt, clippy and build.
Host gate 1438 passed 59 scenarios, zero failed, in 306.72 seconds.

The supported audit stream recorded landing at seq 4972, ticket done at
4973, attempt ended at 4974 and cleanup removed at 4976. Supported sync
consumed the actual head; the next command rearmed watch. Git inspection
confirmed the same clean exact checkout before this archive commit.
No push, publication, installation or frozen daemon replacement occurred.

Y-89's repaired candidate awaits focused G9 and full exact-head host
validation; its review is active. The required final Y-80 audit still carries
the unshown nonzero Pi cache sum clause; this landing does not clear it.
Y-81 stays deferred and parked. Full release scope through Y-80, platform
checks, cleanup and final artifacts remain incomplete.
