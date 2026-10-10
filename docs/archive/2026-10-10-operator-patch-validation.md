# Operator patch validation, 2026-10-10

The operator resumed at `66a81dd`, with Y-54 done and all later redesign
work parked. Y-55 was unparked through the frozen 0.0.3 CLI. This report
records preparation evidence, not permission to publish or switch runtime.

## Cleanup and assertion audit

Three read-only reviewers covered all jobs, Git, boxes, store, core,
harnesses, fixture sources, manifests and workflows. They read binding
ARCHITECTURE.md and AGENTS.md and the latest cleanup rejection conditions.
The operator read all built help pages, README, guarantee rows and G1–G6;
the reviewers read G7–G15 whole and compared cross-file mechanisms.
The operator judged findings against their source and callers.

The operator ran dependency trees, suppression, child-spawn and deferral
searches. Twelve Yard dependencies match the spec; the two e2e dependencies
serve outside observation. Suppressions explain independent execution
inputs. Git and pinfold remain centralized. No fired deferral was found.

`f96b111` removes the duplicate Pi one-file commit shell template in the
fixture module. It uses the existing Claude/Codex shared template. No
assertion changes. Fmt, locked all-targets clippy, locked build and all 37
real host scenarios passed. Full suite: 154.24 seconds. Log:
`/tmp/switchyard-y55-fixture-cleanup-e2e.log`.

`be3694f` corrects G9's sole no-intermediate-gate assertion. The expected
head comes from the worker clone's Git and its `renamed.txt` fixture content,
instead of Yard's approval output. The block names the preceding-head
sabotage. No clause is removed. Fmt, locked all-targets clippy and build
passed. The focused real host scenario passed in 20.83 seconds; log:
`/tmp/switchyard-y55-g09-audit.log`.

G7 implementation statistics establish the shared worker reader/store
mechanism. Review calls the same reader and store update with no role
predicate or later overwrite. A second review-statistics block would be
a twin. Registration, proof isolation, repair identity and worker routes
exercise distinct mechanisms across files and are retained.

The G10 serialization doubt was checked in an isolated checkout and target
directory. The mutation permits two live landing slots and skips an approval
already landing, so it can overlap distinct candidates. The scenario failed
at the overlap assertion, showing starts of executions 3 and 5 before either
ended. It failed in 26.19 seconds, not at a timeout. Log:
`/tmp/switchyard-y55-g10-distinct-slots.log`. The block stays. The mutation
was reverted. Two earlier probes that allowed duplicate launches of one
approval are excluded from this proof. One also lost its development
client/daemon pairing during a rebuild. They are not product test results.

The G7 ignore-since mutation failed at the drain attention assertion in
11.59 seconds, proving that shared since filter is exercised. The distinct
stale-drain mutation (count any old attempt end) passed the scenario in
18.81 seconds. That confirms the idle-board observation gap. Logs:
`/tmp/switchyard-y55-g07-ignore-since.log` and
`/tmp/switchyard-y55-g07-stale-drain.log`. Both mutations were reverted.
The two waiting observations remain pending final observer migration. They may first sample after the next mutation. Parked Y-79 was
edited to revision 4 to carry both clauses into supported real MCP/API
observation without clocks, retries or binary barriers. No guarantee was
weakened. This is not a claim of a cleared final release audit.

Static MCP argument-validation leads remain investigations, not admitted
bug tickets. Re-admit only after a real worker/CLI-seam reproduction:
`yard_context` discards arguments; missing `yard_progress.note` indexes
without an explicit missing-key refusal. No control reduction is proposed.
Other historical rejection conditions remain unchanged.

Audit source cleanup: +1/-3, delete/add ratio 3.00, net -2.
Assertion correction: +5/-1. Before/after total Rust/TypeScript source:
18557/18559 lines. Largest source remains jobs/supervise.rs, 1499 lines.
The operator's initial architecture has 826 lines; Y-55's reviewed release
contract has 843. Final redesign metrics require another whole-tree pass.

## Exact patch artifact

The protected Y-55 diff was read against its ticket. It changes version
metadata, README setup, the release contract and dated preparation records.
DeepSeek correctness check 579 passed without findings; boxed fmt, clippy
and build checks 576–578 passed. Approval 67 binds candidate
`3b886aa825a0cfea0032112e147d739ac62b2fe8` and proof
`a1a96f5077ccfbca312f39e5b5ef443d4faaca97982da5d1129c76232d0f5622`.
The release checklist distinguishes preparation from final cross-platform
gates and publication. The operator retains those pending conditions.

The exact worker-snapshotted x86_64 GNU 0.0.4 binary was copied into an
isolated release e2e sibling directory. The real direct Anthropic API fixture
passed in 28.34 seconds. It proves guest key absence, host injecting headers,
registration-proof eligibility and same-project subscription control.
Before and after SHA-256:
`6191a200de87bfef1b3cbddb95ed00e0c582ec23bd06fc61ad8b1f944851c0c8`.
Log: `/tmp/switchyard-y55-exact-artifact-fixture.log`.

The tested GNU artifact and a provisional checksum file are retained under
`target/release-prepared/0.0.4/`. This is not the static musl release asset
set, and no cross-target execution result is claimed.

## Isolated schema compatibility

The frozen binary created a fresh registered scratch project with a done
ticket, a parked dependent and audit history. Its writer stopped before
project/store and registry backup. Socket files were excluded from backup.
The exact patch artifact reopened the fixture twice. External SQLite dumps
of ticket, dependency and audit rows, schema 8, and Git refs were unchanged.
The frozen binary reopened a separate restored backup with the same facts.
No old binary opened the patch-tested state.

Fixture: `/tmp/switchyard-y55-schema-compat-20261010`.
Result: `/tmp/switchyard-y55-schema-reopen.log`.
This narrow unchanged-schema experiment does not establish successor graph
migration or live-effect recovery; Y-80 must prove those separately.

## Construction boundary

Operational PID 965905 and boundary `05ae25832964703c` remain unchanged.
Operational binary SHA-256:
`05ae25832964703c81525e7c5f5be7a8148954517cfddf69ef9846953bf1ebb2`.
Construction remains Sol 6.1 medium with `.codex/config.toml` requesting
`service_tier = "fast"`, and DeepSeek v4.1 Flash high via Opencode Go.
No push, publication, installation, reviewer switch or daemon replacement
was performed. Y-81 remains deferred and parked. Final Linux arm64 and Mac
host validation needs evidence from those hosts; the old Mac waiver is not
reused.

## Landing and next work

Y-55 landed at audit 3801, merged ref
`0ff8e94a3d9399511be2e3b57767fd44bd9108eb`. Completion is 3802 and checked
cleanup is 3805. Landing fmt, clippy, build and host e2e passed. Host gate
1100 passed all 37 scenarios in 139.99 seconds, including both operator
audit commits. The checkout consumed the landing at 3806.

The release diff from local tag v0.0.3, excluding dated archive reports
and operator .yard, is +249/-41; delete/add ratio 0.1647. Total source is
18559 lines; largest source is 1499 lines and architecture is 843 lines.
These are patch measurements, not final redesign release measurements.

Named branch sync Y-82 is the next dependency-ready construction ticket.
Y-56 waits for it; Y-83 and Y-58 then follow target binding. Later work
continues through Y-80. Y-81 remains parked. No final release or final
redesign completion is claimed.
