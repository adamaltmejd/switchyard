# Y-102 bound report output correction

Correctness review found that the refused-ownership assertion expected the
fixture payload directly in `result.outputs.result.value`. The report output
contains the whole execution-bound publication. Its nested `report` field
contains the fixture payload. The existing generic audit scenario asserts
this same contract. Correct the selected scenario's expectation to the full
publication captured before the ownership refusal. Production is unchanged.

The scoped test-audit gate retains G8's clause: native publication followed
by real host lock refusal stays reserved and ineligible until checked release
admits the same bound report and permits cleanup without replay. Sabotage
remains counting publication or terminal success before checked ownership,
or starting cleanup while reserved, as the test comment names. The distinct
hardest case remains the selected native terminal frame followed by actual
host ownership refusal despite real box absence.

The fixture still independently supplies the expected payload, asserted at
`published.report.report`. This corrected block establishes agreement between
the result output and the earlier execution-bound publication, and the later
control preserves that result across checked release. Agreement is the
specified oracle for this identity-preservation assertion; no expected
payload is inferred from Yard. No new scenario or duplicate block is added.
No guarantee or positive control is weakened. Binary lines removed: 0.
Test lines removed: 1, replaced by the corrected expected output.

Boxed `cargo fmt --check` and
`cargo clippy --all-targets --locked -- -D warnings` pass. Host e2e was not
run in this box. The ownership refusal and successful checked-release control
still require the operator's landing gate; review evidence is not a runtime
pass.
