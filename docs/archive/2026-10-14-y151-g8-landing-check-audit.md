# Y-151 G8 native gate correction

The same native gate reported a failure in
`a_gate_errors_start_reruns_that_gate`. Its absence query included every
landing child, including the host child that had already passed.

Test-audit disposition: rewrite that query's expected result to the one
real preceding passing child's check. An infrastructure-error child still
must produce no check. Its null judgment, null route, empty outputs,
unchanged canonical, retained approval and real same-input retry control
remain asserted. The sabotage comment now names publishing an error check.

- Guarantee: G8 infrastructure errors are not judgments and cannot be
  accepted as failed verification. The allowed passing child supplies the
  same-project positive control for check publication.
- Sabotage: publish a check on the refused box-up child. The exact selected
  check set gains an extra row and fails.
- Distinct mechanism: this is the existing selected merged-infrastructure
  error and requeue scenario. No additional scenario or duplicate block.
- Expected values: the fixture permits the first host gate and refuses the
  following boxed gate. Its externally observed execution identifies the
  sole expected passing check; no expected verdict comes from Yard logic.

Diff removals: 6 test lines beside 0 binary lines. No guarantee clause or
scenario is removed. The daemon and architecture contract are unchanged.

Validation passed: `cargo fmt --check`,
`cargo clippy --all-targets --locked -- -D warnings`,
`cargo build --locked` and `git diff --check`.
Host e2e remains operator-run; the worker did not rerun it.
