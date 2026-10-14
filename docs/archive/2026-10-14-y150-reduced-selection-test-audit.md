# Y150 reduced selection repair

Keep all three required host failures. The first two image-count failures
and their observer diagnosis remain in the earlier archive files. The third,
host6ca7f8, failed 0/1 at g02:400 in 64.10 seconds suite time and 78.1459
seconds wall time. Earlier real stdout counts, image adoption and policy-only
controls passed. The combined failure/skip revision was refused as
unsupported_revision, shape bounded-repeat.

The fixture removed candidate-gate-unknown and the complete route from
candidate-gate-other but retained unreachable landing nodes. Ordinary
selection requires every retained node on its linear complete chain. This
was an unsupported graph, not a reason to change runtime validation.

The same scenario now removes candidate-gate-unknown, landing-gate-closed
and landing-gate-unknown. Candidate-gate-other completes to the retained
landing-gate-other, whose dangling complete successor is removed. The
selected landing gate uses the successful other command. The failed closed
candidate check still needs its own exact acceptance after each revision.
That acceptance cannot authorize the same failing command at landing.

Exact approval accepts the named candidate omission and requests observation
of the two named landing omissions. The existing merged omission API then
accepts those nodes against the actual prepared merge, original landing and
current candidate identity. Host Git supplies the pinned merged ref and
canonical head. Store observations require exactly the declared selected
landing node, with a passing result on that head; the complete landing-gate
list excludes fabricated executions for omitted nodes. Fixture stdout counts
remain unchanged through approval and gain exactly one other command at
landing: closed 9, other 5, unknown 5. Candidate reference, skipped node and
accepted failed rows retain all prior observations. Approval and the merged
omission decision also survive a quiescent reopen unchanged.

The event trace uses future approval events for each revision. Preparation,
omission attention and parent release use retained-or-future searches bound
to the attempt and preparation execution, with sequence ordering. The answer
waits for checked parent release. Landing completion is a future event after
that answer. Cleanup is retained-or-future, after the recorded landing, and
its checked release precedes reopen. There are no sleeps or retries. The
existing FIFO observer still drains stdout, preserves stderr, and returns
the saved real Pinfold exit status, including exit 7.

Test-audit gate: G2's reference/failure/omission clause and G3's selected
verification clause already name this integrated scenario. The sabotage
comment names dropping reference qualification, carrying candidate
acceptance to landing and manufacturing omitted results. The existing
reduced-selection scenarios establish the API; they lack an immutable
passing reference beside repeated exact failed-candidate decisions. This
second no-change revision, queue and reopen remain the hardest integration
case. Node lists and invocation totals come from fixture literals; merge
identity comes from host Git. Stored row comparisons establish immutability,
not expected verification verdicts derived from Yard logic.

No assertion coverage or binary lines are removed. The unsupported fixture
selection is replaced without weakening a guarantee. Production code, the
observer repair, policy matcher repair and bounded diagnostics are unchanged.
No new scenario, deadline, prerequisite or landing reuse is introduced.
Worker validation is fmt, clippy and build. The operator must rerun the same
required scenario; no worker host e2e result is claimed.
