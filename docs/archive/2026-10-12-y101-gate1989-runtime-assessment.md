# Y-101 gate 1989 runtime assessment

## Actual host result

On 2026-10-12 landing 1985 tested merged head
f88d3231eb2b16d24975bcf20bb599b228cd0f1a against target
d0935530aef8bb3a2498425c39d9349c541a9e8c. Format, clippy and build passed.
The full host gate 1989 FAILED: 73 passed and 2 failed in 1084.62 seconds.
This full result remains failed and does not complete Y-101.

Both complete g08_durable_package scenarios passed in that merged-host run.
This includes the malformed-ticket configuration transition repair and the
changed image-owned sealed asset observations, alongside the retained bounds,
bare-answer, publication, native registration/session, frozen selected-input,
secret, ownership, terminal-frame and isolation controls. This is evidence
for those scenarios on that merged head, not a full-gate pass or completion
of the parent integration or dependent supervision child.

The failed scenarios were committed-Codex containment and unrelated-target
landing. Their failures occurred at real Podman image builds: cached image
commit reported image not known, and cached top-layer lookup reported layer
not known. The operator's summary does not bind each diagnostic to an exact
image/layer ID or native cleanup process. Neither error establishes a Yard
package defect or the precise cause of missing runtime objects.

The operator restored /tmp capacity before the run; later snapshots still
showed several GiB free. These observations do not prove capacity at each
failing operation, and do not establish exhaustion as its cause. The operator
also verified that the simultaneous independent native investigation wrapper
used different graph/run roots from the gate's existing FUSE validation
profile. Shared-store interference between those processes and a proven load
cause must not be asserted.

## Source trace

Read the fixture ownership and cleanup paths and both failed scenarios.
Compared relevant source with target d093553 from the supplied target bundle;
no branch merge or runtime configuration change was made for this assessment.

* Machine::new uses a fixture root containing the test process ID and scenario
  name. Its daemon state and wrapper directory are fixture-local. Pinfold
  state and artifact cache directories are deliberately shared across suite
  fixtures; these directory settings are not proof of native graph/run roots.
* Machine::wrapper records image build names in that fixture's own file.
  Machine::drop kills and reaps its daemon, calls real box prune, then retires
  the recorded names through pinfold image rm and removes the fixture root.
  It does not invoke native image/system prune or delete a native layer store.
  The real prune operation's treatment of other running boxes is a Pinfold
  boundary, not established by its caller alone.
* daemon::project_key hashes the project root. supervise::image builds under
  yard-<project key> with a detached canonical context and serializes builds
  within that daemon. Its cache is keyed by project, branch and head. The
  temporary context is removed after the awaited build returns. This lock
  does not serialize all fixture daemons or native cache operations.
* daemon's project forget retires that project's image name under its image
  build lock. box::image_build passes pinfold's real build failure through;
  it does not synthesize successful image identity or retry that failure.
* The Codex containment scenario and unrelated-target landing scenario use
  distinct fixture roots and the default Dockerfile. Distinct names can still
  identify identical image bytes and shared cached layers. The fixture's own
  cleanup comment acknowledges shared image IDs. Name scope alone does not
  prove how the native runtime retains cache objects during another build.
  Neither scenario directly removes another fixture's image or native store.

Target comparison showed the same relevant image build, retirement and
fixture Drop paths. The target's additional distinct_merge_dates helper and
changes to the separate gate-error scenario do not establish a cleanup cause
for these two failures. The prior dated FUSE/contention records were also
read. They concern different runs and listing failures; their hypotheses and
successful experiments cannot establish this run's image/cache cause.

## Diagnosis and disposition

The supplied evidence establishes native image/cache lookup failures. It
does not establish a concrete fixture ownership/cleanup defect or classify
the runtime as unsupported. Concurrent name retirement of byte-identical
images is a possible boundary to investigate, not a reproduced causal finding.
No native command timeline, removed IDs, cache references or runtime metadata
from the failing operations is supplied in this worker clone. There is no
basis to add unique image content, a cleanup mechanism, serialization or a
package/binary fix for these errors.

A causal host investigation would need the failing native build's image/layer
identities and actual graph/run roots correlated with native removal/cache
operations and their fixture owners. That would distinguish premature
fixture retirement from missing runtime cache metadata without changing the
workload or accepting a retry as evidence. This assessment performs no host
experiment and requests no active-store reset or unrelated-resource pruning.

No source or assertion changes are made. All complete package acceptance,
failed scenarios, controls, containment, concurrency and deadlines remain.
Binary lines removed: 0. Test lines removed: 0. No Shown by clause changes.
Earlier failed results and archives remain unchanged. Installed .yard and
runtime/storage mode are untouched. No retry, sleep, mock or test barrier is
added. Fresh full merged-host validation remains required; Y-100/Y-93 retain
their full integration scope and the supervision child must actually land.

## Worker validation

Boxed cargo fmt --check, cargo clippy --all-targets --locked -- -D warnings
and git diff --check passed.
Host e2e was not run in this worker. This archive is the diagnosis boundary
and a record of operator-supplied results, not new runtime validation.
