# Lean suite cleanup, 2026-09-29

Whole-tree code-cleanup and test-audit pass from `d004996`, on
`codex/lean-suite-2026-09-29`. The starting checkout was clean. No push,
release, installation or service change was performed. The operator's
G8 decision was answered and its assertion verified on 2026-09-30.

Three reviewers read the store/git/fixtures, jobs, box/harness/agent
environment/MCP modules and all guarantee files. The operator agent read
CLI, daemon, API, configuration, entry points, build/release files,
ARCHITECTURE.md, AGENTS.md and README.md. All 37 built CLI help pages were
read. The prior cleanup's rejected findings were checked for changed
conditions. The resulting patches were reviewed across file boundaries.

## Measurements

Release base: `v0.0.1`, commit `b196825`. Before: `d004996`. After below
means commit `29f8f9a` before this report. No guarantee or Shown-by clause
is removed.

| Measurement | Before | After |
|---|---:|---:|
| End-to-end scenarios | 75 | 47 |
| Full Mac suite, two threads | 319.31 s | 257.05 s |
| Rust and TypeScript lines in crates | 19526 | 18727 |
| Yard source lines | 12217 | 12217 |
| E2E source lines | 7309 | 6510 |
| Largest source: jobs/supervise.rs | 1490 | 1490 |
| Second largest: jobs/admit.rs | 1314 | 1314 |
| Third largest: G8 tests | 1202 | 1052 |
| ARCHITECTURE.md lines | 812 | 812 |
| Tag-to-tree additions / deletions | 1 / 1 | 652 / 1451 |

Pass source diff: +651/-1450, delete/add ratio 2.23, net -799. E2E is
+649/-1448; binary source is +2/-2, a help correction. Tag-to-tree
delete/add ratio is 1.00 before and 2.23 after, excluding this report.
The scenario reduction is 37.3%; the final elapsed reduction is 62.26
seconds, 19.5%. The draft run took 237.28 seconds, so the measured saving
varies. These warm-cache runs are not a universal timing claim. Static box counts were not substituted for measured duration.

This report adds 159 lines. Including it, this pass is
+810/-1450, delete/add ratio 1.79, net -640;
tag-to-head is +811/-1451, ratio 1.79.

## Commits and dispositions

Every retained block was compared with the spec, its outside observer,
its outside expected value and possible twins. Sabotage comments travel
with the retained mechanisms. Check/approval IDs come from public rows
only where the guarantee explicitly requires those identities to agree.

| Commit | Removed fixture or assertion; survivor |
|---|---|
| 6455010 G10 | Ordinary conflict fixture; config-changing conflict now proves renewed gates/review/approval and old approval retirement. |
| 6ecbed2 G2 | Old-head approval and old-revision edit fixtures fold into changed-head and same-head edit scenarios. Gate/seat digest changes run sequentially. Repair approval retirement moves to G10. |
| 156a962 G3 | Intent-racing edit fixture folds into held update-ref/canonical-move scenario, with another ticket's allowed edit. |
| cbc1ef4 G5 | Pre-box intent and startup refusal fixtures fold into held-worker crash/restart. Both pre-crash state and held startup remain externally observed. |
| 6ac55a1 G6 | Three no-worker configuration projects become one parked-ticket journey, retaining named refusals, no writes and corrected controls. |
| 528b8ff G7 | Separate idle-watch fixture folds into attention/watch/drain phases. Local listener/model-route observations move here from G14. |
| 2858547 G9 | Prompt boilerplate detector; actual edit diff and committed rename remain. |
| 21a3dc3 G11 | SHA length/hex detector; independently observed refused identity and no gate remain. |
| 9ee8c14 G13 | Workspace-write and route/ignored-file fixtures fold into boxed proof scenario. Host env/head/order folds into held-seat proof snapshot journey; expected head comes from Git rather than Yard's approval. |
| 02ba57b G14 | Transferred listener fixture and repeated approval/nonempty-proof probes; hostile Git and named bad-proof refusal remain. |
| 2da85ea G15 | Live-close fixture folds into the existing writable child at approval, with successful close after park/abandon. |
| 27367d6 CLI | Correct watch help to mention idle and since exits. No behavior or help-text test added. |
| c3cf2fb G8 | Second publication folds into publication/crash recovery; priority threshold and panel-none into held-seat teardown; override into max-rounds/edit; abandoned seat/gate into successful retry journeys. Three inherited-session fixtures fold into three real-client environment scenarios. Adds the approved abandoned-seat no-check observation. |
| 29f8f9a G12/main | Three credential fixtures move to those G8 real-client scenarios; orphan module is removed. |

G8 retains genuine publication in a later model turn after both rogue
requests return 404. Its first publication is durable before the crash.
Its limit journey checks two rounds, exactly one extra round from an edit,
stale gate-digest override with no writes, then successful override/landing.
Abandonment phases park their tickets before ending attempts, preventing
new admissions from racing the intended observations.

G12 still searches real Pi, Claude and Codex worker/harness/init
environments and mounted/temporary files. The host fixture observes the
actual injecting/login route's credential and Codex account headers.
Supported clients and credential mechanisms were not collapsed into one.

Kept distinct: G1 isolation; G2 held-seat cancellation, proof identity,
protected paths and policy withdrawal; G3 both crash/ref outcomes; G4
capacity and project grants; G5 host process groups; G6 malformed Claude
payload; G8 registration and linked skill root; G9 natural end versus stop,
dirty retry and session reset; G10 ordered red queue; G13 host landing
merge/edit race; G14 hostile Git and proof bounds; G15 cross-execution
proposal-key collision. Red and conflict both retire approval through
queue::returned before their separate counters, which remain tested.

## Validation

Host: macOS 27.0 build 26A428, arm64, Apple container 1.5.0, pinfold 0.0.9.
Full command: `cargo test -p e2e --locked -- --test-threads=2`.

- Unchanged baseline: 75 passed, 0 failed, 319.31 seconds.
- Independent changed groups, excluding G8/G12: 37 passed, 22 filtered,
  195.99 seconds.
- Full draft: 47 passed, 0 failed, 237.28 seconds.
- Approved G8 no-check assertion: 1 passed, 46 filtered, 12.32 seconds.
- Final full suite at 29f8f9a: 47 passed, 0 failed, 257.05 seconds.
- Each committed file passed fmt, Clippy with warnings denied and locked
  build. The final source also passed all three and git diff --check.

Earlier runs are failures, not green evidence: sandbox lacked runtime
access; two interrupted runs shared the builder with another checkout;
the first exclusive run had 74 passes and one pre-worker image failure,
`buildkit not found`, 304.95 seconds. That chat confirmed it had finished
runtime operations beforehand; the last failure's cause is unconfirmed.
No scenario was removed or assertion weakened to make it pass. No retry
was added to the harness. Runtime use was coordinated with the user's
permission. The full unchanged rerun was green before source edits.

No Linux host or CI run, service-manager installation test or mutation
experiment was performed. Tests run the daemon directly and real workers
in real pinfold boxes against a host scripted model, without public model
endpoints. Logs and per-commit check evidence are in /private/tmp:
yard-mac-green-baseline-2026-09-29.log,
yard-mac-lean-independent-2026-09-29.log,
yard-mac-lean-full-2026-09-29.log, yard-mac-g08-no-check-2026-09-30.log,
yard-mac-lean-final-2026-09-30.log and yard-lean-commit-checks.log.

## Operator decision

G8 promises that a seat ending after abandonment records no
check. Its original scenario observed outcome and attention only. The
approved additional assertion queries checks joined to executions of the
held second attempt. Expected empty set comes from G8; observer is the
read-only store. No new scenario or binary seam was needed. The operator
approved adding the assertion and keeping the promise on 2026-09-30.
test-audit required that decision for every unshown clause. No spec
promise was removed or weakened; the coverage gap is now closed.

## Mechanical checks, docs and rejected work

Twelve Yard dependencies match the spec; e2e's rusqlite and serde_json
serve outside observation and fixture messages. Three lint suppressions
explain their independent execution inputs. No TODO/FIXME/XXX or fired
revisit trigger was found. Git/pinfold spawn through their controlled
modules with cwd, scrubbed environment and output/time bounds.
The prior ps-output byte-cap and service-helper boundary findings remain
operator rework proposals; this pass supplies no reproduced new failure.

No new rework ticket is admitted. Existing shared prompt/model-validation
proposal remains: remove duplicate validators while preserving byte
limits, Pi's @ refusal and distinct session rules. Do not introduce a
wrapper merely to rename independent inputs. It was not filed again.

Rejected, with re-admission conditions: timestamp replacement needs a
second format or store rework; cleanup-derived state needs a second
cleanup outcome; inferred audit targets need a guarantee requiring that
target change; init/fetch cloning needs an otherwise required clone change;
argument structs need actual shared state removed. Approval rereads,
reconcile ordering, no-follow agent environment writes and proof bounds
remain controls. Sibling proposals stay separate unless a simpler fixture
retains cross-execution collision without conditional script bookkeeping.

The spec and README pass found no resolved open question or outdated
contract needing removal. Shown-by clauses remain valid across the new
survivors; the G8 gap is closed. No feature or control is removed.
