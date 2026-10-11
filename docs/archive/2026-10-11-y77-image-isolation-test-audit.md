# Y-77 fixture image precondition repair

The operator's exact-head host run 93044 tested original candidate
`9f0a832fb1199f1aa1ff833a9cb907235eedf90a`, on base `6cad53`.
Boxed checks 791–793 passed. The host suite ended 101 with 61 passed and
one failed in 318.41 seconds. The operator's log is
`/tmp/switchyard-y77-9f0-host.log`. It is not mounted in this worker.

The new Codex scenario failed its initial approval assertion. Execution 1
raised red because `box up` reported `image-missing` for
`pinfold/image-yard-8aff4d6654:18dd530e519d7514-1362631`. No failed,
resumed or interrupted accounting phase ran. The fresh completed/zero,
legacy-null, Pi and Claude controls passed. This is a runtime precondition
failure, not evidence that resumed accounting passed or failed. The operator
confirmed that the preceding Y-90 suite was terminal and did not overlap.

## Boundary read and disposition

`Project::new` commits the scaffold Dockerfile and syncs it before work.
`jobs::supervise::image` clones the admitted build-context commit, builds its
Dockerfile through Pinfold and caches the returned build reference by source
head. `Pinfold::image_build` reads the build's own `ref`; `up_worker` sends
that reference unchanged to `box up`. The fixture wrapper records the names
passed to `image build`, before invoking the real Pinfold. `Machine::drop`
kills its daemon, prunes boxes and retires those names with `image rm`.
All fixture machines share Pinfold state and the host container runtime.

The dated Pinfold 0.0.8/0.0.9 records establish intentional sharing of image
IDs across identical contents and retirement by name. A unique fixture root
already makes project image names distinct. It does not make image contents
distinct. Name retirement and another machine's build-to-box-up interval can
therefore overlap on a shared backing image. This is an exposure in the
fixture, not proof that a particular retirement deleted the reported image.
The actual host log and installed Pinfold source/runtime are unavailable in
this worker. The exact destructive call and upstream cause remain unproved;
no upstream defect or fix is claimed and no speculative proposal is filed.

The repair gives only the Y-77 resumed-accounting fixture a Dockerfile label
whose value is its machine root, including the test-process ID and machine
name. A distinct final OCI configuration makes its image content distinct
from the other machines' identical scaffold images. The fixture commits and
syncs that Dockerfile through the supported operator setup before admitting
work. Production image building, reference handling and cleanup are unchanged.
The worker candidate changes no repository `.yard/` file. The fixture still
uses the real daemon, Pinfold, runtime and Codex. No wrapper fabricates an
image, no binary barrier is added, and there is no retry, sleep or skipped
phase. This removes shared-image cleanup exposure for the scenario; fresh
exact-head host validation must establish whether the runtime precondition
and every accounting phase now succeed.

## Assertion gate

G7's shown-by clause records the isolated fixture image. The promise of
execution-local tokens, cache/reasoning subset fidelity, unknown cost and
partial interrupted records is unchanged. Every assertion in the scenario
is retained, including initial approval, failed and interrupted reasons,
exact resumed identities, reopening, numeric fixture values, partial fields,
null cost and the successful resumed completion. The Y-66 fresh completed,
explicit-zero and legacy-null controls remain unchanged.

Sabotage still means adding subsets, copying cumulative predecessor spend,
dropping nonterminal observations or calling interrupted usage complete.
The hard case remains a failed resumed tool turn followed by an interrupted
resumed turn and completion after reopening. Expected numbers still come
from the scripted host Responses fixture; the CLI and read-only store remain
the external observers. Image isolation changes none of those observations.
It adds no assertion twin or new scenario. Test lines removed: 0. Binary
lines removed: 0. No guarantee is weakened.

Validation: `cargo fmt --check` and
`cargo clippy --all-targets --locked -- -D warnings`. The e2e suite is not run
in this worker, as instructed. The operator will validate the repaired exact
head on the real host and obtain a fresh published review before approval.
This record does not claim a host accounting pass.
