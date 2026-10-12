# Y-95 failed landing diagnosis

Candidate `0175eddc2e9f73328ae4e494ed798d2445e5d0c0` did not land
on target `e4ce7d57e5537679f78125a00f46a3ab4dc7d725`.
The operator supplied landing1736/gate1740 output: the real host suite
ended with 68 passed and 2 failed in 447.74 seconds. This supersedes no
historical evidence and proves no successful full-suite or actual landing.

The publication crash case failed during `yard init`, before its scenario:
`pinfold box list --label dev.yard.project=fe0f64e6ad` timed out. No
publication, crash or reconciliation assertion ran in that case. This is
not evidence of a publication mechanism defect or a timeout success.

The Codex containment case reached its linked-skill ticket, then observed
`red:ownership` rather than the expected `red:error` naming `.agents/skills`.
Its supplied event at seq25 names attempt2, execution4, ticketY-2, with
`lock_contract=true` and detail `external owner not proved gone; capacity
retained`. The daemon also reported unknown execution ownership. The
fixture is `/tmp/ye-3757721-g8-codex/p` on the host, not in this box.

## Source trace and limits

`jobs/capacity.rs::release` requires both a successful pinfold list proving
the execution's box absent and a free inherited host-effect lock. A failed
list, a still-listed box, a held lock or a lock-probe error retains capacity
and raises ownership. `effect` drops its own lock before these probes and
returns unknown ownership when release fails. This is the required fail-safe
behavior. The generic event does not reveal which probe failed.

`box.rs::list` uses the bounded control runner. `collect` kills the timed-out
process group and awaits its child. Git and pinfold effect children inherit
the host lock. The linked-skill refusal could therefore be masked by failed
ownership proof; the supplied output does not prove a broken symlink guard.
The init timeout is observed directly. A common cause for both failures is
not established. No root cause or safe runtime fix is claimed from source.

The changed required-proof comparison is not on either reported failure
path. That alone does not establish the new head is valid. The complete
parent contract, fresh host validation, published review and actual landing
remain required. The original integration record remains immutable.

## Disposition

Keep both scenarios, assertions, command deadlines and ownership controls.
Do not accept unknown ownership as successful teardown, loosen the expected
refusal or rerun an unchanged suite to turn the failure into a green claim.
No runtime or test change is justified by the available evidence.

A parked follow-up, proposal203, was proposed through `yard_propose` for bounded host
diagnosis of the two observations. It requests terminal gate logs and
read-only execution/audit, box, process and lock evidence before a repair.
Missing host evidence was also requested from the operator. A later healthy
box-list probe cannot by itself diagnose the failed probe. Further work
must preserve G5/G8 and the original Y-95 scope, and implement only a
reproduced defect. No host data is fabricated here.

Test-audit disposition: no assertion changed or removed; no guarantee row
changed. Test lines removed: 0. Binary lines removed: 0. No e2e, sabotage,
sleep, retry, test hook, deadline change or manual ownership mutation occurs
in this candidate. No .yard or operational daemon change is made.

This record is a diagnostic handoff, not a repaired integration candidate
or completion of Y-95. Local fmt/clippy validation cannot replace the failed
host gate. `cargo fmt --check` and
`cargo clippy --all-targets --locked -- -D warnings` passed before commit.
