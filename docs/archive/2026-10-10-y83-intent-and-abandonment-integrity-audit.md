# Y-83 intent and abandonment integrity audit, 2026-10-10

Exact source reviewed: 9b013b3. The operator's finished-review handoff
names review1174/check621 and review1185/check625. Its findings are source
evidence, not reported host reproductions. Construction state was not
opened. The available worker context contains no separate review artifact.

## Dispositions

- Review1174/check621 P2, G3 records once: repaired. Read capacity::reap,
  queue::land, reconcile::decide_intent and queue::record_landing. Reap and
  the live controller can both finish external ownership/Git probes after
  reading an open intent. record_landing now checks the current intent in
  the existing decision transaction before any approval, attempt, ticket
  or audit mutation. Resolving that intent and recording the landing remain
  in that same transaction. A second caller sees resolved and writes nothing.
  Retirement and undecided attention also re-read the current intent in
  their transactions. A stale undecided probe cannot raise a new intent
  item after another caller resolved it. An already resolved intent skips
  new probes. No proof of ownership or ancestry is replaced by this guard.
- Review1185/check625 P2, abandoned preparation: repaired. Read the acting
  implementation's final transaction and review's corresponding abandoned
  outcome before changing preparation. stop_preparation now reads the
  current execution and attempt in its existing transaction, even when
  there is no pending stop notification. A non-live attempt ends its still
  running execution as abandoned and raises no stopped item. A row already
  ended is left alone. A live requested stop still ends unchanged and raises
  stopped:unchanged. The two preparation-to-harness checks now read current
  attempt liveness as well as the existing stop signal; a non-live attempt
  launches no harness turn there. The acting path remains responsible for
  the abandoned outcome if the box has already come up. Existing teardown
  and inherited-owner proof still decide reservation release.
- Current P2 spec integration: deferred as directed. The previous archive
  records empty /yard/input, no remote and the absent f9f4602 target object.
  No target input was fabricated or installed. The ordinary landing queue
  must provide its supported bundle on the architecture conflict. The
  combined head must preserve Y83 G4/G5 and Y58 G6/G7, operator_mcp and its
  host scenarios, and pass full checks and review before final landing.
  This candidate does not claim that integration.
- Current P3 retained-owner stop acknowledgement: nonblocking disposition
  retained as directed. No inferred ownership proof or early release added.
- Previous P3 ownership-attention bookkeeping: nonblocking disposition
  retained as directed. This repair does not clear retained capacity or
  change the ownership-attention contract.

## Integrity and coverage

Project::tx holds the existing store mutex through the SQLite transaction.
The open-intent check and all landing mutations therefore serialize with
the competing caller. Execution status is not the admission condition:
an ended legacy row with an open intent can still be proved landed and
recorded. The existing running/ended outcome handling remains in place.
The current G3 records-once, G5 recovery and G10 serial landing contracts
are unchanged. ARCHITECTURE.md needs no contract change for these repairs.

No host scenario or assertion was changed or removed. Read the existing
G3 held-update-ref and moved-canonical scenarios, G5 startup/host-child
proofs, and G4 stopped-waiter and reserved-preparation scenarios. These
remain the controls for landing, restart, capacity and live stop behavior.
The G3 held-update-ref scenario already asserts one landing.recorded event.
It does not deterministically force the precise live-controller/reap
overlap after both external proofs. Its held Git owner prevents recovery
while that owner still acts. There is no existing fixture-held boundary
between completion of those proofs and the decision transaction. No new
hook, first-case duplicate or timing-based race scenario was added. The
precise overlapping-decisions proof remains source inspection, not a
claimed host regression reproduction. The new abandoned-preparation and
handoff guards likewise have source evidence here; the existing live-stop
and acting-path abandonment assertions are not claimed to exercise that
exact preparation interleaving.

Assertion lines removed: 0. Binary lines removed: 6, replacing the stale
retirement status read and stop-only preparation/handoff decisions.
No lifecycle, lock owner, test-only seam, sleep or retry was added.

Validation: cargo fmt --check, cargo clippy --all-targets --locked --
-D warnings and git diff --check. Host e2e remains the landing-stage gate;
it was not run in this worker, as instructed.
