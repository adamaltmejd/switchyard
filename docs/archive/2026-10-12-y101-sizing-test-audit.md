# Y-101 selection boundary and proposed split

Sizing only. No runtime, living contract, test or assertion changes.
Y-101, Y-100 and Y-93 remain incomplete. This is source/test boundary
evidence, not an implementation or runtime pass.

## Inputs and landed prerequisite

This checkout contains Y-99 merge `7a2bbfe` and operator record `a88a328`.
`docs/archive/2026-10-12-y99-operator-validation.md` records its actual
landing. Its focused and full host passes belong to Y-99, not selected
packaged execution.

The agreed `docs/archive/2026-10-09-switchyard-redesign-plan.md` was read.
`docs/archive/2026-10-12-y100-sizing-test-audit.md` is absent from this base.
The complete attempt89 assessment supplied in Y-101's public body was used
as source/test boundary evidence. It is not recreated as a landed archive
or treated as implemented behavior. Attempt89 was retired without landing.

## Source trace

`plan.rs::Runner` describes Agent and Command. Profiles expand node defaults;
`resolve_plan` validates typed references and normalizes ordinary selections.
`runnable_workflow` admits only a selection matching a configured ordinary
workflow. A packaged declaration alone cannot make an execution runnable.

`jobs/admit.rs::admit` records `ordinary-linear`, immutable selected agent
definitions and broad `Loaded::admission_inputs`. `jobs/mod.rs::read_config`
inventories the agent environment's allowlisted regular Git entries.
`load_attempt` compares the admitted inputs and resolved plan against their
original configuration commit. Adding selected inputs requires preserving
that comparison and the ordinary admission contract, not bypassing either.

`jobs/supervise.rs::implement` supplies Y-99 execution-exclusive source,
proof and writable state. It still calls `AgentEnv::load` on the broad base
allowlist. Pi stages its broad extensions and skills and passes base guidance
in argv. Selected instructions need canonical regular-file identities and
separate read-only staging. A candidate `audit.json` or an instruction copied
from the source mount cannot supply that authority.

`pi-mcp-extension.ts::register` initializes the real native MCP session and
registers the daemon's unchanged grants. `registerAuditChild` adds a local
model-elected tool. That tool launches the fixed image-owned controller with
anonymous publication pipes and routing variables only. Its resolved result
is text containing child output or exit status. It does not supply a distinct
daemon terminal event. Deterministic selection must remove model election
without replacing native registration/session proof or treating that text
as completion evidence. No native Pi hook behavior was verified in this
attempt; no new launch design is claimed.

`harness.rs` pins the actual pinfold harness artifacts. The packaged image
identity must remain separate. `runner.mjs` checks runtime and exports before
controller model calls, bounds reads and requests, and refuses bare text.
It currently reads `/workspace/audit.json` and uses `/workspace/trusted`.
Those package roots must change with daemon-owned selected launch staging;
they must not become a second candidate-controlled launch path.

`jobs/report.rs::publish` binds one report to the running admitted node and
rejects supersession. `implement` separately reads registration, terminal
status, report, stop state and checked teardown. The existing reservation
and ownership proof remains authoritative. A child completion frame cannot
replace any of those readings.

## Split proposed through the sizing exit

Proposal 227 (`y101-selected-input-resolution`) is the shared resolver/input
binding change. It gives trusted inspection an explicit packaged declaration
and canonical selected-file identities, observed through the existing typed
plan inspection scenario. It adds a G6/G8/G13 inspection clause in its own
implementation commit. It admits no packaged execution and claims no runtime
freeze. Unsupported runtime stays refused. This avoids an interim executable
or a partially runnable admission path.

Proposal 228 (`y101-selected-runtime`) depends on 227 actually landing.
It retains the complete original Y-101 runtime and acceptance, consuming
those bindings for immutable admission and read-only staging, deterministic
boxed native-Pi invocation and the complete two package scenarios. It must
prove that admission uses the selected inputs; inspection evidence alone is
insufficient. Proposal 229 links Y-101 to that dependent child as integration,
without replacing or shortening its public body.

All three are proposals for the operator. They are not admitted work or
completed dependencies. Preserve the current implementer and reviewer.
Y-100 still requires the separate selected supervision child to actually
land. Y-93 retains its entire integration contract after actual Y-100 landing.

## Test-audit gate assessment

This is a scoped assessment, not a sweep. Keep both complete
`g08_durable_package.rs` scenarios and their corrected Y-99 observations.
No assertion is removed, rewritten or weakened in this candidate.

The first package scenario holds outer Pi's second request after publication
and a replacement outer Pi's first request. Deterministic invocation removes
those requests. The runtime child must replace the holds with real selected
controller observations and preserve the isolation blocks. Deleting the
holds without replacing their observers would lose the hard case. This is
distinct from the dependent supervision child's operator stop while a
selected-controller request is held and its failed-teardown case.

| Clause retained for implementation | Sabotage to name | Distinct hardest case | External oracle |
|---|---|---|---|
| G6/G8/G13 selected-input inspection | Resolve against a different target or substitute candidate paths; admit work while inspecting | Named-target profile differs from default; later canonical inputs differ from an earlier inspected commit; malformed input has a corrected control | Host Git blobs, literal fixture declarations, external store rows |
| G8 explicit selection and native publication | Restore outer-model election, accept bare exit, or bypass native registration/session proof | Missing sealed asset then corrected image in one project; bound and bare-answer refusals retain publication controls | Fixture-supplied schema-valid test-audit report, host-route requests, CLI/store bindings |
| G8/G13 instruction authority | Load candidate launch data, broad guidance or unselected extensions; stage from a later canonical commit | Selected skill/reference differ from candidate executable/extension and later canonical contents | Fixture markers and sentinels, admitted host Git blobs, real boxed read/write probes |
| G12 secret containment | Pass the upstream key into child environment, launch input, checkpoint or model body | Selected controller with guest/source/state/log/store probes and real host route injection | Fixture key and host-route header, clean probes and model bodies |
| G13 exclusive state and frozen read-only mounts | Mount predecessor writable state, resume its session, omit proof or make selected inputs writable, follow checkpoint links | Same-attempt edited replacement while predecessor checkpoint remains; selected controller fixture holds replace outer Pi holds | Fixture checkpoint and host sentinel, live box probes, status history and external rows |
| G8/G13 stop retains ownership until checked end, deferred to supervision child | Release reservation or admit continuation before controller and box are gone | Operator stop while selected controller's own host-route response is held | Fixture hold, real pinfold listings/process observations, status history and store |
| G8 failed teardown leaves publication ineligible, deferred to supervision child | Count publication or child success before checked teardown | Publish then fail teardown/ownership proof, with successful teardown control | Fixture report, real external box/lock hold, store eligibility/reservation and history-bound successor |

The implementation commits must name the exact current clauses they add or
extend. The runtime child preserves package bounds, traversal refusal,
publication identity and supersession, selected canonical freezing, exclusion
of broad environment and candidate code, no outer model call, secret
containment and all Y-99 isolation blocks. No inherited runtime pass satisfies
those changed mechanisms. The supervision exclusion removes no current
lifecycle control or original Y-100/Y-93 acceptance.

Binary lines removed: 0. Test lines removed: 0. No guarantee clause changed.
No installed `.yard` edits, host controller/client, package management,
arbitrary graphs/delegation, resume or performance claim.

## Verification

`cargo fmt --check` passed in this candidate box.
`cargo clippy --all-targets --locked -- -D warnings` passed in this candidate
box. Host e2e was not run and remains operator-owned after approval. A check
pass on this documentation-only candidate is not evidence of selected
runtime behavior.
