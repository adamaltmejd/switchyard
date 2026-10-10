# Y-57 operator validation — 2026-10-10

Y-57 landed and was consumed through supported sync. The useful redesign
release remains in progress. This is operator evidence for this ticket.

## Candidate and scope

Ticket Y-57 revision 8, attempt 59. Base:
`c9a7fd7de2806014efafdef20eab72e6c3551c03`. Landed candidate:
`b813491a29999ff9b808f36fc6274ed338cafd96`. Proof:
`ebbff2092400784446c96ecc0f53bb71f11b2381086a12e89f79b9678a7d719e`.

The existing queue serializes eligible approvals per frozen target.
Unrelated targets share the existing project execution cap. Target locks
also serialize named sync and intent recovery. Intents persist their
exact target ref; recovery retains ownership and ancestry proof.
Default configuration publication keeps one authoritative reading.
Unknown mutation ownership leaves that reading unavailable. A waiter
reserves nothing until capacity admission; a reserved landing retains its
reservation and host lock while waiting for publication. An ended attempt
lapses its approval and withdraws without another effect or attention.
No second queue, allocator, lifecycle or ownership proof was introduced.

## Source and test gate

The operator read the initial candidate, each repair delta, the actual
current attempt show and diff, and all protected architecture changes.
The latest three-file delta contains the ended-attempt lapse, two binding
architecture lines and a dated diagnosis. Its tests are unchanged.
Boxed fmt check 681, Clippy check 682 and build check 683 passed on the
exact landed head and proof. Review execution 1275 passed as check 684.
Its single P3 notes an
unreproduced nullable legacy intent-field panic concern, without identifying
a weakened reachable guarantee. The operator accepted the nonblocking
finding and retained it for final migration verification. No fallback to a
guessed historical target was introduced. Approval 73 binds the exact head
and proof after the actual show and diff were read again; the diff is byte
identical to the previously read candidate.

G3's existing deterministic held-update-ref restart scenario now lands
feature/a while main stays fixed. It observes the persisted intent ref and
exactly-once recording through Git and the store.

G10 adds a held feature/a Git child. Feature/b lands while a second
feature/a waits; release permits ordered feature/a landings. Named sync
refuses the held target by intent identity and permits the other target.
The separate G4 scenario uses cap one: a held landing excludes other
approved targets and a new worker. Reservations, scoped pinfold boxes and
host-model calls provide the observations, followed by allowed continuation.
Each fixture permits three candidate admission lanes while retaining its
execution cap of two or one.

## Current host validation

The isolated host checkout at exact b813491 ran
`cargo test -p e2e --locked`: 52 passed, zero failed, in 187.75 seconds.
This includes the previously failing G2 abandon scenario, both new target
scenarios, one-slot continuation, feature intent recovery, old-binary
migration and operator MCP controls.
Log: `/tmp/switchyard-y57-abandon-repaired-host.log`.
This independent exact-head run preceded fresh landing checks.

## Repairs and failed evidence

Initial source review found missing default invalidation around landing
CAS, unsafe sync publication and a vacuous scoped box absence observation.
Supported ticket edits required repair while preserving controls.

Candidate c02f267 failed host validation: one passed, 51 failed in
76.61 seconds. Real daemon worker stack overflow interrupted supported
sync before guarantee observations. The repair heap-pins Git's bounded
output collection future without changing runtime stack settings.
Compiler layout and assembly measurements diagnose stack size; they are
not platform runtime validation. The first oversized proof handoff was
refused; the worker retained compact evidence within the existing limit.

Candidate 41ae53e had 49 passing and three failing scenarios in
736.45 seconds. An existing one-slot continuation encountered unavailable
default input. Two new fixtures waited for a third candidate their two
admission lanes could not admit. The operator's earlier source gate missed
that fixture precondition. The repair changed the existing capacity wait
and fixture admission settings, retaining all 52 scenarios and assertions.

Candidate c4014a8 passed all 52 host scenarios in 218.71 seconds. Review
check 677 nevertheless found a P1 race: a reserved landing could fail if
publication disappeared before its policy check. P2 findings covered
unnamed sync using a pre-lock default and queue insertion before red
exclusion. A P3 finding covered named intent-recovery attention selection.
Yard automatically started repair 1266; no blocking finding was overridden.
These races were source findings, not claimed host reproductions.

Candidate 4e89bd1 fixed those findings, but host validation found an
existing G2 regression: 51 passed, one failed in 205.07 seconds. The real
execution.ended event at seq 28 named landing execution 2 and outcome
abandoned; the unchanged fixture required withdrawn. Supported ticket edit
revision 7 to 8 interrupted review and requested the existing withdrawal
path, retaining the publication wait and all assertions. Repair 1271
produced b813491. Its subsequent full host result is above.
Log: `/tmp/switchyard-y57-review-repaired-host.log`.

## Fault probe limitation

After c4014a8's passing run ended, an isolated source mutation removed the
busy-target skip. It also admitted duplicate landings for an already
running attempt. External store observations showed premature landing rows
and extra retained reservations. Feature/b could not reach the committed
post-landing assertion. The probe ended with exit 101: zero passed, one
failed, 51 filtered, in 611.74 seconds, at the observation deadline.
This establishes no assertion-level sabotage validation. A deadline is
not counted as a pass. Source was restored after terminal completion;
git diff and status proved the checkout clean before its next use.
Log: `/tmp/switchyard-y57-target-ordering-sabotage.log`.

## Landing and operational boundary

Landing execution 1276 passed fmt 1277, Clippy 1278 and build 1279.
Host e2e gate 1280 passed all 52 scenarios in 221.34 seconds.
Its end was seq 4395, intent seq 4396, and landing execution end seq 4397.
Landing.recorded was seq 4398; ticket.done was seq 4399 and attempt.ended
was seq 4400. The merged head equals the approved candidate.
Cleanup execution 1281 started at seq 4401 and ended removed at seq 4402.
Supported sync consumed b813491 from c9a7fd7 at seq 4403; the subsequent
empty watch returned seq 4403. The frozen construction
executable SHA256 was verified as
`05ae25832964703c81525e7c5f5be7a8148954517cfddf69ef9846953bf1ebb2`.
The operational binary, daemon and model settings were preserved.
The useful redesign release and its platform validation remain incomplete.
