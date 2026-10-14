# Y-70 independent producer base repair

Review1699 found that selecting B advanced the shared attempt base and
silently supplied that base to another retained producer's continuation.
Publication also used the shared base when the new target was not an
ancestor. The repair keeps the producer's immutable prior candidate base,
or its own execution base before any publication. Publication advances
only after Git proves target ancestry. Selection still supplies the base
for chosen downstream judgments. Recreating a missing writer clone uses
its recorded head or base.

The existing five-producer G2/G8 scenario now continues a losing producer
after B repairs a real target conflict. The operator target includes a
committed configuration comment, so comparing that independent producer
against B's base would falsely classify the operator's change as a worker
change. The producer first commits its own .yard file and is refused with
the exact execution and stopped:failed identity. It then removes that file
and publishes a clean candidate at its original base. Its same-profile
review binds that new publication; a new choice still selects B. Ended
history, including the refused execution, remains unchanged. Only B
reaches downstream gates, exact approval and landing.

Gate disposition: keep the extension. It shows G2's exact candidate and
choice binding, G8's selected-only checks/landing, and G9's independent
workspace continuation under the candidate-base contract. The G2 evidence
row includes the new observation. Sabotage is recording the shared selected
base in the independent execution or using it as the publication fallback.
Earlier controls continued B itself, so the two bases agreed and could not
expose this defect. Here B has actually merged a target that the losing
producer has never fetched. Expected bases and heads come from fixture Git
history and merge-base, producer identity from the configured host script,
and forbidden bytes from that script. The allowed continuation in the same
project supplies the refusal control. Historical rows are compared only
for the explicit immutability guarantee.

No assertion or guarantee was removed. Existing five-profile concurrency,
same-reviewer, picker failure/retry, stale/foreign selection, gate repair,
blocking review limit, target-fetch repair, omission acceptance and open
landing-intent controls remain. Test lines removed: 0 assertion blocks.
Binary removal is confined to replacing the shared-base expressions.

Validation: fmt, locked all-target clippy with warnings denied, and locked
build are required before publication. No native or e2e test was run in
the box; host qualification remains the operator's landing gate.
