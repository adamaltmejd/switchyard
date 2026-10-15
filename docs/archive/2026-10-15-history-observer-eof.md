# History observer EOF, 2026-10-15

The real grouped-finding case at Y73 candidate 530fe30 aborted its daemon
with a stack overflow. The history CLI reported that the daemon hung up.
The test then waited its full 600-second event deadline and failed after
669.96 seconds. Candidate 15289c4 reproduced the abort and the same wait.
Both original runs and fatal output remain retained under
/home/yard/validation/y73-*-groups-20261015-evidence.

The history-reader thread silently ended at EOF. Watch retains another
sender for later FIFO observations, so that sender prevented receiver
disconnection. No future history event could satisfy the wait.

The existing observer now sends Closed when its history stream ends. Both
event waits and held-call waits fail immediately on that observation. The
reader returns if its receiver was already dropped. No process is killed,
restarted or retried. No deadline, scenario, assertion or product code changes.
This removes the silent closed-stream path to a later timeout.

Test-audit Gate: this is observation plumbing for the existing G2/G5/G9
crash and recovery cases. Their held points, external Git/box/store values,
allowed controls and sabotage comments remain unchanged. A closed observer
cannot establish any later guarantee, so it must fail rather than wait.
No new fixture, historical build, box, model turn or assertion is added.
Real host full-suite validation of the integrated source remains required.
The daemon stack failure itself is not repaired by this observer change.
