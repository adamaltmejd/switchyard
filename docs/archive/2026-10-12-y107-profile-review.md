# Y-107 admitted profile correction — 2026-10-12

## Operator probe

The operator reported a host probe of candidate
`fa653455`. The original G2 plan-CAS scenario passed. Changing only the
fixture's declared `[profiles.final]` instructions from `Final review.` to
`Alternate final review.` made its first revision call fail at G2 line 89
with `unsupported_revision`. Production code and assertions were unchanged
for that probe. The operator restored temporary changes and rebuilt the
production and test executables successfully.

The reported evidence paths are:

- `/home/yard/validation/y107-fa6534-review-20261012/operator-review-evidence.json`
- `/home/yard/validation/y107-fa6534-review-20261012/operator-admitted-profile-probe.log`

Those host files are not mounted in this workspace. This record preserves
the operator's reported failure, not a claim that the worker reran it.
The earlier sizing and test-audit archives remain unchanged.

## Implementation trace

Ordinary selection compared every reviewer runner with the ordinary seat.
It now permits a runner matching an already declared read-only agent
profile with verdict output and ordinary complete/blocked outcomes. Node
names, panel order, candidate and landing phases, access, inputs, outputs,
required inputs and bounded routes retain the ordinary shape checks.
Quiescent adjustment still freezes implementation, landing policy and target.
The source remains the admitted commit, not the current template.

Review startup also used the ordinary seat's settings and instructions.
It now consumes the admitted selected node runner. Execution intent records
the selected agent's settings; the same settings choose its harness, model
route and launch arguments. New revision evidence records those settings
from the admitted source. Original admission and historical rows stay intact.

The frozen panel digest stays the baseline. A different reviewer runner
extends only its check digest with its selected runner and source-bound
agent settings. Execution recording, current-check qualification and exact
decision evidence share this reading. Panel binding uses each producer's
own selected digest. Node policy and actual typed inputs still qualify
retention; changing a producer invalidates its consumers. Candidate answers
recompute from the frozen baseline before comparing exact selected evidence.

## Test-audit Gate

Keep the existing G2 scenario and strengthen its profile control. G2's
Shown by clause now includes actually selected instructions and affected
seat digest changes. The host model receives fixture instructions: the first
final reviewer gets `Final review.`, later final reviewers get
`Alternate final review.`. The affected execution digest differs, while
the unrelated completed check stays selected. The later producer policy
edit still forces a new consumer with the actual new supplier identity.

Expected instructions come from the fixture, rather than Yard output.
Digest inequality establishes the specified selected-policy distinction.
Existing store snapshots establish immutable history and identity agreement.
The test comment adds old-seat startup and retaining the ordinary-seat
digest for a different selected runner as sabotage. Existing G2 blocks have neither mechanism;
this strengthens the same hardest-case scenario instead of adding a twin.
The read-only audit found no invalid assertion blocks or new unshown clauses.
No assertion or guarantee was weakened. Test lines removed: 3, replaced by
the stronger profile control. Binary lines removed: 20, replaced by selected
runner startup, profile qualification and the shared check digest.

`cargo fmt --check` and
`cargo clippy --all-targets --locked -- -D warnings` passed. The host suite
and sabotage runs were not executed here, as directed by the ticket.
Host e2e remains the post-approval landing gate.
