# Y-151 G1 native gate correction

The operator's native gate on fd7afef reported 83 passing scenarios and two
failures. The G1 reduced-selection scenario expected no passing landing
checks. Y-151 records those real judgments as immutable reference sources.

Test-audit disposition: rewrite the expected sequence in
`operator_reduced_verification_stays_skipped_and_queue_authorized`.
The full control includes its two actual landing passes. Reduced selection
includes only its selected final landing pass. Empty selection still has
no checks. No assertion block or guarantee clause is deleted or weakened.

- Guarantee: G1 selected verification and explicit omissions. G3's actual
  merged-input observations remain unchanged.
- Sabotage: execute omitted gates or fabricate their passes. The exact
  execution and check sequences still fail, as the scenario comment names.
- Distinct mechanism: the same-project full, reduced and empty controls
  expose omitted nodes and the retained panel predecessor. No scenario is
  added and no existing coverage is duplicated.
- Expected values: fixture-selected nodes and scripted pass/fail outcomes.
  Observers remain the CLI, read-only store and host Git.

Diff removals: 1 test comment line beside 0 binary lines. No code changed
in the daemon. No architecture contract changed.

Validation passed: `cargo fmt --check`,
`cargo clippy --all-targets --locked -- -D warnings`,
`cargo build --locked` and `git diff --check`.
The worker did not rerun host e2e; that remains the operator's landing gate.
