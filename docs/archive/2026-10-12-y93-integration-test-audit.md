# Y-93 full Durable audit integration and test audit

Reviewed canonical base `c6daac55483db9ba8a25e10a6c8a9b45b9ab737e`
against the complete original Y-93 body and October 12 operator partition.
The earlier documentation-only candidate is not integration evidence.
This assessment follows actual landing of every prerequisite:

| Prerequisite | Actual landed head | Operator record |
|---|---|---|
| Y-91 publication | e65730bc84ab99974e65966b32d3c181751524a3 | 2026-10-11-operator-y91-validation.md |
| Y-92 package | 9c106745a01ddc443c937d1a543b8e45fb017ef2 | 2026-10-12-y92-operator-validation.md |
| Y-99 isolation | 7a2bbfe1f7b011fca85481f0cbdb83997834b5b0 | 2026-10-12-y99-operator-validation.md |
| Y-100 selected runtime integration | 7272212aa161b7ab814b28529914ac327d0cecc9 | 2026-10-12-y100-operator-validation.md |

Git confirms all four are ancestors of this base. `git diff 7272212 HEAD
-- crates runners` is empty. The latest operator record therefore validates
the same combined production and test source reviewed here, rather than a
stale child-only implementation. It records actual merged host2074 passing
75 scenarios, zero failures, in 576.03 seconds, followed by cleanup2075
ending removed. Both complete package scenarios and shared report controls
ran. Earlier child-only results are not substituted for that combined run.
The record is authoritative host evidence; its raw host artifacts are not
mounted in this worker box and were not independently reopened here.

## Original contract assessment

Read selection and admission in `config.rs`, `plan.rs`, `jobs/mod.rs` and
`jobs/admit.rs`; staging and supervision in `jobs/supervise.rs`,
`agent_env.rs`, `pi.rs` and `pi-mcp-extension.ts`; publication and settlement
in `jobs/report.rs`, `store/executions.rs`, `jobs/capacity.rs`,
`jobs/cleanup.rs` and `jobs/proof.rs`. Read the fixed launcher, controller,
Dockerfile and package instructions. Compare ARCHITECTURE's threat model,
selection, publication, box table, grants, dependencies and guarantee rows
with the implementation and README. The following assessment covers every
original runtime and acceptance clause.

| Original clause | Owning implementation and current external evidence |
|---|---|
| Optional trusted operator runner/profile on the one state machine | Config accepts explicit noninherited workflow profile selection. Plan accepts only one read-only implementer with report output and complete outcome. Unsupported packaged topology cannot fall through to ordinary matching. The package scenario refuses topology and controls it with supported CLI admission and real publication. |
| One box/controller occupies one execution slot | Supervise uses the existing implementation execution and real Pi box, with no child execution. Capacity reserves before effects and releases only after box absence and inherited host lock freedom. The selected stop and ownership scenarios observe retained reservations and release before successor admission. |
| Fixed trusted image invocation; no candidate entrypoint | PackagedAudit stages Yard's native Pi hook. The hook checks admitted launcher/controller/lock hashes, then spawns the fixed image launcher before outer model dispatch. Package assets live outside workspace. Every host fixture request offers read_declared; candidate executable/extension sentinels stay absent. |
| Frozen read-only source and proof, separately selected trusted files | Read-only preparation makes a fresh detached checkout at the execution head or admission base and copies proof through the bounded link-refusing snapshot function; absent proof is an empty mount. Input is a separate read-only mount. Admission selects regular canonical Markdown/reference blobs and staging reads those exact objects. Later canonical/candidate edits preserve the old fixture markers and Git identities. Source, proof and trusted-input writes fail. |
| Explicit trusted instructions; no broad base allowlist | The selected runner uses AgentEnv::empty and disabled Pi discovery. Only selected Markdown supplies controller instructions; references must be explicitly declared. Model context excludes broad guidance, skills, extensions and candidate launch data. Selected skill/reference markers are present, with successful declared reads. Canonical link/size refusals name paths before creating work and have a same-project valid admission control. |
| Exclusive execution/node state and link-safe host handling | Preparation creates a new audits/execution directory and refuses reuse. Only its state subtree is writable in that box. Host preparation never stages into predecessor guest state; cleanup removes the attempt without following guest links after ownership release. The edit scenario keeps the predecessor checkpoint on the host while the replacement's actual mounted state lacks its marker; host bytes remain unchanged. A planted checkpoint link cannot change the host sentinel through cleanup. |
| Ownership identity for later exclusive resume; no resume now | Execution binds attempt, plan revision and admitted node; state path is execution-scoped. Read-only preparation skips predecessor session selection. Both edited executions retain the fixture's implementer node and null resumed. Durable creates fresh scratch and checkpoint directories and never opens predecessor checkpoints. Y-65 remains excluded. |
| Injecting model route/placeholder and per-execution native MCP discipline | PackagedAudit retains Pi's real artifact, stage, route and environment. Native Pi initializes the single session; the child inherits routing variables only. Anonymous pipes carry publication to that registered native client. Host fixture Authorization receives the upstream key; model bodies, guest environment/source/proof/input/state/assets, streams and store rows lack it. Shared report controls refuse wrong-session inherited-bearer calls with 404 and accept native publication. |
| Yard deadlines, cancellation, publication and checked teardown | Supervise owns total/inactivity deadlines, stop and box teardown, and revokes the grant before host inspection. Capacity retains ownership until actual box absence and inherited lock freedom. The selected controller's host-route reply is held during supported stop; an external box-up EOF hold proves reservation and no successor while the box remains. Process/box absence and host lock freedom precede the subsequent selected execution. |
| Missing/incompatible runtime before model calls; no host Node/fallback | Native hook verifies admitted hashes; launcher verifies the sealed closure and Node; controller checks exact runtime, lock, package imports and exports. Missing sealed entrypoint and changed image-owned controller with a consistent seal name runner_unavailable and asset identity with zero model calls. Corrected image succeeds in the same project. Host fixtures copy public archives as data; installation and all package execution occur inside image/box. |
| Bounded read-only tools and schema-valid test-audit report | Controller offers read_declared and native report publication only. Bounds cover declared files, reads, launch input, model requests, responses, tools and report bytes. Fixture supplies the expected summary/findings independently. Traversal declaration, undeclared tool key, oversized source, multibyte summary and endless reads exercise distinct guards; valid declarations/publication control the refusals. |
| Report outcome distinct from infrastructure failure | Daemon validates and binds one publication without accepting worker identities. Native terminal status is separate from report outcome. Store has one eligibility expression requiring ended status, released reservation, checked teardown and output. Bare answer produces no output or outcome; native publication with held host ownership remains ineligible despite successful terminal status and actual box absence. Checked release preserves the same bound report/result without audit replay. |
| Current spec, dependencies, box/grants and G8/G12/G13; preserve G12/G14 | Landed selection, publication, Boxes, Worker tools, dependency and G8/G12/G13 clauses describe these mechanisms. Node/npm remain trusted boxed dependencies, separate from pinfold Pi's artifact pin. Existing injecting routes, native session controls, untrusted Git/proof handling and G12/G14 clauses remain intact. Correct the general harness-state paragraph that still called all implementer state shared per attempt; the audit-specific paragraphs and box table already require execution exclusivity. |
| Existing assertions and full release obligations | No assertion, scenario, guarantee, dependency or operational control changes. This is integration assessment, not a release audit or a sizing/split disposition. Y-64 and its full downstream release scope remain dependent on actual Y-93 acceptance. No performance claim follows from the combined host run. |

No production gap was found after this full original-clause assessment.
The implemented behavior comes from the actually landed combined source,
supported by its actual mixed-tree host gate. Adding another controller,
lifecycle or fixture would duplicate the current mechanism. The spec change
repairs a contradictory description of that existing state ownership.

## Scoped test-audit dispositions

Read `crates/e2e/src/lib.rs` and `model.rs` whole, both complete scenarios in
`g08_durable_package.rs`, all four scenarios in `g08_audit_reports.rs`, and
the complete G13/G14 files for overlapping mechanisms. This is the Y-93
integration audit, not the whole-tree pre-release sweep.

Keep the missing-assets/corrected-image scenario (G8/G12/G13), including
its edited successor, malformed-admission and changed-seal controls. Keep
the iteration/publication scenario (G8/G13), including selected stop and
post-publication host ownership refusal. Keep all shared publication blocks
(G8/G2/G7) and the G13/G14 containment scenarios. No block is added, rewritten
or removed. Gate answers for the retained integration mechanisms:

| Mechanism | Sabotage named in test comments | Hardest distinct case and external oracle |
|---|---|---|
| Package identity and deterministic launch | Skip asset checks/admitted hash loop; restore outer-model election or candidate execution | Missing module then sealed changed controller, both before calls; real boxed seal/runner bytes/sentinel, host request count and same-project corrected report. These are different checks. |
| Frozen selected instruction authority | Stage broad guidance or current canonical/candidate files | Canonical/candidate edits after admission and an edited execution; host Git blob IDs and fixture old/excluded markers. |
| Read/report bounds | Remove declaration/read/request guards; count UTF-8 characters instead of bytes | Traversal resolves to an existing file; oversized source/summary and endless reads; expected bounds from spec and tool results/report from fixture. |
| Secret containment | Persist key/bearer or bypass injecting route | Real routed model traffic and guest probes over all mounted roots/state/package; host fixture key and clean guest/streams/model bodies/store. |
| State/proof and link isolation | Mount predecessor state, enable resume, omit/widen proof mount or follow cleanup links | Edit after real native publication; retain checkpoint during replacement's own held model request; fixture checkpoint/sentinel, actual mount probe, null resume and node rows. |
| Stop and exclusive checked release | Release before controller/box disappearance; cancel only wait or admit while reserved | Selected controller's own held host response across external teardown hold; guest /proc, actual box presence/absence, host lock and ordered history. |
| Publication eligibility | Synthesize report from terminal text or count native success before ownership proof | Bare answer and published report with actual box absence but same-inode host lock retained; fixture report, empty outputs or ineligible result, release-before-cleanup history and unchanged bound result after release. |
| Native session and daemon publication authority | Skip payload/session validation, overwrite or bind mutable identities | Blank then valid then duplicate/wrong-session publication; fixture payload, host Git and frozen identity agreement. |

The ordinary report controls cannot replace packaged import/launch checks,
Durable iterations or its native terminal/selected-controller stop boundaries.
G13's seat/gate proof scenario reads nonempty approved candidate proof; it
has no audit successor checkpoint. G14's writable state replacement and
candidate proof-link refusal do not observe the audit's actual mounted state
or cleanup link. Keep those survivors. Y-99's correction record is read:
deleted host-path probes are not credited as denial-of-access evidence.
Retained checkpoint bytes plus absence from actual replacement state prove
the isolation mechanism. No new twin or unshown Y-93 acceptance clause was
found in this scope. All refusals retain successful controls.

Holds are real host model/pinfold calls, not binary barriers. History waits
remain deterministic; no sleeps, retries, timeout waits, public model calls,
test-only flags or mock runtime are introduced. No crash scenario is added
and no sabotage execution is claimed. Test lines removed: 0. Binary lines
removed: 0. Operator escalation list: empty; no guarantee is weakened.

## Validation and actual parent acceptance

Boxed `cargo fmt --check` and
`cargo clippy --all-targets --locked -- -D warnings` passed before commit.
`git diff --check` also passed. Clippy compiled the combined test targets
but establishes no new host runtime pass. No host e2e suite ran here,
as the ticket explicitly requires it to remain the post-approval landing gate.

Fresh Y-93 acceptance still requires the complete configured real host gate
on the actual merged input, with both package scenarios and shared controls
intact. The operator must record exact candidate/merged identities, terminal
gate outcome and checked cleanup. The earlier combined host2074 is evidence
for the current unchanged runtime, not a renamed Y-93 run. Its isolated FUSE
Linux x86_64 pass is not native-store, arm64, Mac or release acceptance.
Its duration exceeds the historical five-minute estimate without authorizing
assertion removal, deadline changes or performance claims.

No .yard, host controller/client, package manager, graph delegation, resume,
image installation, release artifacts or operational daemon changes are made.
Y-93 is not declared landed or complete by this worker record. Y-64 and the
full downstream release clauses remain required after actual parent landing.
