# Y-157 boxed Git worktree binding — 2026-10-15

Review3717/check1943 reproduced predecessor `core.worktree` redirecting
`commit_source` and final packaged status to a substitute tree inside `.git`.
Cwd alone did not bind Git to the admitted writable source. Both command
paths now pass `--work-tree=/workspace` before their subcommands. This
overrides the repository setting for discovery, add, commit and inspection.
Existing output/time bounds, scrubbed environments, hook/filter/fsmonitor
and signing controls, strict filter decoding, unrepresentable-key refusal
and pre-status Gitlink refusal remain. ARCHITECTURE.md records the explicit
worktree binding. No package protocol, grant, checkpoint or lifecycle changes.
G13 Shown by names the representative alternate-worktree control.

## Test-audit gate

Extend the retained G9/G13 selected cleanup hard case in the same project.
The ordinary implementer first creates its real committed source, then
archives that commit under `.git/alternate` and configures `core.worktree`
there. Both trees initially match so the ordinary candidate can finish.
At the existing initial cleanup hold, the host writes `substitute` bytes
only into that alternate tree. The source-write debugger, pending unsafe
intent, crash, checked release, continuation marker and checkpoint identities
keep their existing observations and expected values.

At the existing post-commit report hold, a scrubbed boxed native Git probe
must read `cleaned` from the actual committed `HEAD:chain.txt`. An unpinned
native status with hooks/fsmonitor and the fixture filter disabled must
report the alternate `chain.txt` as modified. This is a positive control
that redirection is effective and still differs from committed source.
The production final inspection must nevertheless accept the actual clean
source and reach the existing repair/approval/landing observations. Repair
removes `core.worktree` before its ordinary commit. It retains all existing
helper-marker, trusted-input, proof, ancestry, inode, publication, canonical
source and nested Git assertions.

The post-commit request now signals a fixture FIFO before returning its
existing held response. The host uses `Watch.said_before_end` with the exact
replacement execution instead of waiting only on the inspection latch.
Commit failure therefore fails promptly at the execution's end, before the
new probe. The `request` token comes from that scripted model branch; the
held response still needs the existing explicit release. No deadline changes.

The material failure is committing substitute bytes or accepting/rejecting
cleanliness for a different tree from the declared writable source. The
test comment names omission of either explicit worktree binding as sabotage.
Without commit binding, the new HEAD bytes assertion fails. Without final
inspection binding, the dirty alternate tree prevents the retained successful
handoff. Expected bytes come from the model and host fixture; expected status
comes from native Git over those different bytes. The existing allowed
write/commit and successful handoff remain the same-project allowed control.

No new scenario, machine, image, build, worker box or model turn is added.
Added work is one ordinary fixture archive, one FIFO observation and one
boxed native Git probe.
No assertion is relaxed or removed. The existing difficult case is sufficient;
there is no configuration matrix or generic Git wrapper. Test diff: 47 lines
added, 5 replaced. Production diff: 2 lines added, 0 removed. This is a
test-change gate, not a suite sweep.

## Validation and evidence

A separate local native Git probe reproduced the setting and verified that
explicit worktree add/commit records `cleaned`, unpinned status reports the
alternate file modified, and pinned status is empty. It ran no Yard daemon,
worker, package or model code and is not real-box acceptance.

The operator reports the exact c314449 whole affected case passed 1/1 in
66.11 seconds suite, 69.52 seconds wall. Evidence remains unchanged at
`/home/yard/validation/y157-c31444-durable-20261015-evidence`. That pass
establishes the prior write/restart/continuation/repair/landing and nested
controls, but does not cover this worktree finding. All earlier failed
evidence and the e1b93d interrupted diagnostic remain separate.

No e2e runs in the candidate box. The repaired complete affected case and
prepared full 50-case host gate remain required. The host-owned landing
conflict return supplies the unavailable target update after qualification.
No canonical access, runtime/config change, new ticket or release action is
requested.

`cargo fmt --check`,
`cargo clippy --all-targets --locked -- -D warnings` and
`cargo build --locked` passed. No repaired real-box pass is claimed.
