# Y-142 consumed-input integrity correction and test-audit Gate

This correction supersedes the candidate-retention claims in
`2026-10-13-y142-candidate-retention-repair.md` and the reviews-only staging
oracle in `2026-10-13-y142-test-audit.md`. Those historical files are unchanged.
Failed host result2691 on merged ref
`2c492521173051d0eba41d980ba417025ead6c32` remains preserved. Its five observed
consumers exposed an incorrect fixture expectation, not a qualification bug.
An exact candidate output tuple does not prove unchanged consumed inputs.

The completed-implementer exception is removed. Qualification again compares
all consumed definitions, including trusted staged sources, before checking
actual typed supplier bindings. Original producer qualification, ownership
and checked teardown remain intact. The architecture's exception is removed,
and Check1367 P2's contradictory statement that no active definition adoption
is exposed is corrected to describe the bounded ordinary candidate CAS.

The changed G2 assertion blocks pass the test-audit Gate as follows:

- Clause: G2's Shown by now explicitly requires the affected implementer,
  both affected reviewers and both consequent candidate gates after each
  independent extension and skill adoption. Independent gate-only and
  reviewer-settings-only retention controls remain in the same scenario.
- Sabotage: discard trusted staged sources when qualifying a completed
  implementer whose old candidate tuple matches. The test comment names it.
  Existing named sabotages remain: resolved overwrites adopted definitions,
  stale tool/skill identity and whole-panel invalidation.
- Hardest case and distinct mechanism: adoption follows completed judging,
  when an old eligible producer could wrongly survive. Prior gate-only and
  reviewer-only changes do not alter shared implementation inputs. This
  repairs the existing scenario; it adds no twin. G6 frozen-source and G11
  safeguard-sync coverage remain the survivors for their mechanisms.
- External oracles: all three selected Pi agents consume the fixture's
  extension and skill paths. The host model counts one initial implementation
  and one per staged adoption. Reviews count three initial, one profile,
  three supplier/profile, one routing, two settings, then two per staged
  adoption. The unrelated read-only capacity holder is excluded by its report
  grant. Host gate files count one original invocation each, one independent
  changed-gate invocation, two invocations each for fresh candidate suppliers,
  and one original-definition invocation each at landing. No count comes from
  Yard's rows or digest logic.

The scripted model commits different fixture feature bytes for each staged
adoption. Host Git supplies each fresh head, its ancestry and base. Actual
implementer results must supply those values; every candidate gate and review
must name that fresh execution and result output. The final review also names
the actual first-review execution and agrees with its immutable verdict.
Proof identity agreement checks unchanged fixture proof bytes, rather than
reimplementing Yard's digest. Changed reviewer digests cannot retain old
identity; implementation has no judgment digest. Host Git supplies selected
source blobs and the unchanged image commit, Dockerfile blob and whole context.
Every old execution/result, check and plan revision is snapshotted unchanged
across each staged adoption. Original admission and ticket assertions remain.
Approval withdrawal and stale expected/current CAS identities remain covered.

Test source lines removed: 23. Binary source lines removed: 22. The deleted
binary lines are the exception and replaced mutable declarations; no control
is removed. No runtime stub, retry, deadline, concurrency or lifecycle changes
are introduced. The existing capacity hold and checked handoff are unchanged.

Validation: `cargo fmt --check` and
`cargo clippy --all-targets --locked -- -D warnings`. Host e2e and independent
named runtime sabotages were not run in this clone and remain post-approval
gates, including the completed-producer staging exception sabotage above.

Y-140 retains its prerequisites and original ordinary Y-136 and packaged
Y-109 integration obligation. Y135 Proposal360, Check1340 P3, Y139 Check1348
P3 and Check1356 P2 remain carried in. This correction does not complete
Y-137, Y-136 or Y-62, or qualify implementer settings, held startup or the
definition-only implementation handoff gap. Implementation-phase adoption,
image and landing-policy changes, and mandatory packaged activation remain
with the named dependent children. Frozen startup, publication, ownership
and checked-handoff controls remain required.
