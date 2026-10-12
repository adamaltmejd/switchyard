# Y-103 Review1008 integrity repair

Review1008 found two defects against the unchanged Y-103 contract.
This repair changes neither the contract nor its selected verification.

## Invalid selected-child evidence

Both selected-child rechecks previously returned a generic refusal when
passing evidence was invalid: the retained-child admission check and the
final pre-intent check. The controller could then end as error and raise
operator red, instead of retiring the invalid continuation for fresh work.

Both checks now end the existing landing execution as retired in the same
admission transaction that observes invalid evidence. A local return value
stops the controller before selecting that child or opening mutation intent.
The existing capacity effect still drops its inherited lock and performs
checked ownership release. A retained reservation or unknown owner continues
to block the target through the existing supported recovery path. Once
released, the queue does not resume a retired execution; it prepares and
verifies a fresh input. No child row, prepared identity, approval or prior
result is rewritten. No owner, lifecycle or recovery path was added.

The retirement helper now accepts the caller's store transaction so these
checks do not open a nested transaction. Existing preparation/ref retirement
sites keep their transactions and behavior. Identity, policy, prepared-ref,
capacity, target serialization and CAS checks remain in place. This repairs
the G2/G3 invalid-evidence retirement contract. It does not claim that the
existing host scenarios directly exercise both invalid-child guard branches.

## G8 assertion and earlier audit correction

Restore the original broad assertion in
`a_gate_errors_start_reruns_that_gate`: no check row may name any landing
child. The passing host child produces a merged execution result, not a
candidate check publication. Narrowing the query to the infrastructure-error
child was unjustified.

The earlier `2026-10-12-test-audit.md` stated that no existing control was
removed or weakened. That claim was incorrect: the no-check assertion had
been narrowed. This archive corrects that claim and records its restoration.
The earlier archive remains immutable.

Test-audit gate answers for the restored block:

- Row and clause: G8's landing infrastructure-error control preserves
  truthful results and publications while supported same-approval start
  lands the exact pinned input with retained passing child evidence.
- Sabotage: let `supervise::gate` publish a candidate check for a landing
  child, including the passing host child. The test comment names it.
- Distinct mechanism: this existing assertion excludes check publications
  across every landing child before retry. The error-child result assertions
  alone cannot catch a spurious check from the preceding passing child.
  The existing infrastructure refusal and successful same-project retry
  remain, as do the exact-input, restart and target-advance cases.
- Observer and expected value: the test reads the real store through the
  existing outside observer. The expected empty set is the preserved
  landing-child publication contract, not a value inferred from Yard output.

This is a bounded test-audit gate, not a release sweep. No test scenario or
assertion block was deleted. The G7 closed event list remains repaired.
The changed test file replaces 4 lines; the binary replaces 37 lines,
including transaction plumbing. No guarantee was weakened.

## Validation

`cargo fmt --check` and
`cargo clippy --all-targets --locked -- -D warnings` passed in the worker.
Host e2e and sabotage execution remain at the approved operator landing gate.
Neither was run in the worker. No construction state/config was changed,
no successor daemon was operated, and no runtime workaround, sleep, retry,
timeout or concurrency reduction was introduced. Candidate gates,
publication and actual merged-tree host verification remain required.
