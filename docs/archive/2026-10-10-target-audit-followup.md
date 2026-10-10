# Y-56 operator audit follow-up — 2026-10-10

The operator ran cafc88f's 40 real host scenarios in 170.40 seconds.
A focused host CLI/worker fixture then found the missing G15 parent edges:
main's plan workflow was writable, feature/a's was read-only, and accepting
the feature planner's children produced [] instead of [3,4]. Those results
describe the previous candidate, not this follow-up.

## Assertion gate

The existing G15 scenario now imports feature/a with read-only plan and
writable write workflows. Main has the reverse access settings. Its
original [3,4] parent-edge assertion and all existing controls remain.
The same writable control worker also proposes a parked child. Acceptance
leaves its existing [1] dependency unchanged and the child's target omitted.

These blocks show G15's target-config parent classification and omitted
child target contract. The observers are host store dependency rows and CLI
ticket JSON. Expected edges come from the fixture's ticket order and its
explicit dependency; the absent override comes from the proposal payload.
The test comment names resolving the parent through project-default config
as sabotage. Copying the parent target into a child breaks the omitted-target
observation. No existing scenario used opposite access for the same workflow
names across targets. Both directions expose the classification error while
retaining the real read-only and writable mount controls.

The follow-up diff removes 9 test lines beside 1 binary line. No assertion
is weakened. Existing G2/G3/G9/G10/G13 controls are unchanged. The binding
spec records the implemented acceptance and migration behavior.

## Migration evidence

This finding was static, not a reproduced runtime migration. Before target
binding, a live stopped:limit item recorded base, head, proof, revision and
verification digests, but no branch. Reconciliation backfilled its attempt's
branch while stale_part compared the absent payload target against it.
The limit item blocks advance and cannot refresh itself; approve's unchanged
stale check therefore rejected the existing override exit.

Reconciliation now performs both writes in its existing transaction before
serving. It first binds unknown live attempts to the former single-target
HEAD. It then reads each live attempt's known target and adds it only to open
approval or stopped:limit payloads whose target field is absent. This also
repairs an old item whose attempt was already backfilled on a previous
restart. A second restart finds that field present and leaves it alone.
Ended attempts, resolved items, proposal payloads and existing target fields
are untouched. No other candidate identity field changes. Approval still
compares branch, head, proof, revision and digests, and records the same
blocking checks and override flag. No alternate recovery path is added.

This is source inspection evidence. No old-version daemon or store was
constructed or modified in this box, and no construction state was opened.
A host upgrade from a pre-target stopped:limit candidate still needs runtime
verification; this document does not claim it passed.

## Validation

The required fmt and locked all-targets clippy checks passed. The e2e suite
and runtime sabotage trials were not run, as instructed. G15's strengthened
real-host fixture remains for the post-approval landing gate. No public model
endpoint, sleeps, mocks, test-only binary barriers or .yard edits were added.
