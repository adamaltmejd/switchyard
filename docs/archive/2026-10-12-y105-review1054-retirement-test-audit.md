# Y-105 review1054 retirement identity audit

## Disposition and guarantee

Repair the existing retirement assertion in g03_landing_cas.rs. Selecting a
landing by outcome retired and then asserting retired did not identify the
intended continuation. No scenario, product code or guarantee was removed.

The exact G3 shown-by clause is: "An exact accepted merged failure survives
restart with a held remaining host child, retaining ownership proof and its
pinned host Git head; all remaining selected gates run before the shared
final CAS."

The target-advance branch also supplies G2's clause: "With a remaining host
child held, target advance retires preparation and does not carry its
acceptance; a delayed answer names exact old expected/current heads and writes
no decision or audit, then fresh acceptance succeeds in the same project."

## Assertion gate

The named sabotage is retiring an unrelated landing instead of the
changed-input continuation. The previous outcome-filtered query could observe
that unrelated row and pass. The repaired block can no longer do so.

Before target advance, the read-only store identifies the real held second
child's parent and confirms that parent's resumed provenance names the
original preparation. After the existing landing-recorded event, the observer
selects the unique landing resumed from that captured parent, without an
outcome predicate. It reads that execution's actual outcome by its exact ID
and expects retired from the contract. The interrupted held parent and its
post-restart successor are distinct executions on the existing lifecycle.
The identity oracle is the externally read child-parent and resumed provenance,
bound before the host changes canonical; it is independent of the outcome
under assertion. Provenance assertions compare recorded identities, while the
expected retirement outcome comes from the spec.

This is the hardest existing case: an accepted merged failure, a completed
passing child and a real held remaining host child, then a canonical advance
and restart. The scenario already proves host process-group liveness, the
held inherited lock, the pinned Git head and absent mutation intent before
restart. Its FIFO holds the child until reconciliation. Fresh merged input,
old stale answer with no writes, new acceptance, unchanged failed/passing
snapshots, ownership release and remaining-gate observers are retained.
Other retirement blocks cover final CAS races, not retirement of this exact
prepared provenance across restart. No twin was added.

Test lines removed: 8. Binary lines removed: 0. No .yard edit, mock, sleep,
retry, binary barrier or new lifecycle was introduced. The shared predecessor
ownership proof and corrected late attempt identity remain unchanged.

## Validation and retained evidence

This is a source audit. cargo fmt --check and cargo clippy --all-targets
--locked -- -D warnings passed. No host e2e or sabotage was run in the worker.
The named sabotage remains for actual operator host validation.

The landing1965/host1969 failed result remains 72 passed, three failed in
922.03 seconds. Only the Pi-extension result explicitly establishes SQLite
database or disk full. The missing Codex terminal frame and empty gate-error
CLI JSON have no established shared cause in that result. The real landing
attention observer repair remains intact. The operator's retained-artifact
storage action and the unresolved review-timeout evidence remain recorded in
the previous dated archives; this source repair does not resolve either
runtime limitation. Full Y-105/Y-104/Y-96/Y-61 integration and exact-head
host/sabotage evidence remain required.
