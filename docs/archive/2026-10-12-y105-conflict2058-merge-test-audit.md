# Y-105 conflict2058 semantic union and test-audit

Approved candidate 474a8fe4d9616984c38f598334a59fc7f8ba1f72 did not land:
landing2058 conflicted only in docs/ARCHITECTURE.md against actually landed
Y-102 b4b0d12098d9fb8046795f728b6db93b15a2f5e2. The operator reports
Y-102 host2057 passed 75 scenarios, zero failed, in 708.77 seconds; its
intent resolved and cleanup2059 ended removed. That pass applies to Y-102's
actual merged input, not this combined candidate.

The operator consumed that landing and imported its validation record at
canonical 1c72d012d18d2040b2bd121a35d07553f622a787. The repair box's
/yard/input/target.bundle advertises only main at b4b0d12. The reported
canonical validation-record object is absent from both the bundle and clone.
The worker requested a refreshed input and merged the available landed Y-102
head. This does not claim incorporation of the absent validation-record
commit or that the available bundle equals current canonical.

## Resolution and scoped test-audit

Preserve the entire Y-105 implementation and terminal-release observer repair
from 698ffcb/474a8fe. Preserve landed Y-101 selection and Y-102 supervision,
source and tests. The only manual merge edit is the G8 evidence row:
retain its selected eligible merged-failure decision heading and both failed
merged acceptance/error-retry evidence sentences, together with every Y-102
selected controller cancellation and checked publication clause. Y-102's
selected lifetime contract and G13 controller/box/lock evidence merge intact.
No guarantee or ownership/inventory guard is removed or softened.

Apply test-audit as a scoped merge gate. Imported g08_durable_package.rs is
byte-identical to the supplied landed target, including all cancellation,
process/box absence, host-lock refusal, report identity, checked eligibility,
cleanup and no-replay assertions. The Y-102 selected-supervision and three
repair audit records accompany it unchanged. Y-105's G3/G8 test files are
byte-identical to the approved candidate, retaining terminal-release waits,
current unreserved failed-row prerequisites, exact stale/no-write results,
failed snapshots, held-child restart, target change and pinned continuation.
No assertion or scenario is added, deleted or changed to resolve this merge.

The existing audit records name external oracles and sabotage for each
scenario. Selected-controller cancellation/publication and failed merged-gate
operator decisions have distinct mechanisms and observations. The union does
not add a twin. Resolution-only test lines removed: 0; binary lines removed:
0. This is a scoped merge review, not a whole-suite sweep.

## Preserved failures and limits

Host2042 tested 525c2c4 and failed 72/3 in 555.46 seconds. All three failed
acceptance positive controls returned stale ownership for the same original
ended/fail landing with reserved:true, expected released:true; no acceptance
succeeded. The operator retains the complete failure tail at
/tmp/switchyard-y105-host2042-failed.log and the authoritative snapshot at
/tmp/switchyard-y105-host2042-terminal.json. They were unavailable in the
worker filesystem. The source-established historical phase-release observer
defect and repair remain in the two host2042 audit records; neither this merge
nor Y-102's pass converts that failed result into a pass. Conflict2058 is a
separate unsuccessful landing, not host validation of terminal-release repair.

The operator's review2052 P3 disposition remains: accept and repair share
exact identity/ownership validation, and retained-child repair returns before
selected.push. Missing direct runtime repair coverage remains a limitation
for Y-104/Y-96/Y-61 integration and final audit. It is not evidence of a
reproduced bypass, and this merge does not claim to close that limitation.
The full original Y-105 and parent scopes remain required.

Required fmt, locked all-targets clippy, locked build and diff checks passed
on the combined candidate. Worker host e2e and sabotage are not run. Fresh
candidate gates and review, and a full actual combined merged-head host gate
must establish the acceptance positive controls and ownership refusals.
No .yard, operational configuration, runtime store or unrelated box changes.
