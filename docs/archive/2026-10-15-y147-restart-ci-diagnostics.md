# Y147 restart CI diagnostics - 2026-10-15

Run38711153136 on exact172e84b reproduced the unresolved restart refusal.
Linuxx64:49/50 in499.89s; G2 candidate_reuses_original_pass_after_bookkeeping_with_unknown_rerun
printed anotherdaemonrunning and failed serving readiness. Arm64 passed50/50
in397.35s. Both ran Pinfold0.2.3. The selected Durable cleanup case passedboth.
Original run38491717533 remains evidence; neither run identifies actual errno
or holder. This preparation is not a fix or ticket completion.

Reapply only the three source files from the reviewed failure-only preparation
bdf4b0bc7abcee734945b534b3ebc4f17c74a9c5 onto actual integrated172e84b.
Normal singleton locking is unchanged. Refusal now records its actual errno,
device and inode. The existing stdout reader reports bounded host metadata
only when startup closes stdout before saying anything. It reads matching
/proc/locks rows and descriptor metadata, PID/start/parent identities and flags,
never argv or environment. The kill helper still kills/reaps the same owned
child, recording its reaped status. Before-kill/after-reap scans stay absent.
No new dependency, lease, test-only binary flag or runtime change.

Test-audit gate: no assertion or scenario changed or added. Same real daemon,
boxes, model, workload, concurrency, clocks and allowed/refusal controls.
The observer cannot turn a failure into a pass and runs no alternate binary
path. No new automated guarantee is claimed, so no new sabotage assertion or
coverage matrix. Existing architecture/controls remain unchanged; only
failure evidence is richer. The 155-line host observer is diagnostic-only
and must not be treated as a production fix or required final release surface.

Fmt, clippy all-targets locked with warnings denied, and build locked passed.
A full50-case host validation and exact temporary-branch Linuxx64/arm64 CI
remain to be run. Host uses unchanged frozen Pinfold0.2.2 and the owned
shifting profile; CI uses pinned0.2.3. A passing host run establishes diagnostic
compatibility, never original-failure resolution. Preserve every failure.
