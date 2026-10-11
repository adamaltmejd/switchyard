# Y-92 pinned Harness contract repair

The operator's focused package host execution 93207 at e4a6be ended with
exit 101: zero passed, two failed, 342.30s. The missing sealed asset case
reached real outer Pi traffic and the expected refusal. The corrected image
recorded a null report; the iteration case observed zero controller requests
instead of the fixture's required 12. No package or full-suite pass is claimed.
Execution 93207 is terminal. This worker did not retry the unchanged head.

The operator inspected the SHA512-verified public pi-durable 1.0.4 tarball as
data, without installing, importing or executing it on the host.
package/dist/harness/harness.js exports Harness as an object whose open
method is async and callable; package/dist/index.js reexports that object.
This evidence identifies the exact mismatch: runner.mjs required Harness
itself to be a function, although its actual use is Harness.open(storage,
options, context). The guard rejected the valid pinned package before any
model request with runner_unavailable and export Harness.

The controller now requires a non-null Harness object and a callable open
method. An absent or incompatible contract still refuses before a model
request, naming runner_unavailable and export Harness.open. Every other
factory export check and the Type and BACKGROUND_CONTEXT checks remain.
Runtime, lock, package version and sealed closure checks are unchanged.
The binding specification records this implemented package contract. No
dependency is added or changed; Durable still supplies the conversation and
SQLite checkpoints through its pinned Harness.open factory.

## test-audit disposition

Read the test-audit skill and both complete package scenarios. No test,
assertion, observer, deadline or expected value changes. Retain the existing
G8 missing-asset refusal with its corrected-image publication control and
12-request iteration bound with bare-answer refusal and publication control.
The host route observes actual requests; the store supplies the recorded
report. Expected report content and replies come from the fixture, and the
12-request bound comes from the spec. Restoring the erroneous Harness
function check sabotages both positive controls, as the operator observed.
These scenarios exercise packaged execution in the real Yard box; the other
publication tests do not exercise the pinned controller's internal loop.

Retain the prior audit's declared-read, report-byte-bound, trusted-input and
candidate-code-exclusion, secret-containment, permission and native-MCP
assertion blocks unchanged. They continue to establish G8/G12/G13 with
fixture or spec expectations and external route, store and guest observers.
No assertion is weakened or duplicated. Test lines removed: 0. Binary lines
removed: 0.

Failure diagnostics were considered. The existing assertions remain intact;
this repair adds no transcript or SDK exception dump. The controller already
emits a named refusal frame without SDK error text. A general transcript dump
would exceed the requested sanitized-frame scope. No credentials, environment,
checkpoint data or model output are added to diagnostics.

Exact Node and package pins and hashes, source grants, route, tool and MCP
controls, iteration and output bounds, Y-97 clauses, operator .yard and old
archives are preserved. No host controller, Node or package script executes.
No image build or e2e runs inside this candidate.

cargo fmt --check, cargo clippy --all-targets --locked -- -D warnings,
cargo build --locked and git diff --check passed. The lock SHA256 remains
38d2d7a889d74d6e1863ea2a2be86b5ca8ed0ab9e1cef0b288195e6fe12ab33e.
Fresh repaired exact combined source Gate, both focused package host tests,
the full 71-scenario host suite, current review and actual landing remain
required. Static diagnosis does not establish that this is the only runtime
error or constitute packaged runtime acceptance.
