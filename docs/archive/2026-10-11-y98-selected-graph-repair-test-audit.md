# Y-98 selected graph fixture repair

The operator's focused host run of
`selected_skipped_decisions_bind_identity_without_manufacturing_results`
on eb4ad765 failed at reduced admission. Session 79321 exited 101 with
0 passed and 1 failed in 23.57 seconds. The same-project full default
workflow landed. The reduced graph was structurally valid but refused as
`input=plan`, `runnable=false`, `shape=custom-graph`. No explicit acceptance
or downstream assertion ran. The operator's log is
`/tmp/switchyard-y98-eb4a-skipped-host.log`; it is host evidence reported in
the ticket, not a file read in this candidate. The earlier source audit and
successful compilation did not prove runnable admission.

The fixture declared candidate-gate-keep with only the complete outcome and
no routes. Config::workflow_plan calls Plan::ordinary_routes, which adds a
failed outcome and a failed repair route to implementer, bounded to one
repeat, to each retained candidate gate. Plan::ordinary_selection removes
only complete routes when comparing node definitions. The fixture therefore
matched neither the configured ordinary node nor its selected form. The
G6 supported chain fixture in g06_plan_inspection.rs declares this same
failed outcome and bounded repair edge. The working G1 omission fixture
also retains its selected nodes' ordinary outcome and repair definitions.

The repair adds the missing failed outcome and bounded repair route to the
fixture's retained gate. Its configured runner, write-allowed access,
candidate input, verdict output and required candidate input are unchanged.
Its complete path still ends at that gate, so all seats and landing gates
remain omitted. The implementer retains its configured runner, candidate
output and feedback input. Production resolver, runtime admission, structural
validation and containment controls are unchanged.

Test-audit disposition: keep every G2/G8 assertion block. The full default
positive control, stale revision/head/attention/plan/skipped-selection
refusals, no-write observations, blank-text refusal, current acceptance,
remaining retained checks, selected subset evidence, unreviewed panel,
unchanged executions/results/checks, host git landing and quiescent reopen
are unchanged. The test comment still names sabotage: fabricate passing
omission checks or results, remove exact skipped-node/plan/candidate binding,
accept blank text, mark the panel reviewed or lose durable evidence. Existing
ordinary omission coverage has no explicit decision; failed-result acceptance
has execution/check evidence that omissions must not gain. The held edit
is the difficult revision/head case. Expected nodes, graph, revisions and
text remain fixture literals; base/head come from host git. Attention/proof
identity agreement and durable evidence remain externally observed through
the CLI and store. No observer boundary or expected assertion changed.

Test assertion lines removed: 0. Fixture lines replaced: 1, with 2 replacement
lines and one intent comment. Binary lines changed: 0. Y-97 acceptance
controls and the complete Y-98 implementation are retained. The prior archive
is unchanged. No sleeps, retries, deadline changes, mock frames or binary
hooks are added.

Validation in the candidate: cargo fmt --check, cargo clippy --all-targets
--locked -- -D warnings and git diff --check. No e2e execution is performed
inside the candidate. Fresh exact-head focused and full 70-test host runs,
source/controls review, test gate, current published review and actual landing
remain operator host gates. This repair does not claim those gates passed.
