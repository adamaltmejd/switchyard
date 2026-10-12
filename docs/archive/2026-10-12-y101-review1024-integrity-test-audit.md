# Y-101 review 1024 admitted package integrity test-audit

Review 1024 P2 identified an unshown G8 integrity guard. Removing the native
bridge's admitted launcher/controller/lock hash loop would still allow the
missing pi-durable module control to refuse through the launcher seal or
module import. That control is retained. It is not evidence for the bridge
cross-check.

## Scoped gate and disposition

Extend the existing missing-assets/corrected-image scenario in the same real
project. After its original publication, replacement, selected-input and
cleanup blocks, build an image that prefixes the real image-owned runner.mjs
with a fixture execution marker. Run the existing image seal generator and
verify the complete seal during the image build. The launcher, Node runtime,
dependencies and the rest of the controller remain unchanged. No candidate
entrypoint supplies the changed asset.

The external pinfold exec wrapper reads the controller bytes and checks the
complete seal inside the real box before Pi starts. After Pi returns it
probes the execution marker. These observations survive ordinary teardown
and cleanup in fixture-owned host files. They do not modify daemon launch
inputs, grants or lifecycle state.

The changed controller must have the fixture's exact prefixed bytes and a
valid launcher seal, but must not execute its marker or make any model
request. Its native terminal frame must name
runner_unavailable:/opt/yard/durable-audit/runner.mjs with failed child status.
No report, eligible result, output or outcome is accepted. The fixture then
parks and abandons that refused ticket through the supported CLI, restores
the unchanged image in the same project and requires the original package
bytes, valid seal, routed model traffic, the external schema-valid report
and eligible completion after checked teardown.

| Guarantee row and Shown by clause | Sabotage | Distinct hardest case | External oracle |
|---|---|---|---|
| G8 packaged asset integrity before calls: extend Shown by with changed image-owned runner.mjs bytes under a valid seal, refusal by asset before package execution or requests, and the unchanged-image publication control in that same project. Keep both complete scenarios. | Remove runAudit's loop comparing admitted launcher/controller/lock hashes. The sealed changed controller can then execute its marker and request the scripted model, so marker absence, zero added requests and the named native refusal fail. | A self-consistent image-owned package differs from the admitted asset while all its own seal/import checks remain valid. A missing module exercises those other checks and cannot distinguish removal of this guard. | Fixture prefix plus repository runner bytes; real boxed byte readback and sha256sum seal result; guest marker existence; host model request counts before/after; spec-named refusal token and fixture asset path; fixture report and CLI/store result after checked teardown. |

The expected bytes and path do not come from admission output. The request
baseline comes from the host model fixture, and the refused execution must
add zero requests. The corrected image must add requests to the same fixture.
The scripted model already rejects every outer Pi model request. Its original
read, invalid-report and valid-publication replies provide the positive
control without a new model script or public endpoint.

No original assertion block is deleted or weakened. The missing-module,
corrected-image publication, iteration/bare-answer, secret, frozen selected
input, native registration/session, ownership and terminal delivery controls
are retained. The stream repair in ce5faa9 remains intact. Binary lines
removed: 0. Test lines removed: 9, replacing the image helper's boolean switch
and its callers with explicit image-build modifications. Those replacements
remove no assertion. The new seal and byte probes use real pinfold box exec,
and completion uses the existing execution-bound status history.

This is a scoped test-audit gate, not a whole-suite sweep. No mock, binary
flag or barrier, sleep, retry, timeout or concurrency adjustment is added.
No runtime, grant, ownership, publication or lifecycle mechanism changes.
The P3 error-conversion and table findings prompted no speculative work.
Earlier archives and installed .yard remain untouched. Y-100/Y-93 retain
their complete acceptance, including the dependent supervision child.

## Validation

Boxed cargo fmt --check, cargo clippy --all-targets --locked -- -D warnings
and git diff --check passed. The worker did not run host e2e or a runtime
sabotage experiment. The supplied failed host result remains failed; source
review and compilation do not convert it into a pass. Operator host landing
must establish the new block and the retained full scenarios after approval.
