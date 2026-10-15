# Y-72 reference consumer boundary

Review identified an unsupported consumer: writer-graph scheduling and typed
bindings require concrete reader outputs. A reference-only review has none.
After a no-op publication, admitting reuse there can repeatedly launch reviews
without reaching selection. Input equality alone cannot authorize a reference
whose consumer cannot resolve it.

Eligibility and reference lookup now share the admitted consumer boundary.
Bounded cleanup audits resolve report references through their existing audit
reading. Native linear reviewers with no typed downstream consumer resolve
passing checks through the existing current-check reading. Independent-writer
reviews, concurrent readers and other typed suppliers remain fresh. No typed
binding or writer-graph scheduling semantics change.

## Test-audit disposition

Keep and extend the existing G2/G8 five-writer selection fixture. Declare the
scripted model and fixture configuration identities on its reader agent. The
replacement review must still record reuse disabled and have no reference.
Its existing concrete publication, selector inputs and final landing provide
the allowed control. The expected eligibility boundary comes from the binding
architecture; the real execution store supplies its external observation.

Sabotage is named in the test: admit reference reuse for a writer-graph review.
The eligibility assertion then fails on the existing real review, before a
no-op continuation would require extra boxes to demonstrate the replay loop.
This tests the admission boundary directly; a separate repeated no-op writer
scenario is deliberately not added. The existing G2 cleanup case retains the
positive unchanged-input report-reference control and its changed/unknown
input rerun assertions.

No new machine, build, image, box or model turn is added. Existing five-writer
ownership, exact selection and landing assertions remain. The test diff adds
9 lines and removes 1 configuration line; no assertion is removed or weakened.
The binary diff adds 37 lines and removes 1.

Candidate checks are formatting, all-target clippy and the locked build.
Focused and full real-host acceptance remain the operator's landing gate.
No e2e or sabotage execution ran in the candidate box.
