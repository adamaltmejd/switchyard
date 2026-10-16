# Y80 G5 checked-release observer

The operator's 2026-10-16 disposition supersedes the proposed extra
Pinfold-wrapper/FIFO phase. This change adapts only the existing held worker
crash case in `g05_intent_precedes_effect.rs`, based on candidate
`a4151c7101aba74f3ac88486afe60e7ebc278bbb`.

## Evidence and boundary

The operator reported timeout attention 598 on execution 3962. That failure
remains evidence. This change neither attributes it to an original CI holder
or list failure nor claims that unknown ownership is completed teardown.
Startup already reconciles before serving. External ownership may release
later; serving alone does not prove the old execution has ended.

The existing real model request still holds the worker until daemon death.
Before restart, the test still observes the running execution in SQLite,
the worker's actual box and project label through Pinfold, and `feature.txt`
through the host filesystem. It records the last audit sequence after writer
death. After restart, bounded ordered operator events must show checked
release and then an interrupted end for that exact stored execution ID.
Any attention encountered before that end must be `red:ownership` for the
same execution. Unrelated attention, including errors, fails immediately.
Ownership attention is neither erased nor force-resolved by the fixture.

The ended/interrupted store assertion and real box-absence assertion follow
those events. The same supported `attempt.start` then continues the retained
tree. Canonical Git must contain the fixture's exact `feature.txt` bytes,
and the stored implementation reasons must remain `first`, then `retry`.
No production code or control changes. G5's current Shown by remains accurate;
the binding architecture needs no change for this observer correction.

## Test-audit gate

- Material risk: G5 recovery must retain useful work without duplicating an
  owner. A serving-time ended assertion rejects valid fail-safe ownership
  before external release. Waiting for the exact release and interrupted
  end preserves that risk-based case without weakening ownership proof.
- Sabotage: recover the implementation with an outcome other than
  `interrupted`, or discard its retained tree. The outcome assertion or
  continued candidate's Git contents must fail. The test comment names
  these changes. Neither sabotage nor host e2e was run in this worker.
- Existing representative: adapt the held worker crash case. G4 retains the
  harder inherited-owner restart case. No duplicate ownership phase,
  wrapper, FIFO, permanent helper or fixture is added. Added machines,
  historical builds, current/debugger builds, boxes and model requests:
  zero. No elapsed savings or new full-suite resource totals are claimed.
- Outside oracles: the fixture supplies the stored execution identity and
  expected file bytes; SQLite exposes the durable execution/audit state;
  operator MCP exposes ordered release/end and typed attention; Pinfold
  exposes box presence/absence; canonical Git proves the continued result.
  G5 supplies the expected interrupted outcome and checked-release rule.

The existing deadlines, concurrency and held crash point are unchanged.
No sleeps, retries, public model endpoint, synthetic runtime frames or
binary barriers are introduced. No assertions or product lines are deleted
for coverage reduction; the previous immediate observation is moved behind
its release/end evidence. This commit changes one test file and this dated
disposition; production binary lines changed: zero.
The test-file diff adds 27 lines and removes 6; binary lines added/removed:
0/0. The removed lines are replaced comments, identity capture/assertion
lines and the old history watcher, not removed recovery controls.

## Validation handoff

Worker validation passed: `cargo fmt --check`,
`cargo clippy --all-targets --locked -- -D warnings`, `cargo build --locked`
and `git diff --check`. The first fmt check found only wrapping of the new
identity capture; formatting was applied and the check passed again.
Host e2e and sabotage execution remain unrun here and belong to the operator.
The operator-reported running Mac qualification CI 38948373532 remains
prior-source evidence; it is not current-source qualification for this edit.
No platform success, release readiness or artifact requalification is claimed.
Construction, installed runtimes and `.yard` are untouched.
