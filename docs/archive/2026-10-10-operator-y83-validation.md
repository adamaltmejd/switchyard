# Y-83 operator validation, 2026-10-10

## Candidate and integration

Validated candidate: ed07ed30aa3d7930c75b6cdb91b87a9f7dc9ec85 (ticket revision 10).
Admitted target base: f9f4602f1b1f7df7abc0019c9ab55e8e4231d1db.
Proof: e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855.

Approval 71 allowed the queue to return the ordinary architecture conflict
and provide its supported target bundle. Merge 2f8ee592 preserved Y-58's
MCP code and assertions and Y-83's capacity and ownership controls.
The operator compared all guarantee rows with the required parents and the
complete merged architecture with an independently prepared union.
Later repairs added gate-policy admission, initial target-load admission
and truthful stop refusal.
No operational configuration was changed.

## Host evidence

The exact candidate passed all 50 host scenarios in 172.57 seconds.
Log: /tmp/switchyard-y83-sync-invalidation-repair-host.log.
Previous c2 passed50 in178.08s before the refused-sync invalidation repair;
log /tmp/switchyard-y83-explicit-start-repair-host.log.
Previous7f passed50 in252.04s before the explicit-start publication repair;
log /tmp/switchyard-y83-queue-policy-repair-host.log.
Previous4f passed50 in208.80s before the queue-policy/default-input repair;
log /tmp/switchyard-y83-initial-load-fixed-host.log.
The earlier 3b candidate passed all 50 in 170.82 seconds before the initial
landing read repair; that result does not prove the later change.
The isolated checkout was restored clean after the sabotage checks.

The retained-owner stop scenario failed in 14.01 seconds when only the
previous stop implementation was restored. The CLI acknowledged stopping
where the observer required an ownership refusal. Its same-project allowed
waiter-stop control completed first.
Log: /tmp/switchyard-y83-owner-stop-sabotage.log.

The gate-policy admission scenario failed in 9.81 seconds when only the
previous queue implementation was restored. The external store observed no
reserved landing owner at the held policy read.
Log: /tmp/switchyard-y83-gate-guard-sabotage.log.

A held reserved-preparation stop probe reproduced a lost stop on a67a8e6.
The repaired scenario now belongs to the full suite. An operator abandonment
probe passed on cec10d6 in 7.95 seconds and failed on previous 9b013b3 in
11.57 seconds. No public model endpoint, binary barrier, sleep or retry was
used in these host scenarios. Historical old-binary ownership uses a real
held child, daemon death and repeated startup; ambiguous ownership remains
reserved and visible instead of being inferred from missing new locks.

## Decisions and limits

Passing reviews did not override exact controls. Integrity rejections fixed
record-once intent resolution, abandonment outcome, unreserved policy Git
and acknowledgement of a stop without a deliverable handle. The last repair
also corrected a stale sabotage comment without changing its assertions.
Source inspection supports the repeated-resolution guard; there is no claim
of a runtime race held between its proof and final transaction.

Review 1212 passed with an initial landing target-config Git read outside
the reservation and inherited lock. The operator rejected exact 3b at
approval 139 as a G4/G5 integrity exception. Repair b9 moved this read into
existing preparation capacity. Its full run ended 49 pass, 1 fail in337.02s:
the FIFO observer had consumed the start event before a future-only wait.
The child timed out, and the fixture cleaned up without intervention.
The operator's static Gate initially missed this ordering gap.

Revision9/head4f uses the existing seen-or-future helper for the unchanged
exact event. Runtime, spec and configuration are unchanged from b9.
The initial-read omission probe restored only the previous queue source.
It captured the same Git state while held and asserted that snapshot after
completing the allowed gates and recorded landing. It failed in10.36s:
a prepared landing ref existed where the initial boundary requires none.
Log: /tmp/switchyard-y83-initial-load-sabotage.log.
This is a qualified operator probe with cleanup ordering, not an unmodified
committed-scenario run. Expected value and other assertions were preserved;
the exact source and tests were restored clean after terminal failure.

Review1222 passed with a P2 queue pre-admission policy Git omission, a P3
allocator limit-read ownership gap and a self-healing stale-row warning.
The operator rejected exact4f at approval140 for the two guard gaps; the
stale-row warning remains nonblocking. Repair7f moves the queue policy into reserved initial admission and reads
allocation limits from the validated default entry in the existing map.
Startup/scheduler/sync and guarded landing mutations publish it under
canonical; unavailable input refuses allocation. The allocator launches no
Git before reserving. No read-only exemption or control weakening was added.

The extended scenario observes every post-approval policy Git call, including
unowned calls. A default switch and lowering sync leave main's frozen
limit at2 while the current default1 excludes the first queued policy read
until the held worker ends. Allowed continuation and all previous assertions
remain. Restoring only old queue::next failed the exact committed scenario
in13.18s: policy acted before capacity. Log
/tmp/switchyard-y83-queue-policy-sabotage.log. Exact7f source restored clean.
This establishes that named omission; no separate runtime fault of only sync
publication or wrong-target limit selection is claimed.

Review1227 passed with one P2: explicit attempt start validated the default
through load without publishing the entry allocation reads. Source confirmed
its canonical lock already held. The operator rejected exact7f at approval141
for a narrow shared-input binding repair; no runtime stale-input race is
claimed. Repairc2 changes one runtime line to publish and use the validated default
under the already-held canonical lock, before either admission or release
for a live start exit. All50 assertions/scenarios remain unchanged.
The exact stale/unavailable start-input race has source binding evidence;
no externally held point was found that excludes scheduler refresh.
Review1232 passed with P2 premature sync invalidation and P3 transient reap
race. Source confirmed consume refusals could clear the published default
without changing canonical state, making a capacity waiter end red:error.
Approval142 was rejected for this fail-safe binding; the reap warning remains
nonblocking. Repair1233 immediately timed out without code or test evidence.
Attention143 permitted edit; revision10 renewed the clock through that exit
and delivered the same focused repair. Repair1234 produced ed07ed3.

The repair removes unconditional sync invalidation. Checkout-only consume
preserves the published input. Import invalidates at canonical update_ref;
a completed failed CAS refreshes actual state under canonical before refusal.
Default selection invalidates at symbolic_ref. Uncertain mutation or
validation outcomes retain unavailable input. Successful sync still publishes
the actual default before returning; no stale fallback or allocator Git is
added. All50 assertions/scenarios are unchanged. The narrow refused-sync
publication race has source evidence, with no claimed held runtime proof.

Review1238/check664 passed with no findings. Exact candidate gates661-663
passed fmt, clippy and build. Actual show/diff were read; the diff was
unchanged from the host-tested candidate. Approval72 bound exact ed07ed3
and the proof above. Landing1239 reran its gates; host e2e1243 passed all
50 scenarios in206.31s (execution.ended seq4277). Intent seq4278 preceded
the canonical effect. landing.recorded seq4280 names exact ed07ed3;
ticket.done seq4281 and attempt.ended seq4282 followed. Cleanup1244 ended
removed at seq4284. Supported sync consumed ed07ed3 into the clean operator
checkout; sync seq4285. Y57 dependencies are now landed. Y81 remains parked.
The operating 0.0.3 daemon, Sol 6.1 medium requesting fast tier,
DeepSeek reviewer and project configuration remain unchanged. Y-80 still
requires the operational upgrade rehearsal; ambiguous historical ownership
is not waived by this candidate's tests.
