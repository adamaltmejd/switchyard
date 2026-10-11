# Y-92 combined landing failure and ownership prerequisite, 2026-10-11

This is a new diagnostic record, not package acceptance or ticket completion.
Earlier archives remain unchanged. The operator reports the landing host gate
on merged ref `6e0c6211a9b81ac133ad33dd2fafc6fbfc0d7d7a` against target
`e4ce7d57e5537679f78125a00f46a3ab4dc7d725`: 70 passed, two failed,
zero ignored, 1165.64 seconds. The failed scenarios are
`a_seat_killed_after_publishing_has_published` and
`a_gate_errors_start_reruns_that_gate` in `g08_review_is_a_publication`.
No e2e was run in this candidate.

## Target and retained scope

`git bundle verify /yard/input/target.bundle` succeeded. Fetching its
`refs/heads/main` and merging FETCH_HEAD reported already up to date:
the supplied target is already an ancestor of this combined candidate.
No serialized bundle was reconstructed. Production, tests, package pins,
Y-97/Y-98 controls, all 15 guarantee rows, prior archives and repository
`.yard` were not changed by this investigation. No host package or
controller execution was performed.

## Measured failure and source trace

The supplied operator history holds a real published review before daemon
termination. Y-1 attempt 2 execution 4 has a pass publication, a refused
second publication, and no eligible result while held. Another attempt
reaches approval before termination. On restart the first review raises
`red:ownership` at seq 32. Later the same execution records
`execution.released` at seq 33 and `execution.ended` with `interrupted`
at seq 34. The test then fails waiting for Y-1 approval within its existing
600-second deadline. A timeout is failure, not evidence of acceptance.

`jobs/capacity.rs::release` proves the box absent and inherited host lock
free, sets checked teardown and releases capacity. It leaves ownership
attention open. `capacity::reap` subsequently ends a still-running review
as interrupted. `jobs/mod.rs::runnable` refuses advancement while any
nonproposal/nonapproval attention remains open for that attempt. This
explains why the supplied review history stops after checked release.
It does not justify releasing any owner without proof or clearing an
unrelated attention item.

The gate-error scenario's assertion/timeout block was not supplied.
Its source was read through candidate gate refusal, same-head retry,
landing-child refusal/requeue and held gate abandonment. The available
history does not diagnose which block failed. The exact missing artifact
is that scenario's failure block and preceding events from this landing
host log; it was requested from the operator. No common cause is claimed.

## Test-audit disposition and prerequisite

Keep both existing failing scenarios and every assertion. Test lines
removed: zero. Binary lines removed: zero. No test edit or new assertion
is proposed here. The kept published-seat crash block serves G8's explicit
same-execution interrupted result after G5 reconciliation. Sabotage is
allowing successor work before checked teardown or losing the registered
publication during restart. Its held real fixture request, external store
observations before restart, and same-execution binding assertions cover
the hardest case; ordinary publication does not cover death at that edge.
Expected publication content comes from the scripted model, identities
from the fixture's admitted execution and host git, and eligibility from
the binding G5/G8 contract. The gate-error scenario independently serves
G8 infrastructure refusal without judgment/repair and supported retries;
its fixture sabotages real box creation, then removes that sabotage in
the same project. Its expected outcomes come from that fixture and host
git, rather than a copy of Yard's implementation. Neither is deleted to
make the landing green.

The test-audit rule requires ticketing a kept block's product bug.
Proposal 205, `Resume published review after checked ownership
reconciliation`, names this single recovery mechanism and preserves the
proof-before-release boundary and existing host scenario. Proposal 206
requests it as a Y-92 prerequisite. The operator decides both proposals.
Y-92's complete package scope remains open; this record is not a docs-only
replacement. No daemon recovery repair is folded into package selection.

Fresh repaired combined source audit, focused package tests, the complete
host suite, current review and actual landing remain required. Neither
partial reports nor the 70 successful scenarios establish acceptance.

Candidate checks: `cargo fmt --check`,
`cargo clippy --all-targets --locked -- -D warnings`, `cargo build --locked`
and `git diff --check` passed. Host e2e was deliberately not run here.
