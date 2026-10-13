# Y144 implementation digest correction

Operator native gate2780 on `b063511` passed 78 of 79 scenarios. G2 failed
at line 682 comparing an implementation execution's null digest against its
predecessor's null digest. Implementation startup leaves the optional
gate/review check digest unset. This is a second unsupported metadata oracle,
distinct from gate2770's runtime image-ID comparison.

Both failed full raw logs remain preserved on the host under
`/home/yard/validation/y111-operator-evidence-20261013/` in
`approved-y144-gate-capture` and
`approved-y144-observer-repaired-gate-capture`. Earlier archives remain
unchanged. Neither failed candidate is qualified by diagnostic observations.

## Test-audit Gate

Apply the operator's explicit deletion decision to the existing G2 scenario.
Remove only the condition extending the review-digest assertion to the
implementation row at image adoption. Keep the original independent
review-digest assertions and unchanged host-gate digest controls. No binary
change or new observer is needed.

| Gate question | Answer |
| --- | --- |
| Guarantee and clause | G2's independent image adoption with fixed definition/staging selections, real image consumers, unchanged host commands and preserved historical evidence |
| Named sabotage retained | Build current HEAD rather than the selected image context, reuse the prior consumer image, or ignore staged inputs for a completed implementer; the existing scenario comments name these faults |
| Existing coverage and hardest case | The unsupported metadata condition proves no additional clause. Existing direct consumption controls exercise completed consumers followed by independent adoption with newer poisoned definitions/staging and a later unselected image context. Adding another observer would duplicate that proof |
| External expected values | Host Git supplies selected commits, Dockerfile blob and context tree; fixture ENV/COPY bytes gate the productive implementation feature and both reviewers' publication; fixture commands and prior store snapshots supply host-command and retained-history oracles |

Retain exact selected source identities, typed supplier bindings, real
implementation ENV/COPY-gated productive output, both reviews' direct COPY
observations before publication, and all historical snapshots. G3's real
resolved-intent identity refusal and no-write controls remain unchanged.
No runtime IDs or implementation check digests are synthesized. No spec,
grants, lifecycle, ownership controls or source coupling change.

Static Gate accepts deletion of this unsupported condition. Test lines
removed: 3; test lines added: 1; binary lines removed: 0. No guarantee or
Shown by clause is removed. This is a bounded correction, not a suite sweep.
The original native suite and independent named runtime faults, including
`Y144-historical-landing-intent`, remain mandatory post-approval host gates.
They were not run in this box. All original Y144 and parent/packaged
obligations remain. Imported Y143/Y146 qualification and earlier failures
remain unchanged.
