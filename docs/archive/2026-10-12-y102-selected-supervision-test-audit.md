# Y-102 selected Durable supervision test audit

Y-101 actually landed as c826310. Its merged host validation is recorded at
7a4c95a. The Y-100 sizing assessment was read from the public Y-102 body;
`2026-10-12-y100-sizing-test-audit.md` was absent from this base. The redesign
plan at `2026-10-09-switchyard-redesign-plan.md` was also read. The sizing
assessment is boundary evidence, not a runtime pass.

This is a scoped test-audit gate, not a whole-suite sweep. Extend the existing
`a_packaged_audit_bounds_iterations_and_requires_native_publication` scenario
in `g08_durable_package.rs`. Retain its request-bound and missing-publication
refusals. Its successful publication control now crosses selected cancellation
and refused ownership before checked release. Keep the complete first package
scenario, including Y-99 isolation, native publication, bounds, selected-input
authority and secret containment. Keep generic audit, cancellation, capacity
and review ownership coverage. Their mechanisms remain distinct.

| Exact guarantee clause | Sabotage named in the test | Distinct hardest case | External oracle |
|---|---|---|---|
| G8: stop at the selected controller's own held host-route reply retains reservation through externally held teardown; process/box absence and host lock freedom precede the subsequent selected execution. G13: controller cancellation retains its execution slot until those owners are gone. | Release before controller/box disappearance, admit a successor while reserved, or cancel only the wait. | A real selected controller is awaiting the fixture's host response. Supported stop runs while a wrapper holds actual box-up stdin EOF, keeping the real box alive. After the hold is released, explicit start runs the same admitted node in the same attempt. Outer-Pi holds and preparation cancellation do not expose this boundary. | The scripted request offers `read_declared`, with no outer-model request. A real boxed `/proc` probe finds the controller and proves it alive. Real pinfold listings show the retained box, then its absence; pinfold exec returns absent for the recorded guest PID. The old inherited host lock becomes available. Store reservation and history order bind stop/release and subsequent start to their executions. The frozen base is the host Git head recorded before admission. |
| G8: native publication followed by real host lock refusal retains reservation and an ineligible report with no cleanup successor despite box absence; checked release admits that same bound report without replay. | Count publication or native terminal success before checked ownership, or start cleanup while the predecessor is reserved. | The allowed subsequent selected controller publishes the fixture report and completes. A wrapper holds after the real exec exits. A second wrapper holds a real listing after work drops its inherited lock; the fixture acquires that same inode before the host ownership probe. Real box absence meets an actually held host lock. The next listing holds the reaper after ownership refusal. Unlocking permits checked release and cleanup in the same project. Generic seat publication does not cross Durable's native terminal frame. | The fixture supplies the schema-valid report. Host `try_lock`, device/inode readings and real pinfold listings establish the refusal independently. CLI/store show `red:ownership` naming the current execution, retained reservation, unchecked teardown, completed infrastructure, ineligible output and no successor. History places cleanup after checked release. Full report, node/input binding, admission and result comparisons establish identity preservation across refusal/release; two implementation rows prove no audit replay. |

The selected shape has no graph continuation. Cleanup is its actual automatic
successor. A supported restart after cancellation is the acting continuation
control. No graph, checkpoint resume, ownership object or recovery path was
added. The real listing is delegated and its output is unchanged; no synthetic
refusal, successful report, binary barrier or timeout expiration is used.
Fixture holds use host FIFOs and model latches. Event waits use status history.
No sleeps, retries, public endpoints or concurrency-limit changes were added.

Publication payload/session validation and general report identity agreement
remain in `g08_audit_reports.rs`; these blocks do not duplicate those checks.
The changed package control establishes preservation across this selected
boundary. The bound count and bare-answer blocks remain unchanged.
Binary lines removed: 0. Test lines removed: 4 (the extended success control).
Existing test assertion blocks removed: 0.
The former immediate success control is extended, not deleted.
ARCHITECTURE.md adds the current selected lifetime contract and G8/G13 shown-by
clauses in the same commit. No existing promise is weakened.

No production violation was reproduced, and no production source was changed.
Boxed `cargo fmt --check` and
`cargo clippy --all-targets --locked -- -D warnings` pass. Host e2e and sabotage
execution were not run in this box and remain operator-owned after approval.
This candidate is not runtime validation or completion of Y-100/Y-93.
Both integration acceptances remain intact and require actual child landings.
