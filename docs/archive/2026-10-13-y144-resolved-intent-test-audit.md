# Y-144 resolved-intent repair

Review1394 P3 identified an unproved control. The open-intent observation
does not establish refusal after that intent is resolved. The historical
guard was implemented, but the original candidate did not exercise it.

## Test-audit Gate

Keep and extend G3's existing
`a_landing_retires_when_canonical_moves_before_update_ref` scenario.
All previous assertions remain. Its no-write observers now form one local
closure, called at both the open and historical boundaries.
No binary code, lifecycle, grants or source coupling changes.

The initial supported adoption changes a candidate-stage gate definition
that is also selected at landing, and adds the future landing gate. A later
supported change to that candidate-stage command is committed and synced
without adoption. Both source selections keep the ordinary runnable shape.

After the fixture moves canonical and releases the first update-ref, the
real successor merge-tree is held through the daemon's Git wrapper. The
store proves the predecessor's exact ID, resolved intent, retired outcome,
old target and prepared head. Host Git supplies both heads. There is no
open intent or running gate/review; the attempt is live on its candidate,
has no pending next action or open attention, and retains active approval.
The changed definition then refuses naming that historical intent. The
shared observer proves no plan, revision, approval, attention or audit
mutation. Releasing merge-tree preserves the original exact prepared merge,
canonical-move retirement and successful requeue assertions.

| Gate question | Answer |
| --- | --- |
| Guarantee and clause | G3's before-any-intent landing-requirement boundary, now explicitly shown after a real intent resolves and retires |
| Actual independent sabotage | `Y144-historical-landing-intent`: remove only the historical `landing_requirements` comparison and `intent_state.is_some()` refusal block in `attempt_revise`, including its local comparison closure; retain `refuse_during_intent` and every earlier validation |
| Distinct hardest case | The same live attempt has crossed a real durable intent, then requeues after a real canonical CAS failure. The open guard sees no intent. Gate/review activity, attention and pending actions cannot supply another refusal. Existing open-intent assertions cannot detect removal of the historical guard |
| Outside expected values | Fixture commands and before-first-intent success establish support; host Git supplies target and pinned merged heads; FIFO-held Git and store rows prove the exact boundary; pre-call store snapshots supply the unchanged-row oracle |

Deleting this guard must fail the immediate structured refused-code and
historical-intent identity observation. An allowed revision result fails
there, before any deadline or later landing observation. A timeout or an
`active_plan` refusal does not qualify the sabotage. No guard was sabotaged
or host e2e executed in this candidate box; this remains an independent
post-approval runtime gate.

Static Gate review accepts this extension. No assertion or coverage clause
was removed or weakened. The diff removes 42 test lines through observer
relocation and fixture edits; assertion blocks removed: 0; binary lines
removed: 0. No operator weakening decision is
requested. This is a bounded Gate review, not a suite sweep.

The contract clarification names candidate-stage definitions selected at
landing and resolved intents. Definitions outside selected landing
requirements retain their existing adjustment behavior.

## Carry-in qualification

The operator reports Y146's four direct faults and restored control qualified
on `c702930`, with evidence in
`docs/archive/2026-10-13-y143-y146-native-qualification.md`, imported to target
`d529214`. The qualified G9 controls and earlier evidence remain untouched.
This repair neither replaces that qualification nor claims host qualification
for the new historical-intent observation.

All original Y144 acceptance and Y-140 parent integration obligations remain.
Packaged activation remains mandatory in its separate dependent child.
