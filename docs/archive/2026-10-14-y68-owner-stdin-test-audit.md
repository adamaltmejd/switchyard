# Y-68 owner fixture stdin audit — 2026-10-14

The native owner case on 76416ae ended before its model hold. Real pinfold
reported no box spec on stdin. The asynchronous shell replaced fd 0 with
null input before the wrapper's `<&0` delegation.

The shared wrapper now saves stdin before backgrounding. It selects a closed
descriptor from the POSIX shell descriptor range, so it cannot overwrite the
inherited execution lock. The real child receives that pipe as stdin and
closes the extra descriptor. The wrapper closes its copy, records the actual
child PID, waits and reaps that child, and propagates its exit status. No spec
or credential bytes enter argv, diagnostics, the store or fixture files.
The existing teardown FIFO handshakes still follow actual child reap.

Both published_owner_scenario variants were read end to end:
a_seat_killed_after_publishing_has_published and
a_standalone_report_retains_its_owner_across_restart_and_checked_stop.
The fixture model signals its first post-publication held request through a
host FIFO. The existing status history observer selects the exact execution
and observes that signal against execution.ended. A setup failure now fails
immediately with its terminal event instead of waiting only for a model
latch. The latch still proves the request is held before the crash.
No watchdog, sleep, retry or binary barrier was added.

## Assertion disposition

| Block | Guarantee and external expected value | Sabotage and distinction |
|---|---|---|
| Held request before execution end | G5/G6 checked ownership and G8 publication. The host model emits the fixture literal request marker after publication replies; status history binds its wait to the exact started report or review execution. | Fail boxed startup before the model request. The old latch-only wait could report only a later timeout. This is setup evidence for the retained owner cases, not another ownership scenario. |
| Retained crash and teardown blocks | G5/G6/G8/G13. All existing host PID, real box, lock inode, reservation, deadline, result, usage, scope, refusal and allowed-control assertions remain. | Existing comments name release on absence, cancellation outside HOST_LOCK, replay and approval promotion. The corrected real delegation lets those distinct held-owner boundaries be reached. |

The standalone variant still uses trusted operator creation and stop, keeps
null attempt identity and zero ticket/attempt/approval rows, and retains raw
publication without eligibility after interruption. Its explicit successors
still exercise capacity waiting, stop, held real down-child recovery and
allowed completion. The attempt-backed variant still publishes once,
refuses its second publication, allows independent ticket progress and
reaches candidate approval only after checked release. Operator clients are
replaced after restart. Retained events use history.find; the initial
request observer starts from history zero. The one-shot signal is consumed
only at the first held request, so later reports cannot block on that FIFO.

No guarantee, assertion block, production code or .yard configuration was
removed or changed. ARCHITECTURE.md's current contract remains unchanged.
The shared image-preparation protection and held real image-child scenario
remain unchanged. This correction changes only one guarantee test file and
adds this audit record. The test diff replaces 2 lines; 0 binary lines were
removed.

Validation: cargo fmt --check, strict locked all-target clippy, locked build
and git diff --check passed. A focused host shell check delivered fixture
stdin with fd 3 already occupied and propagated a real child's exit 37.
These checks are not native qualification. No e2e or sabotage run was made,
as instructed. Run both affected owner cases serially on the repaired head,
then the remaining affected cases and required full native landing gate.
The earlier zero-ticket native pass does not qualify this repaired head.
