# Y-157 filter keys and fixture composition — 2026-10-15

Review3702/check1931 on 4a6f711 found that valid UTF-8 filter subsection
names can contain `=`. Git splits a `-c key=value` argument at its first
equals sign, so the constructed override could leave the original filter
enabled. Both `commit_source` and `packaged_status` now refuse discovered
keys containing `=` before add/commit or status respectively. Existing
output bounds, decoding checks, environment and helper restrictions remain.
No generic Git configuration mechanism or new runtime path is added.
ARCHITECTURE.md records the implemented refusal.

The earlier decoding review was review3697/check1927, not check1926.
The citation in the earlier dated record is corrected as requested;
check1926 was the build.

## Test-audit gate

The operator's affected native 4a6f711 run failed at the new predecessor
guard: ordinary implementer execution 3 ended with exit 1 and no terminal
frame before cleanup. It took 91.22 seconds in the suite, 94.57 seconds wall.
The prior selected-input explanation was incomplete. Ordinary roles load
`AgentEnv::load`, and Pi explicitly receives trusted base extensions even
when selected skills are present. Selected skills do not suppress them.
The fixture's hostile `unselected.js` writes a sentinel without exporting
a valid Pi extension. This fixture must not feed it to the ordinary role.

Keep that committed extension through the first packaged read-only audit
and its cleanup release. After observing release and the unchanged canonical
`base` source, remove only that extension with an ordinary fixture Git commit
and sync before admitting the handoff. Its original commit remains available
for historical audit inputs. Keep the explicit role skills and the history
guard. No production admission, staging or ordinary-agent contract changes.

This corrects composition of the retained G9/G13 hard case. All existing
audit refusal, legitimate declared writes/commits, Git positive controls,
source-state, owner/release, interruption, continuation, independent grants,
landing and nested Git assertions remain. Their expected values still come
from fixture bytes, native Git, host files and exact execution history.
The guard's comment names failure of the ordinary predecessor as sabotage;
the existing comments retain the product sabotages. No assertion is removed
or weakened. The change adds one fixture commit and sync, no machine, image,
build, worker box or model turn. Reusing the existing case is sufficient.
The equals-sign filter variant remains deliberately without a new automated
case; no exhaustive filter matrix is added.

Test diff: 6 lines added, 1 replaced. Production diff: 8 lines added,
1 replaced. This is a test-change gate, not a suite sweep.

## Validation and retained evidence

`cargo fmt --check` and
`cargo clippy --all-targets --locked -- -D warnings` passed. No e2e command ran
in this candidate box. Complete affected-case and integrated real-host
validation remain required before approval. Unsafe write, restart,
continuation and landing have still not been reached by the reported runs.

The three failed evidence directories remain separate and unchanged:

- `/home/yard/validation/y157-3af1e6-durable-20261015-evidence`
- `/home/yard/validation/y157-5e19c0-durable-20261015-evidence`
- `/home/yard/validation/y157-4a6f71-durable-20261015-evidence`

This record uses the operator's host observations; it claims no new host pass.
