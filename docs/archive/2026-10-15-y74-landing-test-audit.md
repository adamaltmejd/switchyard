# Y-74 landing assertion repair

The operator's landing gate for candidate
`668eba9c010970a7f3f9914f94d4dca0cf0e5ef3`, targeting
`785c443732bb6608dbcde692c8e988f307a5c436`, reported 50 passed and one failed
in 627.98 seconds. G10's feature-target case failed at the complete frozen
admission-input comparison. Actual provenance included the two test-audit
files now installed by init. The literal fixture expectation omitted them.

Inspection of init, trusted staging and the whole G10 case confirms the
capture is correct. Add both expected sources in their existing sorted
position, with blob identities read from the fixture's frozen Git commit
and non-executable modes. Preserve the complete equality assertion, later
repair/restart identity checks and all product controls. No binary behavior
changes. The spec clarifies that init installs the sweep reference too.

## Assertion gate

- Guarantee and impact: G6/G2 admission provenance must retain every trusted
  staged input across target conflict repair and restart. Missing or changed
  trusted bytes would undermine the frozen identity used by workers.
- Sabotage: omit either test-audit source from admission capture. The complete
  expected-input comparison fails. The test comment names this sabotage.
- Existing hard case: adapt the retained feature-target case. Add no fixture,
  build, box, model turn or execution phase. Its allowed feature admission,
  repair and landing controls remain intact.
- External observer and expected values: operator CLI admission JSON is
  compared with the complete fixture plan's committed sources. Host Git
  supplies blob identities at the frozen feature commit. Expected modes
  follow the non-executable init assets, alongside the fixture's executable
  admission skill control. Expected sources do not come from Yard output.

Disposition: retain the assertion and update its complete fixture expectation.
One test comment line is replaced; no assertion or binary line is removed.
No new coverage or deliberate coverage reduction is introduced.

Local fmt, locked all-targets clippy, build and diff whitespace checks passed. The host
e2e suite is not run in this container; the operator owns the landing rerun.
No project `.yard`, installed configuration, runtime or model settings change.
