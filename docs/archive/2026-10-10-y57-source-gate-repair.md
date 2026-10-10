# Y-57 first candidate source Gate repair

The operator's 2026-10-10 Gate covered candidate
8b344e03449870f564b1ebe42d4f96578b0d3c2e. It was a source and assertion
review, not a runtime failure report. This repair addresses its three items.
The earlier dated audit remains the record of that candidate.

## Default input and ownership

The final landing invalidates the published input for its default branch
under the publication lock, in the admitted intent transaction, before
update-ref. The lock is released for the target Git effect. Nondefault
landings preserve the current default input and keep their independent
queue partitions. No queue-wide Git effect lock or second limit is added.

Completed false CAS and successful CAS with ancestry proof publish actual
validated input. Their policy Git reads remain inside the landing's shared
reservation and inherited host lock. The target lock stays held through
capacity release. If ownership remains unknown, the input stays unavailable.
Automatic scheduler/start refresh cannot restore an unavailable input.
The existing intent reconciliation and capacity ownership reaper publish
only after their external ownership proof. Startup refuses publication over
an open default intent or retained default landing mutation reservation.
Intent start can reach reconciliation before reading unavailable capacity
input. The one configuration map and its default selector remain the
allocation authority; no mutation lifecycle or separate config owner exists.

Sync re-reads default identity under publication after taking the target
lock, and again immediately before import invalidation. If selection changed
during validation, incoming configuration is validated against the new
default before effect. Ordinary imports release publication during their
Git child. A completed false CAS revalidates actual input. Command errors
return without restoring it. Default selection also invalidates before
symbolic-ref and leaves input unavailable on error. Pre-mutation refusals
preserve input; checkout-only consume does not heal an unknown mutation.
Named sync refuses a retained target mutation owner after the target lock
proves the live controller has finished.

## Changed assertion Gate

G4's promised scoped box absence is retained and repaired. Before approval,
the scripted model holds this fixture's first real worker. Pinfold reports
its box and project label; the recorded handle only associates that positive
observation with this worker. A query using pinfold's reported project label
shows the allowed box. The same scoped query later directly asserts no box
while the target Git child holds the sole slot. No absent-handle list,
guessed label or host-wide absence is used.

The expected project identity comes from pinfold, and the expected empty
set comes from the fixture holding a host Git effect at max_executions one.
Sabotage: allow an extra worker box during that held landing without recording
its handle. The scoped query still sees it. Existing G4's worker-held
observer cannot prove box absence under an independent-target host landing.
Waiting executions, the exact reserved landing, model exclusion and allowed
target continuations remain. G3 and the existing G10 judgment blocks are
preserved.

Fixture-owned release guards disarm the Git wrapper and write its release
FIFO during unwinding, including an expected sabotage assertion failure.
The FIFO is opened read/write before arming, so cleanup never waits for a
reader. A scoped model release guard releases the identity-capture worker
before scoped-thread joining during unwinding. The release helpers also
serve normal continuation. No operator process killing, test-only binary
paths, sleeps, retries or timeout-as-pass is introduced.

This delta removes 23 test source lines and 32 binary source lines. It
replaces one faulty assertion block and adds its positive scoped observer;
it removes no promised observation or guarantee clause.

## Validation

cargo fmt --check and cargo clippy --all-targets --locked -- -D warnings
pass for this repair. The host e2e suite and runtime sabotage were not run,
as instructed. Fresh exact candidate review and full host validation remain
operator gates. Operational .yard configuration, daemon and reviewer were
not changed, and no development binary opened construction state.
