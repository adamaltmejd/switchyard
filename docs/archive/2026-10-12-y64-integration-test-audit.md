# Y-64 complete Durable audit integration

This assessment covers the complete original Y-64 contract and the October 11
operator partition. It follows actual implementation landings, not manual
ticket completion or the abandoned attempt77 assessment. The reviewed base is
`389bb67ad779f51a4a71051fb94e8bee0c71a5fa`.

No remaining integration defect was found. This candidate records the
assessment. It changes no production code, assertion, configuration or binding
contract. The implemented G8/G12/G13 clauses already landed with their owning
slices. No future capability is added to ARCHITECTURE or README.

## Actual landed inputs and evidence

Every head below is an ancestor of the reviewed base, checked with Git.
These are actual landed heads, which sometimes differ from reviewed candidates.

| Ticket and scope | Actual landed head | Actual host landing evidence and record |
|---|---|---|
| Y-60 ordinary execution integration | e9e68ca546a46483b7ed969827d80860c16f4e34 | host1552: 62/0; [operator record](2026-10-11-operator-y60-validation.md) |
| Y-66 optional usage/provenance | b22ab00d405ce5d6d904803e6750c9d8f67f0026 | host1399: 58/0; [operator record](2026-10-11-operator-y66-validation.md) |
| Y-91 authoritative report publication | e65730bc84ab99974e65966b32d3c181751524a3 | host1647: 67/0; [operator record](2026-10-11-operator-y91-validation.md) |
| Y-92 trusted boxed package | 9c106745a01ddc443c937d1a543b8e45fb017ef2 | host1808: 72/0; [operator record](2026-10-12-y92-operator-validation.md) |
| Y-99 frozen inputs and exclusive state | 7a2bbfe1f7b011fca85481f0cbdb83997834b5b0 | host1857: 72/0; [operator record](2026-10-12-y99-operator-validation.md) |
| Y-101 selected native invocation | c826310620e4c2b3803110202d6a4ad3707f2aca | host2000: 75/0; [operator record](2026-10-12-y101-operator-validation.md) |
| Y-102 selected cancellation/ownership | b4b0d12098d9fb8046795f728b6db93b15a2f5e2 | host2057: 75/0; [operator record](2026-10-12-y102-operator-validation.md) |
| Y-100 selected runtime integration | 7272212aa161b7ab814b28529914ac327d0cecc9 | host2074: 75/0; [operator record](2026-10-12-y100-operator-validation.md) |
| Y-93 complete child integration | 869797aaba92b5da65d4e27bd4bd6a9f58160c19 | host2089: 75/0, no ignored scenarios, 924.47s; [operator record](2026-10-12-y93-operator-validation.md) |

Y-100 integrates the actual Y-101 selection and Y-102 continuation landings.
The direct parent chain Y-60/Y-66 -> Y-91 -> Y-92 -> Y-93 is satisfied by
actual landings. The additional isolation and selection slices are present.
`git diff 869797a HEAD -- crates runners README.md docs/ARCHITECTURE.md` is
empty. The latest combined runtime evidence therefore covers the same
production, package and assertion bytes assessed here.

The host2089 record reports fresh merged fmt2083, clippy2084 and build2087,
published review2081/check1091 with no findings, resolved landing2082 and
cleanup2091 removed. Both complete packaged scenarios passed, along with
the shared publication, containment, route and capacity controls. The host
used the isolated Podman/FUSE Linux x86_64 profile and public build inputs.
This worker read the committed operator records. Their raw host logs and
private review artifacts are not mounted here and were not independently
reopened. These are inherited host observations, not a new Y-64 host run.

## Original clause mapping

Read the actual selection/admission, staging, native bridge, controller,
publication, settlement, capacity, proof and cleanup mechanisms. Compare them
with the binding selection, Publication, Boxes, Worker tools, dependency and
G8/G12/G13 clauses, README and the October 9 redesign plan.

In the table, A names
`a_packaged_audit_refuses_missing_assets_then_publishes_in_a_corrected_box`;
B names `a_packaged_audit_bounds_iterations_and_requires_native_publication`.
Both are in `g08_durable_package.rs` and passed in inherited host2089.
R names the four shared scenarios in `g08_audit_reports.rs`, also inherited
passes. Source conclusions are identified separately from these observations.

| Original behavior or safeguard | Actual implementation | External scenario and observation |
|---|---|---|
| Optional bounded iterative read-only skill audit for trusted operator selection | Config's optional workflow profile never inherits. Plan accepts only one read-only implementer, report output and complete outcome. Profiles/presets/operator overrides resolve through the same admission. | A refuses packaged topology as custom-graph, then validates a profile and admits it through the supported CLI. Fixture-selected Markdown enters controller instructions and the explicit reference is read. Preset syntax uses the same resolver; every selection syntax is not separately exercised. |
| Controller, model client and tools inside one Yard-owned box; one execution slot | PackagedAudit wraps the existing Pi harness. Its native hook spawns the image launcher in that box. The controller creates no box, execution or workflow owner. Capacity reserves the existing implementation execution. | A/B require every host model request to offer the controller's read_declared tool; outer Pi requests fail the fixture. B sees one real runner PID in the execution's box, retained reservation and no successor during teardown. |
| No host Node/npm, model client or tool extension | Host Rust stages data and starts pinfold. Public archive fetching uses curl/jq/checksum tools. install-image and seal run only in the image builder. | A builds an isolated image from supplied public archives and runs real boxed Pi/controller. The absence of any host Node invocation is a source conclusion; no separate run on a deliberately Node-free host is claimed. |
| Operator image/config; pinned Node 26+ and locked pi-durable/pi-ai/chord 1.0.4 at a fixed path | Reviewed Dockerfile/install-image/lockfile install Node 26.11.1 at /opt/yard/durable-audit and ignore package scripts. pi-telemetry 1.0.4 is pinned transitively. README supplies ordinary commit+sync instructions. | A changes only its operator fixture image/config through commit+sync, builds and seals the package outside /workspace. This candidate edits no installed .yard. |
| No runtime install, discovery, fallback download or candidate entrypoint | Native hook checks admitted launcher/controller/lock hashes. Fixed launcher verifies installed closure; controller verifies Node/package identities and explicit exports. There is no runtime npm or package fallback. | A removes a sealed Durable module: runner_unavailable names pi-durable/dist/index.js with zero model requests. Corrected image succeeds in the same project. A changed runner with a valid new image seal is separately refused by admitted hash, naming runner.mjs, before execution/model calls; unchanged image controls it. |
| OpenRouter/Opencode injecting route; guest placeholder, host operator.env key; preserve G12/G14 | PackagedAudit delegates Pi route/env. Child receives only proxy variables, then uses fixed provider URLs and yard-placeholder. The daemon supplies upstream route secrets separately. | A host receipt has the fixture key; request bodies, guest environment/source/proof/trusted inputs/state/package assets, streams and store rows lack it. Existing G12/G14 route/Git scenarios survive the combined suite. The packaged fixture uses OpenRouter; Opencode uses the same existing route mechanism and fixed controller branch, assessed from source. |
| Frozen source/proof, separate explicitly trusted skill/reference inputs | Read-only supervise prepares a fresh detached canonical checkout at the frozen execution head/base and a bounded link-refusing proof snapshot. Admission selects regular canonical blobs; staging reads their exact identities. Packaged AgentEnv is empty and Pi discovery disabled. | A fails source/proof/trusted writes, permits scratch/checkpoint writes, and compares admitted selected blob IDs with host Git. Later canonical/candidate instruction edits do not replace selected fixture markers. Candidate audit.json, executable, extension, broad guidance and unselected skill/extension markers never act or enter model instructions. Existing G13 observes nonempty approved proof snapshots after live proof changes. |
| Selected Markdown and referenced regular files only; no candidate-controlled extension discovery | Plan validates explicit paths/counts. Loaded::inputs selects only the named canonical regular entries with size bounds. Controller permits only declared regular files; no reference expansion occurs. | A linked and oversized canonical inputs refuse by path/reason without attempt/execution; higher-priority malformed tickets cannot block a valid audit and cleanup. Declared traversal refuses before calls; undeclared traversal fails allowlist lookup; oversized source returns the spec's 32768 bytes and truncation flag. Successful reads/publication control each boundary. |
| Writable scratch distinct from execution/node-scoped Durable data; no host code or authoritative checkpoint evidence | Read-only preparation exclusively creates audits/<execution>, refuses reuse and stages only into that fresh root. Box mounts only that execution's state; row binds its admitted node. Controller creates fresh scratch/checkpoints and opens SQLite only in the guest. Cleanup removes whole attempt without following guest links. | A retains the predecessor checkpoint on the host while the edited successor is held. Actual successor mounted state lacks its marker; both rows bind implementer and have null resumed. A checkpoint symlink survives until checked cleanup without changing the host sentinel. Deleted ineffective host-path probes are not credited. Future exclusive resume is Y-65 and is not implemented or tested here. |
| Validated report through existing execution-scoped registration/session | Native Pi owns the one MCP session. Anonymous pipes bridge bounded report data; child inherits no bearer/session. Daemon validates schema/outcome/byte bounds and binds execution/node/plan/ticket/inputs/base/head/proof, never worker identities. | A publishes the independent fixture's schema-valid test-audit summary/findings through registered native Pi. Multibyte oversized summary is refused before valid publication. R refuses blank, duplicate and wrong-session inherited-bearer calls, then accepts the native report and checks its exact frozen binding. |
| Report outcome distinct from infrastructure; bare exit is an error | AuditReader normalizes native terminal frames with unknown usage. Supervise requires registered success plus report. executions::end records infrastructure separately; RESULT_ELIGIBLE alone requires ended, unreserved, checked teardown and actual output. | B endless reads stop at 12 requests with audit_limit; bare answer gives missing_report, no report/output/outcome and ineligibility. Valid publication controls both. R separately exercises successful ordinary terminal text without publication, superseded late publication and unknown legacy admission. No usage totals, cost or completeness are invented. |
| Yard owns lifetime, deadlines, cancellation, publication and routing; Durable owns conversation only | Existing run_harness handles stop and total/inactivity limits. Yard revokes grants and brings down the box; capacity proves actual box absence and inherited host lock freedom before release. Durable wait/SQLite never owns Yard scheduling. | B stops the selected controller's own held host-route reply. External box-up EOF hold keeps the real box/reservation and prevents continuation. Checked process/box absence and host lock freedom precede subsequent start. Native publication followed by actual box absence but held host lock stays ineligible/reserved with no cleanup; checked release preserves the same report/result and permits cleanup without replay. Deadline classification is source-only; no clock-wait test is claimed. |
| Single daemon writer, exact identity, one lifecycle, grants, containment, shared capacity, target serialization/CAS and intent recovery | Package adds no store writer, execution kind, owner or lifecycle. It uses existing report transaction, admission identities, effect reservation, worker_spec, target lock and cleanup/reconciliation. Queue controls are unchanged. | A/B/R observe daemon-bound results and checked ownership. Inherited complete host2089 includes G1/G2/G5/G6/G10/G11/G13/G14 scenarios for these shared controls. This document adds no replacement lifecycle, approval shortcut or manual parent completion. |
| Explain dependencies and update implemented binding clauses in the same implementation commit | Landed spec describes boxed Node and locked Durable/AI/Chord dependencies, publication bridge, state separation, grants and G8/G12/G13. Package README supplies reviewed image scaffold. | Dependency reasons: Node runs boxed Durable SQLite; pi-durable owns internal conversation persistence; pi-ai supplies the routed model client; chord supplies context/tools; pinned transitive telemetry avoids an ambient version. Host runtime dependencies remain unchanged. No dependency is added in this assessment. |

All original runtime acceptance mechanisms have an inherited observed
scenario. Source-only configuration branches and operational exclusions above
are not renamed runtime passes. The fixture supplies report content and trusted
Markdown; its pass proves execution/selection/publication, not model quality
or the effectiveness of a particular real audit. No speed, cost or quality
claim follows from it.

## Scoped test-audit dispositions

Applied test-audit to the integration evidence, not the whole-tree release
sweep. Read both package scenarios and all shared report scenarios, and compare
the G13 proof/host-gate and G14 worker-Git/proof-link mechanisms. Keep A/B
(G8/G12/G13), all four R scenarios (G8/G2/G6/G7), and those G13/G14 controls.
No assertion is added, rewritten, removed or weakened. No guarantee row changes.

| Retained mechanism | Hardest case, external oracle and meaningful sabotage |
|---|---|
| Installed identity versus admitted identity | Missing module then correctly resealed changed runner, followed by successful original image. Real boxed bytes/seal/sentinel and host call count. Skip the respective seal/import or admitted hash checks. These are distinct guards. |
| Frozen trusted inputs and bounded reads/reports | Later canonical/candidate edits, traversal resolving to an existing file, oversized source and multibyte report. Host Git and fixture markers/bounds. Stage mutable/broad inputs, allow undeclared paths, or count characters instead of bytes. |
| Route and package containment | Host receipt key versus guest mounted roots, logs, store and candidate executable sentinels. Fixture key and bytes. Use a host client, literal guest key, candidate launcher or candidate extensions. |
| Exclusive state and cleanup | Edited successor while predecessor checkpoint remains; guest-planted link through cleanup. Actual mounted-state marker and host sentinel. Reuse predecessor state, enable resume or follow cleanup links. G14 instead observes writable harness-state staging and candidate proof refusal. |
| Bounded iteration and real publication | Endless reads and bare terminal answer followed by valid native publication. Fixture report and spec request bound. Remove bounds or infer a report from exit text. Ordinary R exercises the daemon boundary without packaged native terminal frames. |
| Stop and ownership settlement | Selected controller's held model reply, externally held box teardown, then report publication with same-inode host lock refusal despite box absence. /proc, pinfold, host lock and ordered history. Cancel only wait or release/admit cleanup before checked ownership. |
| Daemon authority/session and edit/legacy handling | Wrong-session/blank/duplicate publication; late superseded report; real stopped old planner upgrade. Fixture report/revisions, Git and historical rows. Skip schema/session guards, overwrite publication, accept stale revision or invent legacy admission. |

The shared daemon tests cannot replace actual packaged asset execution,
iterations, native terminal and selected stop boundaries. G13's nonempty
approved proof after live writes does not replace A's exclusive successor
checkpoint observation. Keep both. Expected values come from spec, fixture,
Git, pinfold and host files; binding equality is itself a stated guarantee.
Refusals have same-project success controls and reason/identity observations.
Holds are real external calls, not binary barriers. No sleeps, masked retries,
mock runtime, public model endpoint or new crash point is introduced.
Sabotages are named in comments and assessed from source; none ran here.
Test lines removed: 0. Binary lines removed: 0. Operator escalation list: empty.

## Validation and mandatory parent acceptance

Before commit, cargo fmt --check and cargo clippy --all-targets --locked --
-D warnings passed; git diff --check passed. No host e2e ran here, as required.
No development binary opened the construction backlog. No .yard, operator
image, model, reviewer, credential, operational binary or daemon was changed.

Y-64 still requires fresh exact-head operator host validation, published
review and actual landing gates. Retain both complete package scenarios and
the shared controls. Record candidate and actual merged heads, the terminal
host outcome and checked cleanup; do not substitute host2089 for that new run.
Its FUSE x86_64 result proves neither native-overlay behavior nor arm64/Mac
release validation. Earlier native failures remain failures; deleted probes
remain deleted evidence. The 924.47s run exceeds the historical five-minute
estimate without authorizing test deletion, deadlines or performance claims.

This worker record does not declare Y-64 done or landed. Full redesign/release
scope through Y-80 remains required, Y-81 deferred. Automatic checkpoint
resume remains Y-65. No graph delegation, writable audit, general package
manager, host controller, host Node installation or runtime switchover is added.
