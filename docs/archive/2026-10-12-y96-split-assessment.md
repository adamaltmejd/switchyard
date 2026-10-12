# Y-96 split assessment

Source head: `f2b6c3bb76e0359dfd0cd496b60871d3891d7cbd`.

Y-96 is not implemented. The worker instruction permits proposing a split
and stopping when the ticket is too large. This assessment separates two
mechanisms needed by the agreed phase-5 merged acceptance guarantee. It
does not change the current spec or authorize a daemon switch.

## Source findings

- `jobs/queue.rs::next` creates a fresh landing from an active approval.
  `land` keeps prepared target and merged identities in local variables
  and pins `refs/yard/landings/EXECUTION`. Its children record the parent,
  old target and merged head. A later preparation can create another merge.
- `store/executions.rs::set_intent` persists the identities for the final
  canonical mutation. Preparation is not a mutation intent. Treating it
  as one would let intent recovery decide unverified work.
- `jobs/reconcile.rs::project` ends running pre-intent landings after
  ownership checks. There is no prepared-input continuation reading that
  the scheduler resumes after restart.
- `jobs/queue.rs::land` sends a failed child to `returned`, which ends the
  landing, retires approval and schedules repair. Infrastructure errors
  instead raise red and retain approval for a fresh queued landing.
- `jobs/admit.rs::candidate_answer` validates candidate evidence through
  `checks::Input`. Landing child inputs are different: their base is the
  target old head and their head is the merged head. Candidate validation
  alone cannot authorize merged evidence.
- `store/checks.rs::decision` reads resolved candidate attention as the
  sole decision truth. This contract can be extended without another
  decision store.
- `plan.rs` restricts selected `operator_failure` to candidate gate and
  review nodes. Landing failure handling needs an admission change too.

These are source findings. No CLI reproduction, runtime validation or
sabotage execution is claimed.

## Proposed sequence

Yard proposal **230**, key `y96-exact-continuation`, covers exact prepared
merged-input continuation and recovery. It preserves completed eligible
child evidence, retries infrastructure errors truthfully, and requires
fresh preparation when the input changes. It uses ordinary execution rows
and checked ownership, with no acceptance API or second lifecycle. Its
proposed spec changes are confined to that new G2/G3 contract.

Yard proposal **231**, key `y96-merged-decisions`, depends on proposal 230.
It covers trusted failed/skipped merged decisions, admission, attention,
exact stale refusals and continued remaining requirements. Its proposed
spec changes include the threat model and G1/G2/G3/G8. It requires held-child
restart coverage after an accepted failure as well as a delayed stale
acceptance and the infrastructure-error refusal control.

Both proposals are parked for operator review. Neither is accepted or
started by this worker. Both retain the original exclusions and host-only
validation boundary. Y-96 and Y-61 remain incomplete until the full original
contract is implemented and observed on the approved exact head.

## Test-audit and validation

The test-audit skill was read. No test or assertion block changed, so no
assertion gate or suite sweep is claimed. The proposals require extending
existing controls and adding only distinct continuation/decision mechanisms.
Expected merged identities come from host Git; truthful evidence and
no-write observations compare store snapshots. Each proposed refusal has
a same-project success control. Crash points must hold real external work
and prove state before restart. Proposed sabotage names regeneration,
changed-input inheritance, fabricated passes, teardown and CAS bypass.

Test lines removed: 0. Binary lines removed: 0.

`cargo fmt --check` and `cargo clippy --all-targets --locked -- -D warnings`
passed before committing this assessment. Host e2e is deliberately
unrun under the worker instruction. No development daemon, project, box
or runtime was started. No `.yard/` file was changed.
