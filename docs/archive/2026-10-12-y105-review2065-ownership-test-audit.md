# Y-105 review2065 ownership-refusal integrity repair

Review2065/check1088 identified a required ownership refusal that the existing
positive acceptance controls did not establish. The terminal-release repair
waits for the right release before accepting. That proves successful
continuation, but cannot fail deletion of candidate_answer's predecessor
proof: the parent is already unreserved at the call. Preserve the shared
prove_predecessors reader and every production guard; extend one existing
scenario to exercise the retained-parent window explicitly.

## Scoped test-audit gate

Extend only prepared_input_and_completed_child_survive_host_child_restart in
the existing G3 helper. The changed-target scenario uses the same helper but
does not execute the new hold/refusal branch. No twin scenario or shared
fixture abstraction is added. G3's Shown by row adds the exact ownership
stale/no-write refusal and its same-project checked-release acceptance control;
all existing guarantee and evidence clauses remain intact.

The fixture's failed middle host command writes a fixture marker then runs
real false. The daemon's pinfold PATH wrapper delegates all inventory calls.
The first listing after that marker is the failed host child's checked
release; the wrapper moves the marker to arm the next listing. The following
listing is the original parent's final ownership release, after it records
its failed child check, raises exact landing attention and ends fail. The
wrapper runs the real listing unchanged, captures its actual exit code,
then holds its reply/exit on a host FIFO. It removes the arming marker before
holding, so the acceptance proof's own inventory call delegates normally.
No reply is fabricated and no unknown inventory is converted to success.

The external history observer requires that parent's failed end after its
attention. The probe FIFO names the live wrapper PID and real inventory exit
code; kill -0 proves the probe process exists. A direct store query proves
the same original parent is ended/fail and reserved. While its final probe
is held, the fixture acquires the parent's actual host lock; a separate open
of that lock cannot acquire it. These observations prove an actual retained
reservation and held ownership/probe window, not a historical release.
The fixture holds only its own isolated project's lock and subprocess.

Call the exact yard_attempt_accept request used by the later success control.
Require stale code, identity ownership, expected {execution: original parent,
released: true}, and current naming that same execution, ended/fail,
reserved:true and the unchanged original prepared identity. Compare complete
attention and audit table snapshots before/after. The existing attention row
is the sole decision store; unchanged rows prove no resolution or acceptance
write. Unlock the same host lock and release the same FIFO hold. Then require
execution.released after the exact failed end and a current unreserved failed
row, before the unchanged successful acceptance call in this same project.

| Gate question | Evidence |
|---|---|
| Guarantee and clause | G3's failed-parent final ownership refusal and same-project released acceptance control; G2's original preparation and immutable failed-evidence bindings. |
| Sabotage | Delete admit's predecessor proof or accept while ownership is retained, as named in the scenario comment. The exact stale and no-write assertions fail on that bypass even if later queue consumption retains its independent proof. |
| Distinct hardest case | An eligible failed selected judgment is fully recorded and the original parent has ended, yet its final inventory probe and host lock hold ownership. Its earlier phase releases exist but cannot authorize this call. Passing continuation alone and consumption-time surviving-Git-owner checks do not establish answer-time refusal. |
| External expected values | Fixture false supplies fail; external store gives original parent identity/state; actual runtime listing, PID existence and lock contention prove the hold; spec supplies stale ownership and expected release; external audit sequence anchors final release; unchanged complete store snapshots prove no write. Prepared identity agreement is the required relation between the original evidence and current refusal; existing host Git observers independently prove the pinned input. |

Capture the original complete failed check/result/input snapshot before the
refusal and success calls. Retain all its comparisons through held-child
restart, continuation, landing and reopening, as well as the existing
same-head passing-child reuse, changed-target fresh children, exact stale
expected/current and no-write, preparation retirement, pinned host-Git input
and final CAS controls. No original assertion is deleted or weakened.
Test lines removed: 13, comprising the replaced gate-command line and moved
unchanged decision/snapshot/assertion lines. Binary lines removed: 0.
No sleeps, retries, deadline or concurrency changes, public model calls,
binary barriers or test-only production paths are introduced.

## Preserved evidence and limits

Actual landing2038/host2042 on 525c2c4 remains failed: 72 passed, 3 failed in
555.46 seconds. All three acceptance positive controls reported the same
original ended/fail landing reserved:true, expected released:true; no
acceptance succeeded. The operator's full tail remains at
/tmp/switchyard-y105-host2042-failed.log and its authoritative snapshot at
/tmp/switchyard-y105-host2042-terminal.json, unavailable in this worker.
The source-established historical-release observation repair remains in the
host2042 archives. This new integrity control is not a host rerun or proof
of the actual historical host sequence numbers.

Approved 474a8fe landing2058 conflicted only in ARCHITECTURE.md against
actually landed Y-102 b4b0d12. The existing 585de02 merge retains the full
Y-101/Y-102 and Y-105 semantic union, source and test assertions. Y-102's
reported host2057 pass, 75/0 in 708.77 seconds with resolved intent and removed
cleanup2059, validates its input only. The prior conflict audit records that
reported canonical validation commit 1c72d01 was absent from the supplied
bundle. yard_context still reports candidate base b4b0d12 and supplied main
remains b4b0d12. This focused repair preserves the complete available union
and claims no absent current-target or Y-100 validation-record incorporation.
No living selection/supervision clause or landed package assertion is removed.

Review2052's separate P3 direct repair-branch runtime coverage remains an
integration/final-audit limit for Y-104/Y-96/Y-61. Accept and repair retain
shared exact identity/ownership validation, and retained-child repair returns
before selected.push. No reproduced bypass is claimed and this focused repair
does not expand into that separate coverage. All original scopes remain.

Required fmt, locked all-targets clippy, locked build and diff checks passed
before commit. Worker host e2e and sabotage are not run. Fresh candidate gates
and review, then a full actual combined merged-host gate and required operator
sabotage must establish both the new refusal and existing positive controls.
No .yard, production code, active runtime store, unrelated box or operational
configuration changes are made. Review2065's runtime evidence limit remains
until those operator checks establish the combined exact head.
