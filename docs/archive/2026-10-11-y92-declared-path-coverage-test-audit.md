# Y-92 declared-path coverage repair

Repair review 1703/check 893's first P3 finding identified a coverage gap.
The source:../operator.env tool argument was not declared, so its
undeclared_file result established allowlist lookup refusal, not launch
path validation. Keep that assertion for the mechanism it actually observes.
The second P3 API-shape finding is not the basis of this repair. Exact pinned
and sealed package identity remains unchanged.

The real Pi packaged-image scenario now runs a declared traversal after the
missing-asset refusal and image correction, before the regular-file control.
Ordinary canonical audit.json declares candidate/../large.txt as its source
path. The fixture already has candidate/ and regular large.txt; normalization
would select a real file, avoiding an accidental missing-file refusal.
No private attempt input is written. Commit and sync freeze the same ordinary
fixture inputs through the existing CLI. The real Pi harness, injecting route,
Yard box, trusted child and native MCP session are unchanged.

The new execution must record no report, make zero controller model requests
and emit invalid_input naming declared file. The following execution in the
same project restores the regular large.txt declaration and must publish the
fixture report. All existing assertions remain, including removed sealed
asset refusal, undeclared tool-key refusal, 32768-byte truncation, trusted
skill/reference visibility, code exclusion, report bounds, native publication,
route key containment, guest permissions and iteration bounds. The regular
control's attempt/probe indices advance from 2 to 3; their assertions do not
change meaning. No deadline, timeout, harness or binary behavior changes.

## test-audit Gate

Retain the new assertion block and the existing positive control under G8's
packaged controller Shown by clause. The spec now distinguishes declared
traversal validation before model requests from undeclared tool-key lookup.
The links and traversal containment contract remains intact; this new case
establishes traversal coverage, without claiming new symlink coverage.

The observer is outside the controller: host route requests, the execution
report exposed through attempt show and the recorded transcript. The expected
zero requests and refusal reason/asset come from the binding spec. The
malformed and regular paths, report text and model replies come from the
fixture. No expected value comes from Yard's implementation or output.

Sabotage, named in the test comment: remove the declared '..' component guard.
Path normalization then admits the existing file and reaches controller model
requests instead of the named pre-call refusal. No missing file or allowlist
lookup can satisfy the new assertion block. The earlier undeclared tool key
cannot detect this guard bypass, and the generic Pi publication scenarios do
not run this packaged controller. This case crosses the normalization boundary
with a valid destination, rather than relying on a nonexistent target.

Keep every existing assertion block. No duplicate mechanism is added: the
new declared-input preflight and existing undeclared-tool lookup act at
different boundaries. Test assertion lines removed: 0. Binary lines removed:
0. There is no runtime hook, fabricated frame, sleep, retry or timeout change.
All 15 guarantees, Y-97 clauses, worker grants, package pins, operator-owned
.yard configuration and historical archives are preserved.

Candidate checks are recorded after validation. No boxed e2e runs here.
The operator's focused host run is still live with only partial audit_limit
evidence; this record claims neither its outcome nor any full-suite pass.
Fresh repaired exact-head focused host tests, full suite, current review and
landing remain required.

Candidate validation: cargo fmt --check, cargo clippy --all-targets --locked
-- -D warnings, cargo build --locked and git diff --check passed. No e2e
test was run. The terminal observer repair has its own new dated record;
it preserves this block and all existing positive and negative controls.
