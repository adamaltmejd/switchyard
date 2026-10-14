# Y-70 independent Git observer correction

Disposition: keep both independent-producer assertion blocks in G2/G8's
five-profile scenario. Their merge-base calls previously asked a retained
clone to resolve the later target object, which it has never fetched.
Both calls now use its known original base. Each block also reads HEAD's
commit history and asserts that the later target is absent. No object is
fetched into the worker, and no product behavior or binding spec changes.

These blocks retain the G2 evidence clause for independent continuation
after B merges an operator configuration change. They also preserve G8's
selected-only continuation and G9's independent workspace base contract.
The existing test comment names the sabotage: bind this producer or its
publication to the selected attempt base. Earlier B-only repair controls
cannot expose that error because B has incorporated the new target. Here
the losing producer has not. Both the refused worker-owned .yard edit and
the allowed clean publication remain, with their exact execution/base and
immutable source/review/choice history assertions unchanged.

Expected values come from the fixture's original Git base and later target
commit. The observer is real Git in the retained clone. The original-base
ancestry check and target-absence check together preserve the intended
observation without requiring an unavailable object. The two checkpoints
observe the refused dirty continuation and its allowed clean successor.
No assertion block or guarantee was removed; two Git arguments changed
and two target-absence assertions were added. Binary lines removed: 0.

Validation: required fmt, locked all-target clippy with warnings denied,
and locked build run before commit. A standalone real Git probe checks
these observations with a detached, no-hardlinks clone whose origin was
removed before the later target commit. Native/e2e remains the operator's
host landing gate and was not run in this box, as instructed.
