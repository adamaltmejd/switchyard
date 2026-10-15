# Command classifier fixture JSON tool repair

The operator's exact ec241fb command case, integrated only with canonical
observers 878b77e, failed. Pass-through diagnostics for real execution 27
captured `sh: 1: node: not found` and exit 127. The complete failed real run
(child 101, restored source, unchanged runtime) took 53.79 seconds. Evidence:
`/home/yard/validation/y73-command-error-capture-20261015/{stderr,exit-code}`
and `/home/yard/validation/y73-command-error-diagnostic-20261015-evidence`.
The earlier uninstrumented failure remains at
`/home/yard/validation/y73-637be9-integrated-command-20261015-evidence`.
These failures are not passes or timeout evidence.

## Test-audit disposition

Repair the existing G2/G8
`candidate_reuses_original_pass_after_bookkeeping_with_unknown_rerun` in
`g02_judgments_bind_identity.rs`. Its command box has no agent harness and
cannot assume the harness supplies Node. Explicitly install jq through the
fixture's existing Debian image-revision Dockerfile step, before that image
is adopted. The classifier uses jq to parse its real read-only mounted JSON
input and publish `.identity` unchanged alongside the declared retry outcome
and fixture report. No identity is hardcoded or reconstructed.

All assertions remain unchanged, including exact evidence, one retry,
durable exhaustion across edit, truthful failed verdicts and authorized
acceptance. Their existing sabotage comments name false pass reuse, stale
acceptance and silently repeating or resetting the bound. Expected identities
come from real input, Git and externally read execution/check rows; the
configured repeat bound supplies the expected count. The same representative
case remains sufficient. No test, machine, build phase, box or model turn is
added. The existing image rebuild gains one explicit JSON tool dependency;
no production dependency changes. Test lines: two removed, five added.
Binary lines added or removed: zero. No Shown by or product control changes.

Candidate fmt, Clippy (all targets, locked, warnings denied), locked build and
diff whitespace checks pass. No host e2e is run here. Production e3b7598's
stack repairs, both phase cuts and ec241fb's independently qualified edited
worker hold remain intact. Host qualification and current candidate review
remain operator gates. No installed binary, runtime, daemon, model, reviewer,
observer, policy file or repository .yard configuration changes here.
