# FIFO observation after history EOF, 2026-10-15

The e43b117 observer repair was too broad. Native grouped-case qualification
of production e3b7598 with the current observers passed the earlier native
landing crash point, then failed at g04's held FIFO in its five-writer crash
helper. That helper deliberately kills the daemon before awaiting owner-said.
Its history stream is necessarily closed while its independent FIFO is valid.
The 0/1 result in100.78s remains a failure, not a Y73 qualification pass.

The history reader still reports Closed. Event waits still fail promptly.
A FIFO wait records the closed history and continues observing the independent
FIFO. A later event wait knows the history is closed and refuses immediately.
Already observed events remain readable through find. No scenario, assertion,
deadline or product behavior changes. This preserves the crash fixture's
external held point and its independent observation channel.

Test-audit Gate: existing G4/G5 hard cases remain the positive control for
FIFO observation after daemon death. The known crashing15289c production
with e43's event EOF path failed promptly in73.81s instead of waiting600s;
that was observer qualification, never a passing product case. No new fixture,
box, build or model turn is added. Integrated normal qualification continues.
