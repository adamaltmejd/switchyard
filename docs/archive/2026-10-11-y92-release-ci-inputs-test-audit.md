# Y-92 checked release and CI inputs repair

The correctness review blocked ec8a79c with two findings. P1 identified a
race between execution.ended and checked capacity release. P2 identified
missing public build inputs in both GitHub host-suite jobs. This repair
addresses both without changing package behavior or claiming acceptance.

## P1: observe checked release

executions::end emits execution.ended without setting teardown. After the
work future returns, capacity::release probes ownership and, if proved gone,
sets teardown=1 and releases the reservation in one transaction. That
transaction emits execution.released with attempt and execution identity;
it does not carry a ticket or kind. The package helper now waits for that
event with the exact started execution and attempt before attempt show.
The existing start/end observations still bind ticket and implementation
kind. Every negative case and positive control uses this same helper.

This is the existing G8 publication/checked-teardown boundary, not a new
guarantee. The test-audit Gate retains every assertion. The external observer
is CLI history and supported attempt show. Expected teardown=true comes from
G8, and native report text comes from the fixture. Execution identities are
compared across outputs, not used as invented expected values. Sabotage:
omitting checked teardown fails the existing teardown assertion. Reading at
execution.ended instead would race rather than establish the guarantee.
The existing held-publication test uses this same release observer but does
not exercise the sealed packaged controller, internal bounds or read surface.
No duplicate scenario or extra assertion is added. No sleep, clock evidence,
retry, deadline change or lifecycle change replaces the missing ordering.

## P2: provision public CI build inputs

The fixture still requires YARD_E2E_DURABLE_INPUTS and still builds and runs
the real package. The GitHub e2e job now defines that path under runner.temp
and invokes fetch-build-inputs.sh before the full suite on both configured
Linux architectures. The script selects the official Node 26.11.1 archive
for the runner architecture and verifies its committed SHA256. It parses
all 115 non-root lock entries and downloads each exact registry resolved URL,
checking its locked SHA512 before returning. Numeric archive filenames avoid
collisions between scoped package basenames; the offline installer discovers
these supplied archives as before. Downloads remain data outside the source
tree. No host Node, npm, installation, package import or archive execution
occurs. Package execution remains in the real isolated image builder/box.

The script needs existing host curl, jq, sha256sum and OpenSSL. Their reasons
are fetching HTTPS archives, reading the committed lock, verifying Node's
SHA256 and verifying npm's SHA512. The binding dependency section and package
instructions record this build-tool contract. No package or runtime dependency
is added; no lock, integrity, seal or pre-model identity check changes. CI
installs none of these tools in this repair. An unavailable or corrupt input
fails provisioning; no test is skipped or converted into a passing scenario.

## Assertion dispositions and validation

Read the complete package tests through test-audit. Retain G8 missing sealed
asset refusal and zero controller requests with outer Pi traffic; declared
traversal refusal; same-project corrected report; read/tool/report bounds;
trusted skill/reference visibility; 12-request limit and bare-answer refusal
with native publication control. Retain G12/G13 route key, guest/source/state/
log containment, candidate sentinels and read-only/writable path probes.
Expected values remain the fixture literals or binding spec, observed through
real route traffic, CLI/store, guest commands and host files. The existing
comments name sabotage and explain why other publication tests cannot cover
this package boundary. No assertion is added, removed or weakened.

Test lines removed: 0. Binary lines removed: 0. Existing release behavior,
all Y-97 clauses, worker grants, pins and hashes are preserved. Repository
.yard and old archive records are unchanged. Y-93 still owns daemon selection.
No e2e, image build or host controller execution runs here. Fresh exact-head
focused package tests, full host suite, current review and actual landing
remain required.

cargo fmt --check, cargo clippy --all-targets --locked -- -D warnings,
cargo build --locked, fetch script shell syntax and git diff --check passed.
The script's jq expression accepted all 115 locked archives; both architecture
entries remain in the committed Node checksum file. Lock SHA256 remains
38d2d7a889d74d6e1863ea2a2be86b5ca8ed0ab9e1cef0b288195e6fe12ab33e.
Network provisioning and GitHub execution were not run in this candidate;
static checks do not establish their success or packaged runtime acceptance.
