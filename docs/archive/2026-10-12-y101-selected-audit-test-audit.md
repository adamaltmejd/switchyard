# Y-101 selected packaged audit implementation and test-audit

This candidate implements Y-101's selected runner and instruction authority.
The earlier sizing-only candidate, recorded in
`2026-10-12-y101-sizing-test-audit.md`, was blocked for incomplete delivery.
This implementation supersedes its proposed split. Proposals 227, 228 and
229 are not prerequisites or completed dependencies. That historical record
is unchanged. Y-99 is landed at 7a2bbfe with operator evidence at a88a328.

The redesign plan and the complete Y-100 sizing assessment supplied in the
public ticket were read. This implements the first selection child only.
Y-100 and Y-93 retain their complete integration acceptance. Held
selected-controller cancellation and failed-teardown acceptance remain with
the dependent supervision child, which must actually land before Y-100
completes. Existing lifecycle controls remain authoritative.

## Implementation boundary

`Runner::Packaged` selects only `durable-audit`. Its supported plan has one
read-only `implementer`, report output and `complete`, matching an existing
read-only workflow's Pi agent. Operator graphs, presets and profile overrides
use the existing resolver. An optional explicit workflow profile also permits
CLI/scheduler admission. It is not inherited. Packaged topology never falls
through to the ordinary matcher. Arbitrary graphs and delegation stay refused.

Admission binds selected canonical regular-file paths and Git blobs, plus
fixed image package identity. Readback re-resolves against the original
admitted commit. Execution stages those blobs under read-only
`/yard/input/trusted`, separate from Y-99 source, proof and writable state.
The broad base agent environment is empty for this runner. Yard generates
launch input; candidate `audit.json`, executable, extension and skill cannot
supply instruction or launch authority. References are explicitly declared,
never discovered or followed automatically.

Real Pi remains pinfold's pinned harness and owns native MCP registration and
its execution session. Its selected native input hook invokes the fixed
image-owned launcher before outer model dispatch. Anonymous pipes carry one
bounded report through the unchanged native publication client. The child
inherits routing variables, never bearer/session or upstream key. Its terminal
status is separate from report outcome. Native publication, checked teardown
and reservation release still govern result eligibility. The package identity
does not invent a pinfold artifact. There is no host Node or fallback.
The native bridge compares admitted launcher/controller/lock hashes before
executing package code, so an older sealed image also refuses by identity.
The fixed launcher then checks the sealed runtime closure.

No separate execution kind, owner, slot, recovery path or publication contract
was introduced. Existing deadline, cancellation, ownership and cleanup code
was retained. ARCHITECTURE.md changes selection, admission, worker grants,
dependencies, the box table and the package clauses of G8/G12/G13 together.
G12/G14 and the remaining exact identity/publication clauses are preserved.

## Scoped test-audit gate

Both complete scenarios in `g08_durable_package.rs` are extended. No scenario
or guarantee is deleted. The operator's acceptance list is assessed below.
Each changed block uses the CLI, trusted operator MCP, canonical config/git,
real daemon/Pi/package/box or an external fixture observation. Expected report
content is an external schema-valid test-audit report supplied by the scripted
host model, never Yard's output.

| Clause | Sabotage named in the test | Distinct hardest case | External oracle |
|---|---|---|---|
| G8 explicit single-node selection and native publication | Restore outer-model election, accept packaged topology through the ordinary matcher, skip package identity checks or ignore child terminal status | Missing sealed entrypoint then corrected image in one project; every model request must offer the controller read surface; unsupported topology has the supported profile control | MCP refusal/shape token; zero host model requests before correction; fixture report; external execution/session/registration rows |
| G8 bounded publications | Remove request/read/declaration bounds, count UTF-8 characters as bytes or synthesize a report from terminal text | Traversal declaration normalizes to an existing source file; oversized source and oversized multibyte summary; endless reads and bare answer followed by explicit publication in the same project | Spec's 32768-byte read and 12-request bound; scripted tool results/refusals; no report/output/outcome and ineligible result; fixture report positive controls |
| G8 publication remains ineligible before checked end | Count a native report before the live execution releases ownership | Real controller/Pi publish and exit, while an external pinfold exec wrapper remains held and its box still exists | Fixture FIFO hold; CLI publication plus running result absence/reservation/ineligibility; eligible completed control after checked teardown |
| G8/G13 selected instruction authority | Load broad guidance/extensions, candidate launch/skill/code or current inputs instead of admitted blobs | Canonical skill/reference and candidate skill change after admission; an edited replacement execution must still load old selected markers | Host Git blob identities; fixture skill/reference markers; hostile executable/extension sentinel; model bodies and boxed excluded-input/write probes |
| G12 secret containment | Pass bearer/upstream key into child inputs/state/model bodies, or bypass injecting route | Native selected controller with real routed model traffic and guest source/proof/input/state/package probes | Fixture upstream key in host route header only; guest env/files, captured streams, model bodies and external execution/attempt/audit store rows |
| G13 exclusive read-only audit state | Reuse predecessor state, resume its session, omit/widen proof mount or follow guest checkpoint links on cleanup | Edit while completed predecessor exec is held; hold replacement controller's first host model request while predecessor checkpoint remains on host | Real box checkpoint/link planting; host checkpoint and sentinel; fresh source/trusted/proof write failures, scratch/checkpoint write successes; two execution node/resume rows and history-bound cleanup |

Existing ordinary publication/ownership tests do not cover deterministic
package selection or canonical selected inputs. Existing writable repair tests
do not cover exclusive read-only checkpoints and link-safe whole-attempt
cleanup. Those shared tests remain unchanged. No duplicate scenario was added.

The former outer-Pi post-publication model hold is replaced with a real host
pinfold exec wrapper hold after actual native publication and Pi/controller
exit. It keeps the real box live until the fixture releases the FIFO. The
replacement hold is the selected controller's own real host-route response.
All Y-99 predecessor checkpoint, fresh proof/state, no-resume/node identity and
host-sentinel cleanup blocks are retained. This proves edited replacement
isolation; it does not claim the dependent child's held cancellation or
failed-teardown acceptance.

Test source lines removed: 82; binary source lines removed: 127. These are
replacements of the model-elected helper, launch fixture and staging paths,
and consolidate canonical metadata reads; they are not deleted guarantees or
scenarios. No binary test barrier, mock, sleep,
retry, public model or timeout/concurrency masking was added. No installed
`.yard` file or implementer/reviewer selection was changed.

## Validation

`cargo fmt --check` and
`cargo clippy --all-targets --locked -- -D warnings` passed in the candidate
box. Clippy includes compilation of both extended scenarios. The host e2e
suite was not run, as explicitly required by the ticket. Native Pi input-hook
dispatch, image preflight and fixture holds require that operator-owned host
landing gate after approval. This record is source/test gate evidence, not a
runtime pass, landing, or completion of Y-100/Y-93.
