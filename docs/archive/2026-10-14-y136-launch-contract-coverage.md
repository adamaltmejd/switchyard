# Y-136 recorded launch contract coverage correction

Review1478 P2 found that recording and preserving package_launch could fail
while exact-package fallback kept the current fixtures passing. Conversely,
metadata could keep fallback faults hidden. The previous dated records remain
history; neither branch was independently qualified by those masked fixtures.

This change affects only the existing G8 test file and current contract docs.
It observes the spec-named effort contract directly at new packaged admission
and definition, staging, image and stopped-owner revisions. Dropping metadata
cannot be hidden by a successful current-package model request.

The same scenario separately exercises both capability paths through real
admissions, binaries and boxes. The metadata case archives the candidate
source, appends a fixed comment to runner.mjs, and builds a real daemon from
those changed runtime assets. The project's committed image installs and seals
those exact bytes. Its admitted package differs from the unmodified receiving
daemon's compiled package, while the recorded effort contract remains valid.
The comment changes no launch behavior; actual boxed asset reads establish the
bytes before and after the daemon change. Native identity verification keeps
the real admission authoritative. No admission fields or store rows are forged.

The fallback case builds the actual supplied source
b9e374f59b2141edc3d2666f07796805c865764b. Source inspection shows no launch
contract recording; its launcher, runner and lock bytes match this candidate.
Its real preserved admission has no package_launch. The receiving daemon's
typed selection identity agrees with that actual admission. Both cases stop
and await checked release before changing daemons. Each receives medium,
explicitly adopts committed max while its real request is held, then stops,
awaits checked release and deliberately starts its native publication control.
Original admission, stopped historical rows and frozen startup fields survive.

## Test-audit Gate

1. Row and clause: G8 now names direct effort contract recording/preservation,
   recorded capability with a different receiving package and exact-package
   capability on an actual pre-recording admission. G2's existing ordinary and
   packaged source/CAS promises and G8's older ownership clauses remain.
2. Sabotage: omit package_launch at admission or drop it during definition,
   staging, image or held-owner adoption. The direct metadata assertions fail.
   Independently remove the metadata arm: the different-package receiving
   execution reaches the host model without requested medium reasoning and
   fails its provider-wire assertion. Independently remove the exact-package
   arm: the real b9e374 receiving execution fails that same outside assertion.
   Test comments name these faults. Existing early-terminal observers still
   fail on the named execution ending before the fixture request.
3. Distinctness and hardest case: current admissions pair metadata with matching
   package bytes and cannot separate the two arms. These cases deliberately
   establish only one applicable capability path at the receiving boundary.
   They extend the same existing G8 scenario after the actual older559
   non-null, unsupported-effort/no-write and omission publication controls.
   No new independent scenario, fake identity, runtime stub, binary barrier,
   timeout control, sleep, retry or concurrency change is introduced.
4. Outside expectations: ARCHITECTURE supplies the effort token. The committed
   fixture config supplies medium and max. Actual Git source and fixed comment
   bytes establish the package difference; the supplied b9e374 source supplies
   absence and identical runtime bytes. Box reads observe installed assets.
   Typed selection versus preserved admission checks the promised identity
   relation, whose expectation comes from those source bytes. Host HTTP bodies
   observe actual reasoning, and the fixture report supplies publication.
   Store equality observes promised historical preservation. No Yard digest
   calculation supplies an expected hash or byte value.

Disposition: retain all original refusal, source, native publication,
early-terminal, ordinary and packaged ownership controls. Add direct metadata
and independent real-binary capability controls within the existing scenarios.
Five test source lines are replaced by the expanded actual-source builder;
zero binary source lines are removed. No binary behavior changes in this repair.

Validation: boxed fmt and clippy only. The new branch controls are pending the
operator's native suite and independent faults/restoration after approval.
Compiling the tests, inspecting source, or earlier passing masked fixtures
does not qualify either branch. No runtime result is claimed here. Full parent
scope and all earlier qualification limits remain required.
