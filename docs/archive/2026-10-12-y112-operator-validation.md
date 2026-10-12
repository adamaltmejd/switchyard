# Y-112 inherited execution lock validation

Candidate: 938144caf46223217ccbf447063c5feb087451ba.
Base: f0de85e5f772610520334e1f98101e87b88b3735.
Proof: de0dac4b41f731d261fd241b9099b94e5c838bc3835505671577ca9dcd489d40.
CLI diff matches Git. Diff SHA256:
cfce8b1ce9474a1f5caac5de4d1228df88f29dd8d961bfdea192f1809e7d1772.

## Observation and correction

Y-110 af034ef's real lost-ready startup reached native ready and box
inventory. It failed its inherited-lock assertion at G4:1212 while the
real owner retained that descriptor. Its 36.78-second failed result is
preserved. The pinned nix 0.31.3 Flock guard explicitly unlocks its shared
open file description on drop. Closing the controller's guard therefore
unlocked its surviving descendants.

Y-112 returns the locked File instead. Parent drop closes only that
descriptor. The last inherited descriptor retains the lock. The one-file
change adds six lines and removes five; no assertion, scenario, dependency,
config, lifecycle or promised control changes. The separate lock_free
probe remains an independent, non-inherited guard. The actual callers
and Git, Pinfold and host-gate inheritance paths were read.

## Qualification

The standalone existing an_old_host_child_retains_capacity_across_restart
passes 1/0, 77 filtered: 31.31 scenario seconds, 46.126 total with compile.

The stronger startup fixture was still unlanded. Temporary combined
source af034ef plus only the exact Y-112 patch passes the full expanded
existing failed-preparation scenario: 1/0, 47.44 scenario seconds, 62.616
total with compile. It reaches retained native ownership, box absence
while the host lock is held, checked release, successful pending-stop
teardown and deliberate restart. It is not a standalone or landed tree.
The exact earlier af034ef source without this patch failed the existing
lock assertion; this is the recorded old-implementation comparison, not
a new mutation run. Later blocks were unreached in that comparison.

Changing only the combined startup error arm to propagate before honoring
stop fails directly at G4:1075, red versus stopped: cargo 101, 21.66
scenario seconds, 21.821 total. Later ownership and restart blocks are
unreached in that mutation and proven by the passing control. Exact
combined source was restored and its build passed.

Y-110 later repaired its two fixture roots, tool-owned clone cleanup and
query ordering in 27ef855. That changed candidate still needs its own
parallel host control on the landed Y-112 source. No earlier combined
result is substituted for that qualification.

## Landing

Approval128 led to landing2255. Actual fmt2256, clippy2257 and build2258
passed. Full real host gate2259 passed 78/0 in 584.39 scenario seconds.
It started 21:46:20.647Z and ended 21:56:20.217Z: 599.570 seconds including
build overhead. Landing intent7824, landing.recorded7826 and ticket.done7827
name exact merged commit938144caf46223217ccbf447063c5feb087451ba.
Cleanup2260 ended removed. The exact landed commit was consumed before
this operator record. No manual ticket closure substituted for landing.

## Evidence and limits

Private raw diffs, proof, review, controls, old-source failure and actual
landing audit are under /home/yard/validation/y112-938144-review-20261012,
/home/yard/validation/y110-af034e-review-20261012 and
/home/yard/validation/y110-y112-combined-20261012. Review 2249/check1172
passes without findings. Exact approval is 128. The actual recorded landing
and cleanup qualify completion of this prerequisite.

The failed fixture's exact surviving container was verified by name,
owner and project labels, stopped gracefully without a force-kill deadline,
then removed by native box down. Final stat returned absence exit3. Only
verified investigation resources were cleaned. Its failed native down
EAGAIN result remains preserved, not counted as cleanup success.

Y-110, the enforced Y-111 checkpoint and full Y-62/release scope remain.
Y-81 remains parked. The frozen operational daemon and reviewer are
unchanged. No push, publication, installation or runtime switchover.
