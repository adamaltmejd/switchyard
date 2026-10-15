# Y-157 packaged cleanup retry inputs — 2026-10-15

The operator's exact e1b93d6 native diagnostic reached the held unsafe write.
Old cleanup execution 4 ended interrupted with teardown 1 and reserved 0.
The deliberate retry created execution 5 with no resumed predecessor and a
fresh checkpoint. The retained identity comparison shows one difference:
retry bookkeeping appeared at `$.inputs.feedback`, though cleanup declares
only the implementer's candidate. Other source, head, proof, plan, settings
and package identities matched. Execution 5 ended failed with `missing_report`.

The writer/handoff binding inserted implicit `start.feedback` for every
implementation execution. Packaged cleanup now receives only its declared
inputs. Declared input resolution still runs unchanged, including any
declared feedback. Ordinary writer and repair implicit feedback remains.
The exclusion happens when binding the invocation, before storing its inputs
or constructing checkpoint identity; no identity comparison strips or
normalizes a consumed input. Strict identity equality, checked predecessor
release, node/package binding and unsafe replay policy are unchanged.
ARCHITECTURE.md states this implemented boundary.

## Test-audit gate

Retain the G2/G9/G13 selected cleanup hard case and all its existing source
markers, held unsafe write, ownership/refusal, checkpoint/resumed identities,
repair/landing, independent grants and nested Git controls. Replace only the
continuation's latch-only wait with the existing host-model/FIFO/history
observers. When the scripted model sees the interrupted-tool result, it
writes `request` to a fixture FIFO before returning the same held response.
The host observes the replacement execution's start and calls
`Watch.said_before_end` for that exact execution. An execution end before the
FIFO signal now fails promptly. The replacement row query binds that same
execution; no other running cleanup can satisfy the identity assertions.
The response remains held until the existing deliberate release, after the
unchanged `after-release` source marker and predecessor/checkpoint checks.

The material failures remain duplicate writer/checkpoint ownership and
unintended effects before deliberate continuation. The test comment adds
undeclared retry feedback preventing continuation to its binary sabotages;
the existing unsafe replay and ownership sabotages stay. Expected bytes
come from the scripted model and host marker; Node's debugger, boxed SQLite,
native Git, pinfold and exact execution rows/history remain the observers.
The FIFO signal comes from the existing interrupted-result branch, with no
new model request. No machine, build, box, scenario, matrix, retry, timeout
or deadline is added. Reusing the existing hard case is sufficient.
No existing assertion or expected value is weakened or removed.

Test diff: 19 lines added, 3 replaced. Production diff: 2 lines added,
0 removed. This is a test-change gate, not a suite sweep.

## Diagnostic evidence and validation

Evidence remains at
`/home/yard/validation/y157-e1b93d-durable-20261015-evidence`, including
`checkpoint-identity-comparison.json`, `before-interruption.sqlite`,
`cleanup-4.jsonl`, `cleanup-5.jsonl` and `operator-interruption.json`.
Earlier failed runs remain separate and untouched.

After the host wrote `after-release`, source became `first-write` through a
fresh model invocation. This does not prove replay of the interrupted tool.
The operator retained a consistent store backup, verified no running or
reserved execution and exact private daemon/test identities, then ended
only that fixture daemon and native test. Wrapper exit 101 at 391.74 seconds
wall is an interrupted diagnostic, neither a pass nor an ordinary asserted
failure. No successful continuation, repair or landing is claimed.

`cargo fmt --check` and
`cargo clippy --all-targets --locked -- -D warnings` passed. No e2e command
ran in this candidate box. The affected real-host case and final integrated
host gate remain required. Target 9c34dd56 remains unavailable here; per the
operator's instruction, target preparation uses the host-owned landing
conflict return after the affected case passes. No new access or bundle is
requested.
