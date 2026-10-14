# Y-70 selected panel on policy reapproval

Review1707 found that queue policy refresh counted every agent verdict in
the graph. A losing candidate's reviewer could hide the chosen candidate's
empty panel. Initial approval and policy refresh now share the reading:
ordinary plans count admitted agent verdict nodes; selection plans count
only those bound to the current chosen producer. Required check selection,
current-choice validation and exact operator approval are unchanged.

Disposition: keep the focused G2/G8 host scenario beside the existing
policy-lapse scenarios. It uses two independent real producers and one
reviewer of the losing source. A read-only picker selects the unreviewed
source. Its real candidate check passes, automatic approval starts landing,
and a real host gate holds landing while the operator commits and syncs
manual approval policy. Releasing the gate withdraws automatic approval.
The refreshed item stays unreviewed and names only the chosen source's
candidate check and the same exact choice. No model request or candidate
judgment reruns. Exact operator approval lands the chosen content. The
losing review and original candidate judgment remain unchanged.

The G2 evidence row now names this policy-lapse observation; G8 retains
empty-panel honesty and selected-only verification. Sabotage is restoring
queue::lapsed's graph-wide agent-verdict test. The test comment names it.
Ordinary empty-panel policy scenarios have no unrelated reviewer, so that
sabotage leaves them green. The existing five-profile scenario reviews
every producer; it remains unchanged and cannot expose an empty panel.
The new case exercises the later policy refresh after initial approval,
where the two readings previously diverged.

The observer is the CLI, real Git, host FIFO, scripted host model and store.
Expected source heads and landed bytes come from fixture Git and scripted
producer content. The declared graph supplies the sole required candidate
check and the absence of a chosen-source reviewer. Its check must pass,
while the loser's real review must separately pass on the other Git head.
Check-list agreement and immutable-row comparisons serve the explicit exact
identity and retained-history guarantees. The same-project operator answer
is the allowed control for live manual policy withholding auto landing.

No existing assertion or guarantee was removed. Test lines removed: 0.
Binary lines removed are the duplicate global panel reading and the initial
generator's separate panel calculation; both use one shared function now.
No new lifecycle, copied verdict, policy privilege or test-only binary path.

Validation: cargo fmt --check, cargo clippy --all-targets --locked --
-D warnings and cargo build --locked passed. Native
e2e was not run in this box, as instructed; operator host qualification
remains pending. No .yard configuration changed.
