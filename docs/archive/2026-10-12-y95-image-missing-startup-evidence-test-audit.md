# Y-95 corrected image-missing startup evidence

## Corrected actual observation

The operator read complete saved host1852 terminal detail. Before the first
model hold failed, old execution1 emitted `image-missing` for
`pinfold/image-yard-77612586ff:18ddb77b9464d83b-1445894`: the image was not
present locally, and native inspection said `image not known`.
This establishes startup refusal before the absent hold. It does not establish
which operation removed the reference or whether it ever remained present
in the runtime used for box startup. The hold failure is not an unexplained
model delay, and no FUSE cause is proved.

Actual landing1848/host1852 on
`2860c3051031d7bbc78bd7cfb452de30fde3371d` still failed71/1 in499.01s.
The G6 repair passed. No corrected observation turns this into a host pass
or landing. The earlier first-hold archive remains immutable; its statement
about the retained tail reflects information then available. This new record
supplies the correction. No new host execution or native inspection ran here.

## Build and retention trace

The real old binary comes from
`c85369dfaec8f8554da49ddf02a4dbec03bcad02`. Its implementation calls
`supervise::image` before passing the returned string through Worker and
BoxSpec to `up_worker`. Image construction is serialized by the old daemon's
image-build mutex. It clones the canonical build head, invokes real Pinfold
`image build yard-<project-key>`, and caches the returned ref by head.
`box::image_build` accepts the `built` event only with exit code zero and
returns its ref; it does not reconstruct a tag or choose another image ID.
Startup failure revokes the grant, invalidates that cached ref and records
failure. There is no automatic successful-start claim or retry in this test.

Current source retains this build/ref protocol, adding target branch to the
cache identity. Project keys in both old and current binaries are the first
ten SHA256 hex characters of the project path. Distinct machine roots and
project paths give distinct names in the fixture design; they do not prove
that differently named builds have distinct backing image content.

The fixture wrapper records image-build names before executing real Pinfold.
Machine Drop closes its daemon, uses existing box prune, and retires exactly
those recorded names using `image rm NAME`. All machines share the fixture
Pinfold state and host runtime. This scenario calls no project forget or
image retirement before its first wait. Its own Drop follows the wait panic,
so its own normal cleanup cannot explain the earlier startup refusal.
Other machines can concurrently retire their own names. Their actual calls,
name associations, shared runtime IDs and timing for host1852 are not supplied.
No name collision, cross-name deletion or fixture race is asserted as fact.

The immutable September Pinfold 0.0.8/0.0.9 records describe shared content
IDs across names and retirement by name. The Y-77 image-isolation audit and
operator record document another image-missing startup refusal and a fixture
exposure to concurrent shared-content retirement. The Y-94 gate scenario also
uses a machine-specific image. These are relevant precedents, not evidence
that those earlier mechanisms caused execution1's missing reference.
An isolation change here would not identify its disappearing-ref cause and
is not presented as a demonstrated repair.

## Compatibility and runtime limits

Old and current Yard source both consume Pinfold's own built ref unchanged.
The old binary lacks current reservation/teardown fields, but that does not
explain absence of a built runtime reference. A built event can establish
Pinfold's reported success; the supplied terminal detail does not include
the corresponding built event, image ID, retention table or destructive call.
No old-binary compatibility defect is established by the source comparison.

Pinfold is not installed in this box and no host runtime inspection is
available. The public v0.2.2 Podman source fetch returned HTTP403, so its
current deletion/retention implementation was not read here. The repository's
runtime-contention report concerns listing locks and explicitly leaves fault
ownership unresolved. Those observations and the isolated-FUSE successes
cannot diagnose this different image-reference refusal. No upstream defect,
FUSE defect or fabricated host observation is claimed.

The next operator-owned diagnostic must correlate the actual built event/ref
and runtime image ID with startup inspection, recorded fixture image names,
concurrent retirements and Pinfold/runtime retention outcomes in the same
profile. It must distinguish a missing build registration/ref, a fixture
retirement race and runtime disappearance. A later successful inspection
alone proves none of these. No retry until green, deadline change, concurrency
cap, manual pruning or process kill is introduced or authorized by this record.

## Narrow diagnostic extension and test-audit

The existing scoped first-hold capture gains two fixed boolean detail flags:
`image-missing` and `image not known`. It still emits no raw detail, ref,
payload, headers, prompt, credential or worker output. These flags preserve
the corrected startup category in a future failed fixture before cleanup;
they are substring observations, not assertions or causal classifications.
Original panic, row bounds, wait duration, cleanup and every assertion remain.

Disposition: keep all G8/G6 unbound/unknown-history, edit and same-project
fresh-publication controls. Existing spec clauses, fixture expectations and
sabotage comments remain unchanged. No assertion block, twin scenario,
guarantee weakening or production repair is added. Test lines removed: 1,
replaced by the same diagnostic query with two additional flags. Binary lines
removed: 0. The original Y-95 proof guard and all selected-check, result,
queue landing/reopen and protected ownership controls remain.

`cargo fmt --check` passed. `cargo clippy --all-targets --locked -- -D warnings`
passed in 1.54 seconds, and `git diff --check` passed. Host reproduction, diagnostic
capture and sabotage remain unrun here. This is corrected evidence and bounded
source diagnosis, not a fix clearing an unchanged host retry. Fresh exact-head
operator host validation, published review and actual landing remain required.
The full parent, Y-96, Y-61 and redesign/release through Y-80 remain; Y-81
stays deferred. No .yard, model, reviewer, native package/pins or operational
daemon change occurs. Historical archives are not edited.
