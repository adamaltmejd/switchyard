# Y-74 presets and targeted overrides

The scaffold ships ten complete typed graphs. The existing resolver handles
TOML, structured MCP graphs and node overrides. No execution lifecycle,
provider, capacity override or worker grant was added. Operator catalog
discovery exposes definitions and settings from one canonical target.
OpenRouter and opencode-go remain existing provider connections.
Ordinary implementer substitution now accepts a declared agent profile with
the same access, outputs and outcomes, including another configured agent.

## G6 assertion disposition

Adapted `selected_admission_refuses_without_work_and_freezes_operator_inputs`.
The existing held execution now selects a named profile using a distinct
configured agent. The complete literal expected plan remains the observer.
An unavailable override must return `invalid`, naming
`overrides.implementer.profile` and `unresolved_reference` before the same
project admits the valid control. These protect wrong role selection and
unresolved capabilities at the operator boundary. Sabotage is ignoring the
profile override or accepting an unavailable profile. Both are named in the
test comment. Expected settings come from supplied TOML and the literal
complete graph, independently of Yard's output.

No machines, historical builds, boxes, permanent fixtures or model turns
were added. No assertions were deleted. The existing upstream writer,
finding-selection, concurrent join, classification and cleanup scenarios
retain execution coverage. No duplicate preset happy-path suite was added.
Every scaffold variant and model combination is deliberately untested.
The test replaces two source lines. Binary deletions are predominantly the
34-line inline scaffold moved into the embedded TOML asset; the role guard
also replaces its ten-line same-agent branch. No test coverage was removed.

## Verification

The development build used this checkout's target directory. An attempted
operator smoke check used separate state, config, cache, runtime, socket
and project paths under `/tmp/y74-validation`; it never opened construction
state. Startup refused because this container lacks pinfold. Downloading the
CI-pinned pinfold release returned HTTP 403. No stub was substituted.
The host e2e suite was not run, as instructed; its gate follows approval.
Required fmt and locked all-targets clippy checks and a build are recorded
with the candidate. A standalone TOML syntax lint parsed ten presets and
fourteen profiles; it did not exercise Yard resolution or execution.
No installed project `.yard` configuration was changed.

## Operator host qualification and cleanup correction, 2026-10-15

The operator qualified c94312c with binary SHA256
`cb77b8a63c9d4415b358423e795fee4f0dc4fe6c9bee0c32a02106ed5568b706`.
The modified G6 case passed 1/1 in 22.37 seconds. Evidence is retained at
`/home/yard/validation/y74-c94312-g6-20261015-evidence`.

A separate real candidate daemon and operator MCP probe resolved all ten
presets from a committed fresh scaffold. Nine were runnable. Cleanup was
structurally valid but not runnable: `shape=bounded-repeat`, `workflow=null`.
Full responses are in
`/home/yard/validation/y74-c94312-all-presets-20261015/resolved.json`.
Its `terminal.json` records unchanged zero ticket, attempt and execution
counts and unchanged audit sequence 1. The first-failure evidence remains at
`/home/yard/validation/y74-c94312-scaffold-smoke-20261015`.
Both isolated daemons exited cleanly. No model was called or box created.
These are operator-supplied results, not checks rerun inside this container.

The shipped cleanup implementer incorrectly copied ordinary feedback.
`Plan::handoff_selection` requires its original implementer to have empty
inputs; feedback belongs to repair. The correction removes only that
implementer input. Inspection of the complete retained contract confirms
all three roles share workspace `implementer`, use declared matching
profiles and writable access, and publish candidate, report and candidate
respectively. Cleanup requires the original candidate. Repair requires the
cleanup report and retains optional external feedback. Only repair feeds
candidate checks and receives their bounded failure routes. The merged
landing input remains external. Reducing this handoff still yields the
ordinary retained gate and review chain.

The runtime guard, other nine presets, profiles, discovery and G6 assertions
are unchanged. No new fixture or topology suite was added. The operator will
rerun the cheap no-work MCP inspection; host e2e remains operator-only.
After correction, `cargo fmt --check`,
`cargo clippy --all-targets --locked -- -D warnings`, `cargo build --locked`
and `git diff --check` passed in this checkout. No host suite was run.
