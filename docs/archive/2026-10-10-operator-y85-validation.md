# Y-85 operator validation — 2026-10-10

Ticket Y-85, attempt 64, revision 4. Base:
5d1230f8631f70fc9555a5c86aaf4ac12ba658a7.
Candidate: ec434d7f78ff129650acebf60c10f9727fbd70c8.
Proof: e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855.

The operator read the complete initial 16-file delta and both repairs.
The source and test-audit Gate pass on the full candidate. The admission
source is immutable Git input, checked against recorded input identities
and resolved plan. Execution consumes persisted selected definitions.
Trusted staging reads the admitted source even after candidate base repair;
image construction uses that source and preserves separately observed
runtime image IDs. Template sync affects future admissions. Check and
approval bindings include nullable admitted plan revision; historical
unknown admissions preserve current-template behavior and supported exits.

Selected-target approval and protected paths stay live. Automatic approval
reads them under target serialization; queue revalidation reads current
policy before gates and intent. Policy withdrawal preserves the same
candidate and valid judgments. Capacity uses the current default, and
box launch narrows admitted egress with current containment before issuing
worker grants. Credentials and grant revocation retain precedence.

The revised G2 and G8 expectations and binding architecture replace implicit
active template adoption together. Head, ticket revision and proof changes
still invalidate judgments. G10 observes actual frozen gate counters, image
bytes, reviewer instructions and trusted skill inputs through conflict repair
and quiescent restart, followed by a future admission selecting v2 inputs.
Expected definitions are literal fixture inputs; Git, pinfold and fixture
model requests supply external identities and observations. G11 retains the
worker configuration refusal and observes allowed operator sync on future
work. Manual-policy withdrawal has its same-candidate positive landing
control, distinct from protected-path withdrawal.

Initial candidate 2991446 passed boxed checks, but the operator test-audit
found new future-admission controls abandoning open unparked tickets before
subsequent counter/history observations. That permits scheduler readmission.
Supported revision-bound edit sent a park-before-abandon fixture repair.
No host run is claimed for 2991446. Candidate 0dc20aa added those two parks
and a new dated audit, retaining all assertions and changing no binary.

The full isolated host run at 0dc20aa ended exit 101: 53 passed, one failed,
773.81 seconds. Existing independent_target_landings_share_the_project_execution_cap
failed waiting for target landings within 600 seconds. Live read-only fixture
rows showed the existing scenario's abandoned Y-4 readmitted as attempt 5;
its implementation execution 8 held the sole slot with no box handle while
Y-3 feature/c landing execution 6 waited without a reservation or intent.
The evidence and log were preserved before fixture cleanup. No retries,
operator process kills or latch changes occurred.

Exact source established a runtime lock-order regression: new worker launch
reserves capacity before taking its target lock, while final landing took
the target before waiting for capacity. The held rows matched that cycle.
The operator had missed that final landing ordering in the first source Gate.
The host suite caught it. The repair request required a runtime correction
and preserved same-target contention coverage, rather than a fixture-only
change hiding the regression.

Candidate ec434d7 reserves landing capacity before acquiring the target lock.
The guard remains held through intent, publication and ownership proof.
It introduces no lifecycle or limit bypass. The existing fixture now parks
its cancelled Y-4; every assertion stays. New deterministic G4/G10 coverage
holds a landing host gate and a same-target pre-box Git clone at live cap two,
lowers the current default cap to one, releases the gate and observes final
landing waiting without intent. Releasing the clone reaches a real boxed
worker held at the host model with the sole slot and unchanged target.
Releasing that worker permits approval and both actual candidate landings.
Host Git proves their ancestry. The test names the target-before-capacity
sabotage. No sabotage run is claimed; no sleep, retry or binary barrier exists.

At exact clean ec434d7 in isolated checkout
/home/yard/validation/switchyard-y84-20261010, full cargo test -p e2e --locked
passed 55 scenarios, zero failed, in 214.86 seconds; session 55253 ended exit 0.
Both the previously failing capacity scenario and new deterministic control
passed. The checkout stayed clean at that head. Previous validation was
terminal before reuse. Log: /tmp/switchyard-y85-ec4-host.log.
Fresh boxed fmt check 714, clippy 715 and build 716 passed at revision 4.

Review execution 1346 published check 717, pass, round 1, with one P2
correctness finding. Concurrent explicit attempt start can admit after the
scheduler's live-attempt read; spare_lanes then finds no admitted source in
its cache and aborts that scheduler tick before admission, queue and cleanup.
The next tick loads the source. This is a transient fail-closed availability
finding, not an incorrect identity, capacity bypass or irrecoverable data
path. Per yard-drive's below-blocking default, the operator landed over it
and carries it to the final whole-tree audit. No permanent CLI failure was
reproduced, so no speculative defect ticket or runtime workaround was filed.

The reviewer also attempted optional clippy; dependencies failed to download
with proxy CONNECT 403 after 131.6 seconds. Its pipeline's exit zero came
from tail and is not a passing compile result. Required candidate clippy
check 715 passed separately; no network grant or reviewer change occurred.

The approval diff matched the fully read and host-tested combined candidate
except CLI trailing newlines. Protected architecture hunks were read.
Approval 76 bound exact ec434d7 and the empty proof above.
Landing execution 1347 ran fresh fmt 1348, clippy 1349, build 1350 and
host e2e 1351. All passed. The full landing host suite passed 55 scenarios,
zero failed, in 243.89 seconds. Landing was recorded at audit seq 4656;
ticket completion and attempt termination followed at 4657 and 4658.
Cleanup 1352 ended removed at 4660. Supported yard sync consumed the
exact ec434d7 landing into the operator checkout at 4661.

Y-86's preset and one-off admission and Y-59's full integration remain
required before Y-60 and Y-66. Deferred Y-81 stays parked. The complete
useful redesign release, final cleanup and platform validation remain
unfinished. No push, publication, installation, frozen operational daemon
replacement or model/reviewer switch occurred.
