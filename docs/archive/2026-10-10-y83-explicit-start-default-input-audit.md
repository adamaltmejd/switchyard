# Y-83 explicit-start default input repair, 2026-10-10

Review1227/check656 passed with P2 explicit-start default input binding.
Source inspection confirms attempt_start held canonical but used load,
which could return a validated reading without publishing it for allocation.
Admission could use that reading while capacity::effect read an older or
unavailable published default. This is a G4 input-binding omission, not a
claimed host reproduction or a general cache optimization.

attempt_start now calls the existing refresh_default under its existing
canonical lock. That function invalidates the default selector, validates
canonical's current default, publishes the returned branch/head entry in
the existing configuration map and notifies capacity waiters. Validation
failure leaves the default unavailable and returns before admission or
answering attention. There is no second config authority or numeric limit.

Source binding audit:

- Inactive tickets use that returned default for fallback target selection,
  the transaction's target identity check and spare_lanes. Explicit ticket
  targets still load their own configuration; max_lanes admission is unchanged.
  Publication precedes recording and spawning the implementation execution.
- Live attempts retain their frozen target selection. Publication precedes
  attention identity validation and canonical release for answer_start.
  Existing start exits, landing-intent reconciliation, rerun/repair semantics
  and ownership refusal remain unchanged.
- capacity::effect takes canonical before reading the published default and
  reserving. It performs no allocator Git or target-limit fallback. A later
  canonical mutation uses the existing invalidation/refresh mechanism, so
  publication here does not freeze the allocator to an obsolete start reading.

Coverage disposition: no assertion or scenario changes. Existing G4 manual
start races, stopped-waiter and stopped-preparation restart controls, and the
default-selection/lowered-limit landing scenario remain. Their held worker,
host Git and gate points occur after admission or on the landing path. None
holds between explicit-start validation/publication and allocation while
excluding a normal scheduler refresh. They do not distinguish restoring
only the old load call. No supported deterministic held point was found for
that narrow stale/unavailable input case; this repair has source binding
evidence, not a new race test. No timing inference, sleep, retry, scheduler
barrier or test-only binary path is added. All 50 scenarios and every guard
and identity assertion remain unchanged. Test-audit Gate adds no assertion
block to judge; assertion lines removed: 0.

The operator reports all 50 exact 7f613bf host scenarios passed in 252.04
seconds, log /tmp/switchyard-y83-queue-policy-repair-host.log. The exact
expanded G4 scenario detected restoring only old queue::next in 13.18
seconds through an unowned policy observation before capacity, log
/tmp/switchyard-y83-queue-policy-sabotage.log; source was restored clean.
These are operator evidence for the preceding queue repair, not worker-run
tests or a runtime reproduction of explicit-start stale publication.

Y58 integration, G4/G5/G9 controls, schema, .yard and frozen operational
settings are unchanged. Prior dated archives remain history. Worker checks
cargo fmt --check, cargo clippy --all-targets --locked -- -D warnings
and git diff --check passed. Host e2e is not run in this worker as instructed.
Fresh exact-head review and full host validation remain required before
final approval.
