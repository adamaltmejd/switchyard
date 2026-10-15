# Y80 migration recovery expectation correction

2026-10-15. This corrects the restored-landing characterization in
[the candidate runbook](2026-10-15-y80-migration-switchover.md), specifically
its paragraph beginning "Its separate restored old copy", and the G15 and
sabotage descriptions in [the cleanup audit](2026-10-15-code-cleanup.md).
Those archives remain unchanged as the original candidate record.

## Controlling host evidence

The operator ran `cargo test -p e2e --locked an_upgrade_` against candidate
`b57cf66eca79d73b01ad7304a7afd17fe44bcb91`, proof
`0dd919a3d999676131499385b84306b89dfdc7ff378cd947baabe045df2bd5a2`.
It exited 101: one passed and one failed in 50.56 seconds; full wrapper
56.105 seconds. Raw result SHA256:
`fd6b1deafbb89f63e36a7ada4ba3fa2fd0bb17a1ffbfab6407830f9a176bf472`.
Verified private Pinfold 0.2.3 SHA256:
`d89172077c000759fab378b3cd9b515b5fe9cd4321294bd98cdc9d2e720fec18`.
The operator reports clean source and unchanged runtime artifact. Review2057
passed without findings; that review does not override this runtime failure.
No full-suite result is claimed.

The genuine 0.0.3 pre-box child upgrade passed. The planner/limit/old-restore
case failed at line 423 on its post-release expectation. The real old source
was `81da8c677a0ab43966588429c24a8affedb9e8ec`, built once and version-verified.
The disposable restore had migrated, raised ownership attention while the old
Git child was held, then advanced canonical to the exact intended merged head
after release. After restart the test expected open/reserved=1; the store
correctly showed resolved/reserved=0.

Retained restored store: schema 26, landing execution 5 ended/landed,
lock_contract=0, reserved=0, intent resolved. Audit 29 raised ownership on 5;
audit 30 released 5; audit 32 recorded its landing. The untouched original
upgrade project snapshot remained schema 8 because the assertion stopped the
test before its successor branch. The operator's read-only capture retained
15 snapshots of three stores in `migration-recovery-observation.json` and
`native-stores/`. These are operator-provided observations, not executions
repeated by this worker. Preserve the raw failed result and captures.

## Correct recovery reading and runbook instruction

ARCHITECTURE's Legacy ownership contract already permits an open legacy
landing to use its existing landing lock when that file exists and is free,
with proven box absence. `capacity::release` implements that reading;
startup reconciliation rechecks release before `decide_intent`, which also
checks the actual landing lock and canonical refs. While the old child holds
its lock, ownership attention and the open reservation are correct. After
that child finishes, this landing has supported ownership proof. Matching
canonical to the exact intended merged head then records the landing once.
This is more than canonical agreement alone.

For this restored landing, the runbook's expected post-release result is
ended/landed, resolved intent and released reservation, preserving the exact
intent execution identity, raw failed review and override. Reopening must keep
one landing record and the same canonical merged head. Do not force release,
add a recovery predicate or treat lock_contract=0 as permanent ambiguity.
The original pre-box child case remains the ambiguous owner: it lacks a
supported historical lock proof and stays reserved after its child finishes.
All other backup, isolation, restore and operator switchover instructions in
the original handoff remain in force.

The local artifact manifest previously counted only raw-string migrations and
incorrectly listed 7 current/4 historical entries. Source has 26 current/8
historical migrations, consistent with the supplied store observations. Its
manifest is corrected; the executable bytes and hash are unchanged.

## Test-audit gate and repair

The existing G5/G6/G15 representative is retained. Before release it keeps
held-child PID, old canonical ref, open intent and exact ownership attention;
it now explicitly checks that the migrated reservation is still open while
held. After release it checks the same intent ended/landed and resolved with
reserved=0, the exact intended Git head, and exactly one `landing.recorded`
audit row bound to the fixture's intent ID. It checks those values again on
a second reopen. The expected ID/head come from the old store and host Git;
there is no literal execution-5 dependency or forged store row.

Sabotage comments name refusing the supported legacy landing lock, losing
recovered intent identity or recording a duplicate landing on reopen. The
held refusal and allowed recovered landing are in the same restored project.
The original pre-box ambiguity assertions remain unchanged. No production
recovery code, fixture machine, historical build, box or model turn is added.
One extra daemon reopen checks no duplicate recovery. Shown by now describes
both the supported recovery and the retained ambiguous-owner case.

This is a fixture expectation correction, not a reproduced production defect
or a removed failing scenario. The fixture diff adds 34 and deletes 16 test
lines; production lines changed: zero. Boxed fmt, locked all-targets clippy
and locked build are the local checks. No e2e or sabotage is run in this box.
The repaired host migration gate, full Linux x86_64/arm64 suite, Mac validation
and final release readiness remain pending operator execution. Earlier build
artifacts and passing baseline gates do not establish those results.
