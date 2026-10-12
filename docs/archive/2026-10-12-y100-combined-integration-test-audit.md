# Y-100 combined source integration and test audit

Reviewed combined base `1c72d012d18d2040b2bd121a35d07553f622a787` against
the complete Y-100 revision 2 contract, including its sizing addendum.
Y-99 landed at `7a2bbfe1f7b011fca85481f0cbdb83997834b5b0`.
The accepted selection child Y-101 landed at
`c826310620e4c2b3803110202d6a4ad3707f2aca`. The supervision child Y-102
landed at `b4b0d12098d9fb8046795f728b6db93b15a2f5e2`.
All three heads are ancestors of this base. Their operator records establish
actual landing, not merely accepted proposals or candidate approval.

The ticket's sizing archive at 56ac172 is absent from this checkout.
The complete public sizing addendum and the landed children's source and
audit records were read instead. No missing historical file was reconstructed.
The redesign plan remains the scope reference. This is a fresh combined
source review, not fresh host execution evidence.

## Source and contract

Read both complete scenarios in `g08_durable_package.rs`, their fixture
holds and external probes, and the shared `g08_audit_reports.rs` blocks.
Traced the selected runner through `plan.rs`, `config.rs`, `jobs/admit.rs`,
`jobs/mod.rs`, `jobs/supervise.rs`, `pi.rs`, `pi-mcp-extension.ts`, the fixed
launcher and controller, `jobs/report.rs`, `jobs/capacity.rs` and
`store/executions.rs`.

`Plan::packaged` accepts one read-only implementer with report output and
complete outcome, without graph inputs, requirements or routes. Packaged
nodes cannot fall through to ordinary matching. The selected agent remains
the configured read-only workflow's Pi agent. Profile selection is explicit.
Admission freezes canonical regular-file Git blobs and package identity.
Readback resolves the original canonical commit; execution stages those
blobs separately under read-only input. An empty broad agent environment
excludes base guidance, skills and extensions. Pi discovery stays disabled.

Real Pi registers the execution-scoped MCP session. Its selected input hook
verifies admitted image hashes and invokes only the fixed image launcher
before any outer model dispatch. The launcher checks the sealed closure;
the controller checks Node, lock and exports before model requests. There
is no host Node path or fallback. Routing variables alone reach the child;
the injecting route carries the upstream key. Anonymous pipes carry the
bounded report to the native client without a bearer or session in the child.

The existing implementation execution owns the box, deadline, stop signal,
route, reservation and teardown. Report publication stays distinct from
terminal infrastructure status. The sole result eligibility expression
requires ended status, released reservation, checked teardown and output.
Capacity release checks actual box absence and inherited host lock freedom.
It cannot start cleanup while ownership is retained. No additional lifecycle,
slot, owner, recovery path or publication authority is needed for integration.

Read ARCHITECTURE's selection, admission, publication, capacity, box table,
worker grants, dependencies and G8/G12/G13 clauses with README. The contract
already describes the combined landed implementation. Remove the stale
sentence saying runner selection remains future integration work and include
the existing noninherited optional profile in the workflow key table. Preserve
the threat model, all guarantee clauses, G12/G14 controls and exact bindings.
No production defect was found in this scoped source review.

## Combined test-audit gate

Retain both complete package scenarios and all shared publication blocks.
No assertion is added, rewritten or removed. The four gate answers below
map every original acceptance mechanism to current combined coverage.

| Clause | Named sabotage | Distinct hardest case | External oracle |
|---|---|---|---|
| G8 explicit selection and native publication | Restore outer-model election, ordinary matching for packaged topology, or terminal-text publication | Unsupported topology, then supported profile; real native controller returns an externally supplied test-audit report | Operator MCP refusal/shape; every scripted request offers read_declared; fixture report; native session and registration rows |
| G8 missing/incompatible image refusal | Skip seal/import checks or admitted hash comparison | Missing sealed entrypoint, then corrected image in one project; changed runner bytes with a valid replacement seal, then unchanged image | Zero host model requests; runner_unavailable and asset identity; real boxed seal and execution sentinel; repository runner bytes |
| G8/G13 selected instruction authority | Stage current canonical/candidate inputs or broad guidance/extensions; execute candidate code | Change canonical skill/reference and candidate skill after admission, then edit the held execution | Host Git blobs; old fixture markers and absent new/excluded markers; candidate sentinel; trusted/source write refusals |
| G12 route and secret containment | Pass key/bearer into child data, state or model bodies; bypass injecting route | Real selected native controller with model traffic and guest probes across source, proof, trusted input, state and package | Host fixture Authorization key; clean guest environment/files, streams, model bodies and external store rows |
| G13 exclusive execution state and G14 link-safe cleanup | Mount predecessor state, resume its session, widen proof access or follow guest checkpoint links | Edit after native publication while the old box is held; hold replacement's own model request while old checkpoint remains | Host checkpoint and sentinel; real boxed read/write probes; no-resume/node rows; history-bound cleanup |
| G8 existing package bounds and required publication | Remove request/read/declaration bounds, count characters as bytes or synthesize terminal reports | Traversal normalizes to an existing file; oversized source and multibyte summary; endless reads and bare answer, then explicit publication | Spec's 32768-byte read and 12-request limit; fixture tool results and reason tokens; absent output/outcome; same-project report control |
| G8/G13 cancellation through checked release | Release before controller/box disappearance, admit a successor while reserved or cancel only the wait | Selected controller's own host-route response held while supported stop crosses externally held box-up stdin EOF | Real guest /proc PID; pinfold box presence then absence; host lock freedom; reservation and release-before-start history |
| G8 publication with failed teardown remains ineligible | Count native terminal/publication before checked ownership or start cleanup while reserved | Published report and real box absence meet a held same-inode host lock; checked release permits cleanup without audit replay | Fixture report; host lock/device/inode; red:ownership execution identity; retained reservation and ineligible result; unchanged bound report/result and release-before-cleanup history |

The first package scenario combines image, selected-input, secret, bounded
read and Y-99 isolation controls. The second retains endless-read and bare
answer refusals and extends their successful control through selected stop
and ownership refusal. Shared audit tests establish payload/session validation,
duplicate refusal and identity agreement. Generic cancellation and review
ownership tests use other execution boundaries; they do not observe the
selected controller's own response or Durable native terminal frame.
No twin block or uncovered original acceptance clause was found in this scope.

All holds are real host fixture/model or delegated pinfold holds. Event waits
use status history. No crash scenario, sleep, retry, public endpoint, binary
barrier, timeout change or concurrency masking is introduced. Test assertion
lines removed: 0. Binary source lines removed: 0. No dependency, installed
image, .yard file, implementer or reviewer is changed.

## Validation and remaining acceptance

Boxed `cargo fmt --check` and
`cargo clippy --all-targets --locked -- -D warnings` passed. Clippy compiled
the current combined e2e targets without running them. No host e2e, package
execution, image build or sabotage execution was run in this box.

The child's host records are prerequisite evidence only: Y-101 host2000
passed 75/0 and Y-102 host2057 passed 75/0. Neither result is relabelled as
a fresh Y-100 pass. Their failed gates remain failed historical evidence.
Fresh operator-owned host validation of this exact combined candidate and
its actual merged input remains required after approval. Run the complete
configured host gate with both package scenarios and shared controls intact.
Record exact candidate/merged identities, gate outcome and checked cleanup.
This boxed audit does not complete Y-100 or Y-93. Only actual Y-100 landing
unblocks Y-93's complete original integration and fresh acceptance.
