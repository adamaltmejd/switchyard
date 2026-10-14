# Y-68 image preparation integrity repair

Review 3217 identified cancellation dropping image serialization while an
inherited-lock Git child can still write the shared context. Preparation now
records the existing execution before context mutation. Failed or cancelled
preparation keeps that association. Subsequent preparation and image retirement
wait for the same execution's checked reservation release. No new owner,
capacity pool or execution lifecycle is introduced. Successful preparation
clears the association after its awaited child completes. Forget retains the
registered project until proof so recovery can still check its owner.

## Assertion audit

Keep `cancelled_image_preparation_retains_its_context_until_checked_owner_release`
in G5's shown-by row. The fixture delegates a real image-context Git clone,
records its successful exit, then holds that actual host child with its
inherited execution lock. Supported operator MCP stop ends the execution while
the fixture still holds its child. Before daemon restart, host process and flock
observations prove the surviving owner. With project capacity two, a successor
reserves its own slot after restart but waits for the original preparation.
Context inode and fixture bytes remain unchanged while that owner is held.
Fixture release permits checked ownership release and a real boxed command
completes. The original command never supplies a result or landing evidence.

The sabotage is dropping shared preparation protection at cancellation, or
discarding its association on restart. Existing held up/exec/down scenarios do
not touch a shared image context and cannot detect its deletion with spare
capacity. The teardown-child scenario remains unchanged.

Expected capacity comes from the fixture's two slots. Context identity comes
from host Git's committed input. PID, successful real Git status, inode and
marker bytes come from the host wrapper and filesystem. Host flock proves that
child ownership remains. Store reads prove both reservations and absence of
box handles. The refusal observation names `image_preparation` and the exact
original execution. Its positive control is successful successor completion
after fixture release, with completed infrastructure on its execution result.
The scenario uses operator stdio, retained-event searches after restart, and
no worker creation grant, public model endpoint, sleep, retry or binary barrier.
The wrapper preserves real Git's exit status after the hold. A delegating
pinfold wrapper records the real boxed command's exit on the host, proving no
command before owner release and successful actual execution afterward.

All image callers were read together: implementation preparation, review,
boxed gate and standalone work. They share execution reservation and inherited
lock protection. Git timeout can leave a child alive; failed image preparation
therefore retains the association and context as cancellation does. Image
retirement uses the same proof. Historical stores need no new schema or inferred
owner: the association is written only by newly admitted preparation.

Test lines removed: 0. Binary lines replaced: 9. No assertion was weakened.
The binding architecture describes the current mechanism and G5 observation.
The worker ran fmt, clippy over all targets, and build. Native e2e was not run:
the operator explicitly reserves it for the host landing gate after review.
Run this affected case serially with the existing held teardown/report cases
and required zero-ticket command/report and migration cases, then the full
required native landing gate. Candidate checks do not prove host behavior.
