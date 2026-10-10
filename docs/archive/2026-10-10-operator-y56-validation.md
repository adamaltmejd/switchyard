# Y-56 operator validation — 2026-10-10

This record covers serial attempt target binding. Release readiness,
publication and operational switchover remain separate work.

## Candidate corrections

The first candidate was cafc88fbd6a70e94b2176cc500a8457cd6f7a44e.
It passed 40 host scenarios in 170.40 seconds. A focused real host fixture
then reproduced the missing G15 parent edges. Main made plan writable;
feature/a made the same workflow read-only. The parent targeted feature/a.
Accepting its proposals produced [] instead of the fixture's [3,4].
The unchanged assertion failed in 30.28 seconds. Only fixture setup changed;
no binary, store or assertion was edited. Probe source was restored.

A static audit also found that old live stopped:limit payloads lacked the
new branch identity. Backfilling only the attempt made a correct override
stale. The operator added both findings to Y-56 revision 4. Yard stopped
review 1122 and started implementation 1123.

Revised head 2ce60ab342149f19711494f411adfab8cbb3ca3e passed all 40
host scenarios in 148.74 seconds. Source and assertion audits passed.
DeepSeek check 594 then found an ended legacy planner's pending proposal
refusing its unknown target, and default selection evicting a live branch's
config before publication. Yard started repair 1128. The review also named
an obsolete git helper comment.

## Latest validated candidate

Head bcadef5b2b51f0523fac3708846becb121792245 fixes both defects.
Only ended unknown-target proposal classification uses the current default.
Their stored history stays unknown. Known targets retain branch config,
and new children retain omitted target overrides. Default selection keeps
branch cache entries; normal branch/head refresh remains authoritative.
The stale comment is removed. The binding spec changes with the behavior.

The new G15/G8 fixture builds a real pinned pre-target Y-82 binary.
Its real daemon and boxed workers create an ended planner's open proposal
and a stopped-at-limit candidate. The fixture observes quiescence, stops
the old daemon, and starts the candidate binary on its isolated state.
Acceptance preserves the parent edge and unknown ended target. Approval
preserves the old pending head/proof, records a blocking override and lands.
No fixture writes the store. CI fetches the pinned source history.

Read-only source and test-audit Gate reviews passed. The exact candidate
passed all 41 real host scenarios in 185.49 seconds. Initial historical
binary compilation took 26.78 seconds. Candidate fmt, locked all-targets
clippy and build passed. DeepSeek review 1132/check 598 passed with nonblocking findings.
Approval 69 bound the exact head and empty proof digest
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855.
Landing execution 1133 passed fmt, clippy, build and all 41 host
scenarios in 188.22 seconds. The intent was event 3925, landing.recorded
was 3927, ticket completion was 3928 and attempt completion was 3929.
Cleanup 1138 ended removed at 3931. Sync consumed the recorded head.

Logs:

- /tmp/switchyard-y56-host.log
- /tmp/switchyard-y56-plan-probe.log
- /tmp/switchyard-y56-repaired-host.log
- /tmp/switchyard-y56-review-repair-host.log

## Remaining release evidence

Y-80 revision 5 requires real operational 0.0.3 backup/restore and repeated
reopen, including both pending decision shapes. The pre-target test does
not replace that final upgrade rehearsal. Required arm64 and Mac host
validation, final cleanup/spec pass, release artifacts and handoff remain.
Y-81 stays parked. Nothing is pushed, published, installed or switched.

## Nonblocking review findings

Check 598 named startup failure when canonical symbolic HEAD is invalid,
silent scheduler skips when target config cannot load, workflow edits for
a missing target, and an escaped apostrophe in an attempt doc comment.
These were landed over under yard-drive's nonblocking default. The startup
concern is availability under invalid canonical state; no identity guard or
fail-safe is bypassed. Scheduler visibility and the doc comment remain for
the final spec/cleanup pass. Workflow-only edit already propagates
load_target's missing_target reason and branch identity. The review's
suggested alternate reason therefore exists; no default-config fallback
was introduced. These dispositions do not claim runtime reproduction.
