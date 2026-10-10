# Y-57 publication wait and fixture admission repair

The operator's exact-head 41ae53e23ec6b161e8998e735f0a99a3a1db8a53
host observations on 2026-10-10 came from full52 session87326,
log /tmp/switchyard-y57-stack-fixed-host.log. The run was still live.
The earlier startup stack overflow was absent and most existing scenarios
passed. This audit claims no terminal total or full-suite pass. The live
run, checkout, FIFOs and operational processes were not touched.

## Default publication wait

The existing one_slot_lands_with_child_gates_and_excludes_a_worker failed:
execution9 reported that the project default configuration was not loaded.
The second worker had already been admitted and was waiting for capacity.
A legitimate default landing invalidated publication before its Git effect.
The capacity loop treated that unavailable input as a terminal error before
checking the execution's continued eligibility.

Unavailable publication now yields no available slot in the same capacity
loop. The execution reserves nothing and performs no external work. It
uses the existing execution.waiting event and project event notification,
registered before the publication reading, without adding a lifecycle or
another limit. Validated publication wakes it to reread the exact current
default limit. No selected-target or stale configuration supplies a limit,
and allocation runs no Git.

Eligibility is still read on every iteration and rechecked in the
reservation transaction under the publication lock. Stop ends an unreserved
worker row as unchanged; abandon ends its attempt. Their transactions wake
the waiter, which exits without acting and leaves those recorded outcomes
intact. The current stop handle remains installed until its controller
finishes. Unknown mutation ownership cannot publish input or grant a slot;
its existing reservation and attention remain visible until external proof
and existing resolution permit publication. No owner is guessed released.

## Fixture source and test-audit Gate

Read-only operator snapshots showed the two new target scenarios had only
two live, lane-holding candidate attempts awaiting approval. Their third
ticket had no attempt. target_candidates creates three tickets and waits
for all approvals before approving any; max_lanes two cannot meet that
precondition. The earlier static Gate missed this admission deadlock.
Those observations established no runtime independent-target progress.

Both fixtures now declare max_lanes three. Their shared max_executions
values remain two for G10 and one for G4. Approval ordering and every
assertion remain unchanged. This change does not serialize the suite or
increase effect capacity. Fixture-owned FIFO and model cleanup remain.

The test-audit disposition is retain all blocks, correcting fixture
admission only. No new assertion, scenario or guarantee clause is added.
The inherited Gate answers remain:

- G10/G3: feature/b lands while feature/a's real update-ref child is held;
  the second feature/a waits, and named sync refuses the exact intent.
  Canonical refs, candidate clone heads and ancestry come from Git;
  expected ordering comes from fixture approvals. A project-global busy
  predicate or mutation lock prevents b's continuation. Removing target
  serialization starts the second a early. Existing CAS/repair scenarios
  cannot show independent-target progress under this held child.
- G4: a held feature/a host effect at cap one excludes two approved targets
  and a new worker. History and store expose waiting and the exact reserved
  landing; the host model supplies the expected unchanged turn count.
  Pinfold supplies the project label from a real positive held-worker
  observation, then directly observes that project's empty box set. Git
  supplies target heads before and after release. Bypassing shared landing
  admission or counting only boxes lets another effect act. An unrecorded
  fixture box still fails the scoped absence block. Existing same-target
  capacity blocks do not expose independent-target bypass. Release permits
  all approved targets to land at their fixture candidate heads.
- Existing G4 one-slot continuation is retained: after the held host gate
  is released, the first landing records and the second worker reaches
  approval in the same project. Restoring the unavailable-input error
  prevents that continuation. The separate stopped-waiter block retains
  its stopped row and deliberate later allowed start.

Test assertion lines removed: zero. Test source lines removed: two fixture
configuration lines, replaced with three-lane declarations. Binary source
lines removed: eleven, replaced by optional-input slot exclusion. No
sleeps, retries, timeout-as-pass, public model endpoints or test-only hooks
are added. The heap-pinned Git I/O boundary and bounded textual proof are
preserved. Earlier dated audits remain unchanged.

## Validation

cargo fmt --check and cargo clippy --all-targets --locked -- -D warnings
pass. cargo build -p yard --locked passes. Host e2e and runtime sabotage
were not run in the worker, as
instructed. Fresh source/test Gate, exact-head review and the unchanged
full52 host suite remain required before approval. Operational .yard
configuration, daemon and reviewer were not changed.
