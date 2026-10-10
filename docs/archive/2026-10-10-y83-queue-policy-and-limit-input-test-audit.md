# Y-83 queue policy and allocator input repair, 2026-10-10

Review1222/check652 passed exact 4f9ac519774d32b01934fde5718a1ae1fecb9471
with P2 queue pre-admission Git and P3 allocator limit-read Git. Source
inspection confirms both omissions. The operator reports this head passed
50 host scenarios in 208.80 seconds and a qualified omission probe failed
in 10.36 seconds. These are operator observations, not worker-run host
tests. They did not prove the first queue-policy occurrence. Earlier dated
worker audits remain history; their completeness claims were insufficient.

queue::next now screens store rows and creates the existing landing job.
It performs no target-config or protected-path Git. Initial preparation
reserves that row, inherits its host lock, then takes canonical. Its initial
admit rereads the active approval, candidate base/head/proof, ticket revision,
config digests and current policy before preparing any merge. A lapse uses
the existing withdrawal path. Child-gate and final-intent admits remain
current and guarded. The project-wide serial queue and red/open-intent
exclusions remain. There is no new queue, lifecycle or nested reservation.

The allocator now reads the published default branch's validated entry
from the existing configuration map. The default selector and branch/head
entries are in that same map; there is no separate numeric limit or second
configuration document. Startup and scheduler refresh publish the validated
canonical default under canonical's lock. Sync invalidates the selector
before mutation, then validates and publishes the actual current default
before returning. Landing target mutation likewise invalidates before
acting and refreshes inside the reserved final effect after a proved
outcome. Failed validation leaves no allocatable default, not a guessed
previous limit. Ordinary target cache misses cannot replace the published
default entry with an earlier head. The default selector is rebuilt on
startup, not a persistent lease or owner.

Allocation acquires canonical to read that input and reserve. It launches
no head-branch, branch-validation, rev-parse or config Git before reserving.
It still reads the current project default on every capacity reconsideration,
never the attempt's selected-target limits. Sync/default/limit changes and
scheduler refresh notify waiters; acquisition of validated input is not
queued behind a full execution limit. max_lanes admission is unchanged and
uses the same validated default config returned by scheduler refresh.

Source audit boundaries:

- Registration and scheduler configuration reads validate canonical input
  for admission. Operator sync validates/imports/selects that input under
  canonical, and operator status/ticket/start validation still performs
  its existing control-plane configuration reads. None is relabelled as a
  landing, and no read-only exemption is added to the specification.
- A created execution's allocator performs only in-memory/store operations
  before reservation. Its first external execution work is inside HOST_LOCK.
- queue::next performs only store reads/writes and spawn. Preparation owns
  initial config/current-policy admission, protected-path diff, target read,
  ancestry/merge/commit and prepared ref. Conflict-return bundle creation
  uses that same effect. Every child admission owns its current config and
  policy Git, then releases the parent before the child acts. Failed-child
  return reserves the landing before bundle Git. Final admission, intent,
  target compare-and-swap, ancestry and default-input refresh share its effect.
- Existing recovery canonical probes after ownership proof are unchanged.
  The scheduler's earlier protected-path diff that decides whether to give
  automatic approval is also unchanged. It precedes a recorded approval;
  this repair and observer prove the subsequent queued landing-policy
  path, not ownership of that earlier judgment. No claim that this earlier
  diff is capacity-guarded or new exemption for it is made here.

Test-audit Gate: extend the existing
landing_gate_policy_reads_hold_the_shared_execution_slot, keeping all
50 scenarios and every previous assertion.

- G4's shown-by clause adds a real held worker excluding the first queued
  policy read after default selection and a lowering sync. Begin with two
  real held model turns under main's limit two. Select capacity with limit
  three, then sync it down to one while those effects drain. The frozen
  main target still declares two. Release Feature to its automatic approval;
  its landing must wait with reason capacity, unreserved, while the external
  store shows exactly the Holder execution still owning the sole slot.
- The host Git wrapper sends every protected-path call to a FIFO observer,
  including calls without an inherited landing lock. A read-only external
  SQLite connection records ownership and reservation identities once an
  active automatic approval exists. That recorded approval separates the
  first queue-policy read from the earlier decision that gives approval.
  No post-approval policy call may occur while the holder acts. Restoring
  policy Git in queue::next records the unowned call before its later
  capacity-wait event, failing the observer without a timeout as a pass.
- Release Holder and observe stopped:unchanged in the same project. The
  existing initial target-read lock/reservation, no prepared ref/child,
  seen-or-future start identity, held gate-policy diff, next child and final
  canonical-byte assertions remain. Every post-approval policy observation
  must name that same sole reserved landing and its inherited lock. This
  supplies the positive continuation, not an absent/never-admitted ticket.
- Choosing main's selected-target limit instead of the current default
  breaks the lowered-limit wait. The scenario proves the combined default
  selection and lowering result; it does not claim a separate sabotage of
  only sync publication when scheduler refresh can correct that publication.
  Publication order and invalidation on all daemon target/default mutations
  are source-audited. The allocator has no pre-reservation Git in source.
- The fixture provides limits, branches, names, held turns and expected
  landed bytes. The CLI history supplies named wait/start identities;
  external rows agree on those identities. Host FIFOs/open descriptors,
  actual Git children, read-only store snapshots and canonical Git observe
  effects. Neither expected ownership nor exclusion is inferred from time.
- Existing blocks held workers/gates or filtered for already-owned Git;
  they missed the first unowned post-approval queue call. This extension
  observes that distinct omitted path without a twin scenario. The FIFO
  observer services actual calls, not retries. Its Drop disables the probe,
  releases held model/FIFO replies and ends the reader, including during
  expected sabotage assertion failures. No binary hook or internal import
  is added.

Assertion lines removed: 0. Binary concepts removed: the unreserved queue
policy path and the allocator's execution-owned configuration Git path.
The extra config structure groups the existing map with its selected
validated default; it introduces no role limit, scheduler, lease, receipt,
owner or dependency. Retained-owner stop refusal/control, preparation latch,
abandonment and once-only guards, Y58 MCP/clauses and operational .yard/frozen
settings are preserved. The unrelated transient reap-row warning and final
ownership/preparation classifications are not expanded here.

Worker validation: cargo fmt --check, cargo clippy --all-targets --locked
-- -D warnings and git diff --check passed. Host e2e was not run in this
worker, as instructed. Fresh exact-head review and full host validation
remain required before approval.
