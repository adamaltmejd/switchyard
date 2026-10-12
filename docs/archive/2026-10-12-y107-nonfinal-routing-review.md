# Y-107 non-final profile and routing repair — 2026-10-12

## Retained operator evidence

The operator's final-seat profile control passed on `b64d940`. The operator
also reports a direct digest sabotage failure. That sabotage result is
retained here as reported evidence; its exact patch and log are not mounted
in this workspace. No host test or sabotage was run by this repair worker.

The first failed profile probe used `fa653455`. Changing only the fixture's
declared final-profile instructions to `Alternate final review.` refused
the first revision with `unsupported_revision` at G2 line 89. Its reported
host evidence is:

- `/home/yard/validation/y107-fa6534-review-20261012/operator-review-evidence.json`
- `/home/yard/validation/y107-fa6534-review-20261012/operator-admitted-profile-probe.log`

The second failed probe used `b64d940`. Changing only the first revision
target from `review-1` to `review-0`, with the same admitted read-only
complete/blocked profile and unchanged production and assertions, refused
with `unsupported_revision` at G2 line 100. This reproduced review check
1143 P2. The temporary source was restored and the test executable rebuilt.
Its reported host evidence is:

- `/home/yard/validation/y107-b64d94-review-20261012/operator-nonfinal-profile-probe.log`
- `/home/yard/validation/y107-b64d94-review-20261012/operator-nonfinal-profile-probe-result.json`

These host files are not mounted here. Both probe records and the prior
passing control remain operator evidence, not claimed worker reproductions.
Earlier archives stay unchanged.

## Guard repair

A declared reviewer profile describes complete/blocked capabilities. The
ordinary seat position supplies runtime outcomes and bounded routes.
Validation previously compared the profile's blocked capability with a
non-final seat's complete-only outcomes and refused it. The repaired guard
normalizes compatible declared profile outcomes to the retained ordinary
seat before comparison. It still compares access, outputs, typed inputs,
required inputs and routes with the ordinary shape. It does not normalize
an arbitrary undeclared outcome or grant an extra repair edge.

The selected runner still launches with its source-bound agent settings and
instructions. Profile selection changes neither the review grant nor the
execution lifecycle. Original admission, implementation, landing policy,
target, ownership containment and the quiescent CAS requirements stay frozen.

## Test-audit Gate and P3 observation

Keep and strengthen the existing G2/G5/G9 scenario in the same project.
The fixture now declares three ordered seats, allowing a middle omission
to change an actual supplier without making the retained first seat final.
The original final-profile control, held live refusal, stale second edit,
append-only history, original rows, quiescent reopen, stale approval and
corrected landing remain. The refused and corrected non-final edits name
the same profile and policy.

Revision 2 selects the alternate final profile and retains the unrelated
completed reviews. Revision 3 selects that same profile for the non-final
first seat. The host model observes its alternate instructions. Its digest
and selected check change; the middle seat binds the actual new first-seat
execution, and the final seat binds the actual new middle-seat execution.
The final runner digest stays unchanged despite its new typed supplier.

Revision 4 supplies a literal fixture graph omitting the middle seat.
The completed first seat's complete successor changes from the middle to
the final seat. Its unchanged evidence remains selected without a new
execution. The final seat loses its old check and takes a new one bound to
the actual first-seat execution instead of the omitted middle execution.
All previous execution/input/result rows remain unchanged. This directly
observes P3's existing routing-only qualification contract.

Expected instructions, routes, outcomes, revisions and execution sequence
come from fixture literals. Host Git supplies the candidate identity;
store rows supply externally observed executions and checks for identity
agreement and immutable-history comparisons. The test comment names old
seat startup, retained ordinary-seat digest, CAS removal, admission
overwrite, revision-only qualification, missing node/policy identity,
complete-route qualification and obsolete supplier selection as sabotage.
There are no sleeps, retries, public endpoints or binary test barriers.

The read-only audit read G2 and G6 whole. No existing block covers a
quiescent non-final profile change or retention across a changed complete
successor with a replacement actual panel supplier. No twin or invalid
assertion block was found. The binding contract and G2 Shown by clause
state both observations. No guarantee or assertion was weakened.
Test lines replaced: 22. Binary lines removed: 0.

`cargo fmt --check` and
`cargo clippy --all-targets --locked -- -D warnings` passed. Host e2e
remains the post-approval landing gate and was not run here.
