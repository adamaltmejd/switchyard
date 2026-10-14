# Y150 frozen policy reference repair

Reviewer1579 P2 identified a gap between direct qualification and reference
matching. Direct qualification compared frozen node policy, while reference
matching compared only semantic consumed definitions. A replacement execution
could therefore reference a result produced under different node policy.

The matcher now compares recorded typed `verification.policy.node` from both
launches. It removes only the `complete` route, as direct qualification does.
Missing policy refuses reuse. Original execution, result and check rows remain
unchanged. No identity is collected from environment values or credentials.

The existing required G2/G3 scenario gains a policy-only `operator_failure`
revision after the image-adoption boundary. Code, base, head, proof, consumed
definition and runtime remain fixed. Fixture-owned counts require exactly one
more invocation of the changed gate and no invocation of the unrelated gate.
The unrelated gate retains its explicit original check reference. MCP/store
observations require a new passing execution without a reference, exclude the
old check from selection, and preserve its original execution/result/check.

This shows G2's changed-policy invalidation clause. Earlier command and runtime
adoption blocks cannot catch it because they change consumed definitions.
Sabotage is omission of the original frozen policy from matching while only
qualifying the replacement execution. The test comment names that sabotage.
Counts and policy come from fixture literals. Candidate identity comparison
proves the policy revision preserves the prior observed candidate. Original
row comparisons prove immutability and the explicit reference relation.

No scenario, deadline, concurrency setting or existing assertion is removed.
The later failed-command totals increase by one for the new required invocation;
all earlier totals and unrelated-consumer totals stay fixed. No guarantee is
weakened and no operator decision is needed for the test-audit change gate.

Required boxed checks run in the worker. Neither full nor targeted host e2e
is run; the operator runs the existing independently runnable scenario before
approval.
