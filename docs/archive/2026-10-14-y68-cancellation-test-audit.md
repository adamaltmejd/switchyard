# Y-68 cancellation ownership audit — 2026-10-14

Review found cancellation dropping capacity::effect before spawning box down.
That discarded both the execution lock and HOST_LOCK. A surviving down child
could therefore be absent from restart's ownership proof.

Shared capacity acquisition now returns the existing execution lock. Ordinary
effects still use the same acquisition, reservation and checked release. A
standalone execution cancels capacity waiting separately; once admitted, it
selects work, stop and its original deadline inside HOST_LOCK. Error teardown
also runs there, before dropping the lock and checking release. There is no
new lifecycle, reservation, owner record or recovery predicate. An unreserved
cancelled waiter starts no teardown child.

ARCHITECTURE.md states that cancellation retains the inherited lock through
teardown, and G5's shown-by clause includes the teardown child crash boundary.

## Assertion disposition

The test-audit gate extends
a_standalone_report_retains_its_owner_across_restart_and_checked_stop in the
existing published-owner fixture. Its original crash, waiter stop, normal
supported stop and successful explicit report controls remain. The shared
candidate-review variant keeps its existing assertions.

| Block | Guarantee and external oracle | Sabotage and distinction |
|---|---|---|
| Teardown crash proof | G5/G6 host children inherit the execution lock. Host wrapper delegates real pinfold down, preserves its exit status, waits for FIFO handshakes after wait/reap of the real up/exec children, and holds until released. Before restart, host PID probes prove those children gone and down alive; real pinfold proves box absence; the host lock refuses acquisition; the store retains running/reserved/result-null/unchecked teardown. | Put stop/deadline select outside HOST_LOCK or spawn cleanup after dropping the lock. The previous crash holds an exec owner and cannot catch a down child launched without that lock. This crash removes the original owners before testing teardown ownership. |
| Retained ownership and refusal | G5/G6 checked restart. History must raise ownership for this exact execution; operator stop refuses ownership with its identity and leaves the complete row unchanged. The host lock remains held after restart. | Release solely on box absence or fail to retain teardown ownership. The earlier supported stop and unreserved waiter are allowed controls in the same project. |
| Checked release | G5/G6 immutable scope/publication and truthful recovery. Host release lets the actual down child exit. Store/history show interrupted, released, checked teardown, unchanged scope and raw report, no completed output, and resolution of only the exact ownership item. Host lock acquisition then succeeds; the existing fresh real report completes. | Promote raw publication, lose scope, or resolve ownership before the child exits. Expectations come from fixture actions, host process/lock/box observations, the spec, and before/after equality where immutability is the guarantee. |

The changed scenario was read end to end. Operator clients and history are
replaced after restart; retained terminal/release events use history.find.
All subprocess wrappers delegate real calls and propagate their exit status.
The host FIFOs hold external subprocesses, not a binary test hook. There is no
sleep, retry or timeout-as-pass. No existing assertion block was removed or
weakened. The diff replaces 8 test lines and 19 binary lines.

Candidate checks passed: `cargo fmt --check`,
`cargo clippy --all-targets --locked -- -D warnings`, `cargo build --locked`,
and `git diff --check`. No .yard configuration changed.
Native e2e and sabotage runs remain reserved for the operator host gate, as
instructed. Run the affected standalone and original candidate-review owner
scenarios serially, then the remaining affected Y-68 cases and full host gate.
This file records static audit evidence, not a native test pass.
