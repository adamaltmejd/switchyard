# Y-154 fixture count repair

The focused host run of `only_the_declared_choice_is_checked_and_landed`
failed at the initial raw finding count: ten rows were present, but the
assertion still expected five. The new provenance assertions passed.
The operator supplied this result from
`/home/yard/validation/y154-987252-picker-20261015-evidence/native.log`
and `terminal.json`.

The test-audit Gate keeps the G2/G8 assertion and corrects its external
fixture expectation to ten: five initial reviews each publish two findings.
The sabotage remains dropping or conflating identical findings. This repairs
an existing assertion; it adds no scenario or observation. All source
identity and historical evidence comparisons remain intact.

Inspection of the scenario's other counts found no further stale finding
counts. Per-review source and finding counts already expect two. Candidate,
review execution, check, approval, repair and landing counts are unaffected.
Later raw findings are compared by durable row identity against the complete
initial snapshot, with no fixed later total.

Product code and scope are unchanged. The host reruns the affected cases;
the worker does not run e2e.

Validation passed: `cargo fmt --check`,
`cargo clippy --all-targets --locked -- -D warnings`, and `cargo build --locked`.
The diff replaces one test line and removes no binary lines.
