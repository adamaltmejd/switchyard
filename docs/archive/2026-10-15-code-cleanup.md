# Y80 release cleanup and test-audit disposition

2026-10-15. This records the operator's final audit disposition against
`8cbcef7768edea75d394a7324c19232709048f3f` and its implementation through
`67152fa`. It is not host release approval. The operator supplied a whole-tree
baseline read by three read-only reviewers at `f172a476`, refreshed Y133 at
`8cbcef`, and source-bound ledgers for all 21 assertion files. This worker
refreshed the changed source modules, fixtures and living docs; it does not
claim a second independent whole-tree audit. The authorized dispositions are
the scope of this pass. Product controls and dependencies stay exact.

## Measurements

Counts are `git diff --numstat` or `--shortstat`, not elapsed savings. Metrics
end at `67152fa`; this evidence file itself is excluded. All-path counts include
tracked docs, archives, skills, runner assets, configuration and lockfiles.
The narrower source/test rows include only the respective `crates/yard` and
`crates/e2e` trees. No `.yard` file changed in Y80. Binary artifacts and private
proofs outside Git are excluded. Delete/add means deleted lines / added lines.

No tag refs are present in this clone. The operator identified source
`81da8c677a0ab43966588429c24a8affedb9e8ec` as v0.0.3; that source declares
0.0.3. Its release comparison is explicitly source-based. The skill's literal
no-tag fallback is root `a482f4ebea20d3a457e405c26f02c0b682cf6253`.

| Scope | Added | Deleted | Delete/add |
|---|---:|---:|---:|
| Y80 all tracked paths from 8cbcef | 593 | 460 | 0.776 |
| Y80 production | 35 | 50 | 1.429 |
| Y80 tests/harness | 366 | 341 | 0.932 |
| Release all paths from verified 0.0.3 source | 58822 | 4137 | 0.070 |
| Release production from that source | 16685 | 1782 | 0.107 |
| Release tests/harness from that source | 10915 | 1899 | 0.174 |
| No-tag fallback, all paths from root | 77088 | 356 | 0.00462 |
| No-tag fallback, crates from root | 42334 | 0 | 0 |

Before this pass, the root comparison was 535 files, 76952 additions and
353 deletions; after, 536 files, 77088 additions and 356 deletions. The 0.0.3
source comparison changed from 506 files, 58655 additions and 4103 deletions
to 507 files, 58822 additions and 4137 deletions. These aggregate release
figures include the redesign, not just cleanup. The migration fixture adds
required restore and legacy-effect evidence; deletion is not a quota.

Largest production source remains `crates/yard/src/jobs/admit.rs`, from
3309 to 3305 lines. `supervise.rs` is 2989 lines (previously 2987).
ARCHITECTURE.md is 2614 lines (previously 2603). The current tracked Rust/TS
source total is 42013 lines, including e2e.

## Landed source and docs

- `e5b531d`: remove unused `Run.session_id` and duplicate transcript location
  decisions. The persisted worker session remains. Existing
  `supervise::transcript` is authoritative for ordinary and scoped writers,
  inspection and tail: scope present or classification reason uses effect
  storage; other attempts use ordinary storage. Command classification still
  has no transcript. This corrects Y78's actual write/read mismatch without a
  fallback archive, migration or lifecycle.
- `5fff5e0`: remove unnecessary seven-argument classifier lint suppressions;
  no signature rewrite or threshold change.
- `99f5a6b`: remove a disjoint second JSON object/merge and admitted-input
  serialize/as_object round trip. Typed node input fields preserve producer
  qualification, nulls, decoder errors, historical/concurrent/handoff paths.
  The larger single JSON macro requires recursion limit 256 at compile time.
  No runtime performance gain is claimed. Strict JSON decoder consolidation
  was already landed and was not recreated.
- `67152fa`: 0.0.5 metadata and migration/switchover handoff; README describes
  isolated agents and one verified queue. Cut repeated failed-evidence and
  Durable contract prose while retaining actionable setup, selection,
  source-write/commit, unsafe replay, publication and landing-omission steps.
  Remove speculative library/extension open questions. Update Shown by for
  actual retained assertions and real 0.0.3 fixtures. The release section now
  states successor/platform/backup/operator boundaries rather than an older
  direct-API patch rollout. No supported behavior is removed.

## Test-file decisions

Each changed file has one commit. All dispositions below are coverage choices,
not relaxed controls. No removed scenario was a retained failing product
result. No replacement fixture restores deliberately omitted variations.

| Commit/file | Disposition and retained hard case |
|---|---|
| 398313c, g02_judgments_bind_identity | Delete separate manual-policy flip; retain held protected-path withdrawal and explicit manual approval/landing. Delete only closed_inputs-token-only adoption; keep actual command/image, unknown consumers, policy/tree/proof, immutable history and failed-definition restart/checked release. Rebase counts on changed command. Add host transcript vs actual operator tail/stats observation to the existing completed agent classifier, while its attempt remains live; no new box/turn. |
| c346581, g03_landing_cas | Refine held recovery attention to ownership reason and the fixture's actual intent execution ID. Keep existing release/start control; no new phase. |
| b5dbdcd, g04_capacity_holds | Initial foreign/blank/duplicate refusal trio remains once, with valid subset/combined controls. Later three pickers retain their distinct historical-source refusal and valid publication. Remove nine duplicate request/reply turns. Preserve all Y133 exact provenance/application/outcome/restart observations and the entire second simultaneously interrupted writable writer phase. |
| da8b053, g07_operator_mcp | Delete stale edit/head twins now exercised through the same operator MCP in G2. Keep valid edit as unknown-field refusal control, held cancellation/EOF, worker denial, trusted park, external candidate and exact approval/landing. No machine/box/model saving claimed for these line cuts. |
| b586561, g08_audit_reports | Delete native terminal-only missing-audit machine; missing review publication remains the stronger false-approval risk. Remove only wrong-session curl turn/status assertion in held report case; retain invalid/valid/duplicate publication, frozen source and held eligibility. Session controls remain in G1/G7/hostile review. |
| 0ef7fcb, g08_durable_package | Delete successful read-only continuation machine; specialize corrupt-checkpoint/fresh retry. Remove false-only inode/latch/fixture-marker branches. Preserve resumed-N unexpected-interruption marker and absence assertions, debugger-held unsafe intent, external old-box absence, corruption refusal/fresh control and immutable usage/history. Entire selected writable unsafe cleanup/no-replay, clone/checkpoint, boxed Git guards, report-only handoff, repair and native landing remains. |
| 2133ef2, g08_review_is_a_publication | Delete separate Pi stripped-registration machine; retain real Claude API unregistered publication and same-head proven retry. Delete final API-to-login extra ticket. Keep dedicated Pi/Claude-login/Codex hostile-source/secret and allowed controls; no adapter changes. |
| b6a9a24, g15_plans_and_proposals | Replace shared historical 0.0.4 build with verified genuine 0.0.3. Add stopped-writer snapshot and one separate old restore machine, old override/held landing proof, successor conservative ownership stop/reopen. Preserve limit target binding, ended unknown planner target, omitted child target/edge, repeated accept no-duplicate, raw old rows/audit and exact landing. Keep original pre-box FIFO child/PID/reservation/capacity waiter. Delete unsupported no-handle stop secondary block and stop/restart overclaims. Narrow planner comment to real dependencies/explicit close. |

Both integration conflict scenarios remain: real dirty/unmerged source refusal
with same-tree resolution, and partial A publication despite unresolved B.
No unknown-owner fail-safe, unsafe-write no-replay, source isolation, raw loss,
foreign Git/secret guard or supported legacy recovery was deleted.

Deliberately untested secondary variations are the separate manual approval
policy flip, token-only closed_inputs adoption, native audit terminal-only
completion, successful read-only Durable continuation and its resumed numeric
variation, stripped Pi registration negative, exact API-to-login round trip,
and legacy no-handle stop refusal. Their product contracts remain exact.

### Gate for strengthened assertions

The material risks are G3/G5 wrong-owner recovery, G5/G6/G8/G15 lost historical
identity/decision or duplicate effect, and G7 false artifact unavailability.
The first uses the existing held landing and externally observed intent ID.
Migration uses the real old executable, scripted real worker, host Git/PID,
old read-only store and independent snapshot, followed by supported successor
MCP. Expected rows/edges/heads/checks come from those external sources; no
store seeding or private binary seam is introduced. The sole added machine
is required for old restore without reopening upgraded state. Historical
build is shared. Classifier observation uses existing work and compares actual
host bytes with supported tail/stats; no new fixture or model turn.

Comments name the corresponding sabotages: scope-only transcript selection,
wrong owner attention, dropped historical columns/target binding, fabricated
ended target, lost rows/audit, or replay/release of an unproved legacy intent.
These sabotages and fixture runtime were not run in this box. They require
operator host validation; source review and clippy do not prove a failing
sabotage. Refusals retain their same-project allowed controls.

`g07_usage.rs` is unchanged. Its real Pi fixture supplies reported input 700,
cache reads 200 and writes 100 for expected input 1000, and retains unknown,
interrupted, resumed explicit-zero, settled cleanup and reopen controls. The
operator carry-in reports nonzero-cache saboteurs failed and restored pass;
this worker did not repeat or newly claim those executions.

## Whole-suite work and execution evidence

Static source inventory is 47 test functions and 49 fixture-machine
invocations, including nested five-writer and separate restore machines.
Baseline was 51 functions and 52 machine invocations (two helpers each ran
twice). Approved cuts remove four machines; restore adds one. Runtime is not
inferred from that count. The retained suite has one shared current build,
one shared genuine 0.0.3 build and one shared current-source debugger build
for the two retained Durable checkpoint scenarios. There is no version matrix.

The admitted static reductions are three G8 machines/nine worker boxes/18
scripted responses; the G2 manual variant removes one machine/four boxed
executions/two host-gate children/two scripted replies. The token-only phase
removes two command/reference gate boxes. G4 removes nine repeated invalid
publication exchanges while retaining four pickers; held audit removes one
additional raw curl model exchange. The restore machine adds no worker box
or model response; held legacy landing adds native Git orchestration. These
are static workload changes, not measured time savings or a full remaining
box/model total. Actual launches/turns, platform wall times and the full
remaining runtime cost must be recorded by the exact-candidate host gate.
This session launched zero e2e machines/boxes/model requests.

The operator's prior clean 8cbcef native gate passed 51/51 in 786.04 seconds,
raw SHA256 `316bd56595020a37795075a0e7cb9814215e6bfeab1ce9026006377eeb453aa1`.
That is a baseline, not a result for this candidate. Prior failed CI/native
results remain evidence; none was deleted or converted to a pass here.
The approximate five-minute landing budget is not established for this
candidate, and the baseline exceeded it. No elapsed savings are claimed.

Boxed checks passed: `cargo fmt --check`,
`cargo clippy --all-targets --locked -- -D warnings`, and locked builds.
The final optimized GNU build took 1m42s, including compilation in this box;
that is build output timing, not host e2e timing. The worker was explicitly
instructed not to run e2e. Linux x86_64/arm64 runtime, old restore/migration
execution, Mac runtime/security and final release-asset checks remain pending
operator work. A registered online Mac runner alone establishes none of them.

The staged local GNU candidate is version 0.0.5, boundary
`0767f01e8d4708ec`, SHA256
`0767f01e8d4708ecfc5a280aa6eb08c8df67eaee1693bf2213ba8fda0304e903`.
It was built with Rust 1.99.0 (b940084d7, LLVM 23.1.1). The executable and
manifest live under `/yard/proof/y80-artifacts`; static Linux and Mac assets
remain to be built/validated. See the separate dated migration/switchover
handoff for stopped-writer backup, restore and operator actions.

## Rejected work and operator disposition

The operator admitted the exact source corrections, cuts and docs pass above.
No new ticket or dependency was created. Mechanical dependency/suppression,
child-process and deferral review produced no additional admitted work.
Existing dependency lists match the spec; remaining supervise argument-count
suppressions express existing intent. No runtime performance claim is made.

Rejected shared harness validation, generic scheduler abstraction,
service-child/ps rewrites and hypothetical performance work stay rejected.
Re-admission requires a supported-seam reproduction or a concrete module
consolidation naming what it removes, with the operator deciding any touched
control. Hypothetical eight-box integration savings are rejected because they
remove distinct dirty-tree/wrong-candidate evidence. No replacement coverage
matrix or prerequisite cascade was created.

The operator's supplied ARCHITECTURE/README disposition is implemented here.
Fresh operator review of the resulting docs and exact host/platform evidence
is still required before release. Construction remains on its frozen binary;
final installation, config commit/sync, daemon replacement, publication and
push are operator decisions after readiness.
