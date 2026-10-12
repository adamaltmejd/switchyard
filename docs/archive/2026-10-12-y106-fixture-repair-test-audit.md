# Y-106 fixture and observer repair

The operator ran `cargo test -p e2e --locked
selected_skipped_decisions_bind_identity_without_manufacturing_results`
on exact candidate `fbdf5fe93cf67469028c5406f03f093328f031e3` on the real
host. Cargo exited 101. The scenario took 66.05 seconds and failed at
`g02_judgments_bind_identity.rs:945`. Selected admission returned `invalid`,
reason `unresolved_reference`, input
`nodes.landing-gate-keep.inputs.merged.output`.

The reduced graph referenced external `merged` without declaring it.
The repair declares `merged` as `candidate`, the supported landing input.
The empty graph retains only its used external `feedback` declaration.
Production validation stays unchanged.

The failed run reached the full default landing and planner cleanup. It
did not reach selected candidate omission acceptance or any new merged
boundary clause: exact pinned-input acceptance, predecessor lock refusal
and release, held host child and target advance across restart, delayed
stale answer with no writes, preserved old decision, fresh acceptance,
retained fresh gate, ordinary reduced/empty controls, or explicit empty
decision before mutation and persistence through landing/reopening.
Those clauses remain required and unverified by that run.

`Watch::find` searches cached events first. The former generic landing
attention predicates could return the original question for a successor
or another attempt. Each question now binds its exact attempt and the
execution from `landing.prepared`. A successor preparation binds the
externally advanced target and follows the old question's sequence; its
question also requires a distinct attention and later sequence. The empty
question binds its own preparation and follows the preceding question.
Cached-event recovery remains available.

The same observer pass found that the ordinary/empty loop read an attempt
ID from approval event data, which contains only kind, reason and payload.
It now reads that ID through `attempt show`. This fixes the identity source
at the user seam without changing a binary or shared watch helper.

Test-audit disposition: repair the fixture and observers in the existing
G2 control. Keep every assertion block and positive control. G1's queue
controls, G2's exact identity and persistence, G3's checked ownership and
pre-intent empty decision, and G8's absence of omitted judgments remain
the promised clauses. Expected heads come from host Git, node/input names
from the fixture, and ownership from the external lock. Persistence checks
compare snapshots. The existing sabotage comments and held crash point
remain intact. No twin scenario was added. Y-105's failed-result and
infrastructure-error controls remain unchanged.

Five test lines were replaced; zero assertion blocks or binary lines were
removed. No deadline, concurrency, runtime, spec or production guard changed.
Worker validation is fmt and clippy only. The repaired control and meaningful
sabotage require the operator's exact-head host rerun. The complete Y-106
and Y-104/Y-96 integration gates remain required.
