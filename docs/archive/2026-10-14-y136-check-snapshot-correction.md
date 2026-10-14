# Y-136 check snapshot correction

Review1486 found that the unsupported-effort refusal snapshot queried
`check.attempt`. The current migrated schema removes that column. A check
binds to its attempt through its execution. Select checks whose execution
is in the actual attempt's execution IDs. Keep all seven table snapshots,
their full rows and ordering, and the unchanged-row assertion.

Restore G8's declared-file naming clause in the guarantee row. The current
runtime contract already requires that diagnostic. Do not restore the
prohibited assertion of diagnostic prose.

## Test-audit Gate

1. G2/G8's unsupported-effort refusal clause requires exact requested and
   recorded values and no revision, question, owner, approval, check or audit
   write. This query repairs the check part of the existing no-write block.
2. Sabotage: remove the adoption capability guard while retaining the launch
   guard, as the existing test comment names. The exact refusal and row
   comparison must fail. A check inserted or changed by a refused adoption
   must also fail the retained check snapshot comparison.
3. The other six table snapshots cannot observe a check-only write. This
   is the existing hardest boundary: an actual old six-field controller's
   successor holds a real model request during committed effort adoption
   after upgrade. No scenario or assertion is added or removed.
4. The store file is the outside observer. Its schema supplies the execution
   foreign key. The CLI supplies the actual attempt identity; the query uses
   it to establish binding agreement. The full pre-decision rows supply the
   expected unchanged values. Committed max/omitted fixture settings supply
   the refusal and same-project successful publication expectations.

Disposition: repair the check predicate only. Keep original admission,
historical execution, expected/current refusal, same-project omission
publication, metadata/fallback controls, early-terminal observations and
all three attempt-bound cleanup waits. Two test lines are replaced; zero
assertions and zero binary lines are removed. No promise is weakened.

Earlier dated evidence remains unchanged. Native gate2927's 78/79 result
and its 1031.46-second failed run do not qualify the later controls or
historical build cost. The runtime budget requires a complete native run.
The two ABI fault/restoration trials and separate capability controls
remain host obligations after approval. No host e2e is run in this box.

Validation: cargo fmt --check, cargo clippy --all-targets --locked --
-D warnings, and cargo build --locked.
