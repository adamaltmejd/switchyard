# Y-92 integration onto Y-97

The frozen target is 96bb11d2a2ea55e2b31208e5c04674d3763a87be,
including landed Y-97 at 0c5ae61. The preserved packaging candidate is
bebbbe4e6522a6289c424ddb24bde9db9a0a3d78, with prerequisite 870ba9d.
The candidate object was already available in this clone. Its object identity
and prerequisite ancestry were checked. The supplied transport was not
decoded and its SHA256 was not verified in this integration. No regenerated
bundle hash is used as evidence.

The only merge conflict was the G8 guarantee row. Resolution retains the
target row verbatim and appends the candidate's packaged-controller clauses.
This preserves Y-97's migration, automatic repair, approve-at-limit, selected
acceptance, held-publication and infrastructure-refusal controls. A mechanical
prefix comparison against the target row passed. G12 and G13 package clauses
and the binding package contract are retained.

The runner tree, production Pi child bridge, new scenarios, test registration
and all three historical Y-92 archives compare byte-for-byte with the preserved
candidate. No repository .yard file changed. No daemon, reviewer or model was
reconfigured. Installed image/config handling remains operator-owned.

Node remains exactly 26.11.1. pi-durable, pi-ai and chord remain scoped
@earendil-works packages at 1.0.4; pi-telemetry remains overridden to 1.0.4.
The exact lock SHA256 is
38d2d7a889d74d6e1863ea2a2be86b5ca8ed0ab9e1cef0b288195e6fe12ab33e.
Dependency reasons remain in the binding spec: Durable supplies conversation
and checkpoints, Pi AI supplies the model client and schemas, and Chord
supplies context. Telemetry is a pinned transitive dependency. The controller
is packaged at the fixed trusted image path; daemon selection awaits Y-93.

## Focused test-audit Gate

Applied test-audit's Gate and sweep tags to the imported assertion blocks.
Read the new scenarios and shared fixture helpers; compared existing audit
publication, Pi extension/credential and gate/seat isolation mechanisms.
This is a Gate for this integration, not a whole-suite release sweep.
The prior real-pi-package-gate archive remains immutable and contains the
original repair assessment. Its earlier packaging-test-audit predecessor
describes a rejected fixture and provides no acceptance evidence.

| Disposition | Row and failure mechanism | External expectation and control |
| --- | --- | --- |
| Keep | G8 preflight; omit sealed-asset checks | Removed entrypoint names runner_unavailable with zero child requests while outer Pi runs. Corrected image publishes in the same project. |
| Keep | G8 reads; remove declaration or byte bound | Fixture supplies oversized source and traversal. Spec supplies 32768 bytes and undeclared_file; declared reads succeed. |
| Keep | G8 report validation; count characters as bytes | Fixture's 4001 two-byte characters exceed the spec's 8000-byte limit. invalid_report precedes valid publication in the same conversation. |
| Keep | G8 instructions/code exclusion; load candidate or offer shell | Fixture skill and reference reach the model, both hostile payloads are read, and guest sentinel stays absent after an executable request. |
| Keep | G12 child persistence; inherit or store upstream key | Fixture Authorization carries its key. Guest environment/files, checkpoint/scratch/package assets, model body, transcript and stored report lack it. |
| Keep | G13 permissions; permit input writes or lose writable state | Real guest input writes fail; distinct scratch/checkpoint writes succeed. |
| Keep | G8 iterations; remove request cap | Endless fixture reads stop at the spec's 12 requests with audit_limit and no report. Fresh publication in the same project controls refusal. |
| Keep | G8 publication; synthesize a report from text | Bare fixture answer names missing_report with no report; fresh explicit native publication succeeds in the same project. |

Existing audit tests retain their daemon validation and result-eligibility
assertions. Existing Pi exclusion tests retain discovery and session controls.
The new cases exercise the installed Durable closure, registry, child bridge
and checkpoint persistence, so those mechanisms are not twins. Report content
comes from the fixture; teardown is observed through attempt show. No assertion
was changed or removed. No crash, sleep, retry or timeout-as-success was added.
No operator decision to weaken a clause was requested or needed.
Test lines removed: 0. Binary lines removed in integration: 0.

## Validation boundary

cargo fmt --check, cargo clippy --all-targets --locked -- -D warnings and
cargo build --locked passed on the integrated tree. Shell syntax and
git diff --check passed.
The exact operator host packaged-image tests, whole host suite, fresh review
and actual landing gates remain required. No e2e ran in the candidate.
No host Node, library, controller or package script ran. No runtime package
installation or download occurred. Source checks are not packaged acceptance.
