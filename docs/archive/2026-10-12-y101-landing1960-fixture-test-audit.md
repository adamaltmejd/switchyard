# Y-101 landing 1960 fixture transition repair and test-audit

## Actual failed host evidence

The operator's landing 1960 tested merged head
de8b6883ef7e56d50ac419a6573de4f7d21e87aa. Host gate 1964 finished FAILED on
2026-10-12: 74 passed, 1 failed, in 505.20 seconds. The bounds/native
publication package scenario passed. The missing/corrected/integrity package
scenario failed before its changed-sealed-controller observations. This
result remains failed. It is not evidence that the new integrity block ran
or passed.

The malformed-input admission block left Y-4 and Y-5 open, naming linked
and oversized workflows. The later launch_with_source reconfiguration
removed those workflow definitions. Sync correctly refused removing
workflows that open tickets still name. This was a fixture sequencing
defect, not a runtime or sync safeguard defect.

## Scoped repair and assertion gate

After every malformed-ticket and independent-valid cleanup observation,
the fixture now uses ticket abandon with a reason to close Y-4 and Y-5.
At that point both tickets have been observed open and without attempts;
the external store has proved no attempts or executions for them, and the
independent valid audit has published, become eligible and completed cleanup
while they remained ready. Only then does their closure permit the later
fixture configuration to remove the workflows. Parking alone would still
leave open tickets naming them.

| Disposition and clause | Sabotage retained | Distinct hardest case retained | External oracle retained |
|---|---|---|---|
| Keep G8 selected canonical input refusal and independent cleanup assertions unchanged; retire fixture tickets only after those observations. | Propagate the selected-input refusal out of scheduled admission, or reserve work before validation. | Linked and oversized P0 tickets remain ready throughout independent valid admission, publication and cleanup in the same project. | Fixture paths and size, host Git blobs, CLI refusals and ticket state, external store absence, fixture report and execution-bound cleanup history. |
| Keep G8 changed sealed asset refusal and corrected control unchanged; repair the configuration transition that previously prevented reaching them. | Remove the native bridge's admitted asset hash loop. | Image-owned changed runner.mjs with a consistent launcher seal, followed by the unchanged image in the same project. | Fixture asset bytes and path, real boxed seal and execution marker probes, host model request counts, native terminal refusal and external report. |

No assertion block, expected value or Shown by clause changes. No new
scenario is added. Test lines removed: 0. Binary lines removed: 0.
ARCHITECTURE.md already states both retained G8 clauses and needs no change.
The original missing-module control, corrected-image publication,
iteration/bare-answer, secret, frozen selected input, registration, ownership,
terminal-frame and isolation assertions all remain. The stream repair and
the complete implementation are retained.

Only the supported CLI fixture lifecycle changes. No runtime, deadline,
containment, guard, concurrency, retry or assertion meaning changes. No mock,
binary barrier, sleep or timeout masking is added. Earlier archives and
installed .yard remain untouched. Y-100/Y-93 and the dependent supervision
child retain their complete acceptance.

## Validation

Boxed cargo fmt --check, cargo clippy --all-targets --locked -- -D warnings
and git diff --check passed. Host e2e and an unchanged failure retry were
not run by the worker. The operator's next host landing
must verify the repaired transition and all retained acceptance blocks.
