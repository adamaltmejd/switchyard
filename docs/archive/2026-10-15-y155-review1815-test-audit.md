# Y-155 Review1815 test-audit Gate

Extend the existing multi-review join and finding publication scenario in
`g04_capacity_holds.rs`. No scenario or schema-shape test is added. Preserve
its G2/G8/G13 identity, raw-source, candidate change and no-authority controls,
the native candidate selector controls and the capacity/restart scenarios.

Gate disposition: keep the added held selection block under G5 and G8. The
same commit extends their Shown by clauses. No guarantee is weakened.

- Clause: a completed registered selection retains reservation and its exact
  ownership attention while its real inherited host owner remains held. Checked
  release resolves only that item without replay or rewriting the publication.
- Sabotage: recognize only pass/fail at checked release, or clear ownership
  before the inherited lock is free. Both are named in the test comment.
- Existing completed judgment coverage uses pass/fail; interrupted publication
  recovery uses a different result path. Neither detects omission of selected
  typed output from successful completion reconciliation. The existing join
  now holds the successful picker owner through restart after real box removal.
- Expectations: the fixture chooses the exact box; the wrapper delegates real
  pinfold down and records its successful exit and the actual live child PID.
  Real box inventory and the same-inode host lock prove disappearance and
  continued ownership. Host store rows preserve the original report/result
  and identify the exact ownership item. History observes checked release and
  that item's reconciled resolution. The same project then reaches approval
  and retains the existing single-selection, raw-source and landing controls.

The daemon is killed only while the child blocks on a fixture FIFO. Before
restart the test proves successful completed finding-selection, registration,
reservation, unchecked teardown, box absence, live PID and held host lock.
After restart the exact committed row and open ownership item remain. Releasing
the FIFO allows lock freedom and the existing checked reconciliation. The old
history follower is replaced after restart. No sleep, retry, clock expiration,
store mutation or binary barrier is introduced. The shell child is portable
POSIX shell and inherits the real lock descriptor with stdio redirected.

Public operator output schema now admits finding-selection. Selector tool
input schema retains its strict candidate/finding union under an object root,
as required by the MCP tool input schema contract:
https://py.sdk.modelcontextprotocol.io/advanced/low-level-server/.
Daemon dispatch, worker grant and authority remain unchanged. No schema-shape
assertions are added; native candidate/finding controls remain the host gate.

Test diff: 129 added, 1 replaced line. Binary diff: 18 added, 3 replaced lines.
There are no deleted assertion blocks or removed guarantees.

Worker validation: cargo fmt --check, cargo clippy --all-targets --locked --
-D warnings, cargo build --locked. Native e2e and sabotage execution are
explicitly deferred by the operator while Y71 fullgate runs; they are the
sequential host landing gate after approval.
