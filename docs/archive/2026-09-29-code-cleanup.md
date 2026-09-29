# Pre-release cleanup and test audit, 2026-09-29

Whole-tree pass from `800f311`, fetched `origin/main`, on
`codex/pre-release-cleanup-2026-09-29`. PR #2 was opened as a draft after the
first commit. No push to main, squash, force-push or merge was performed.
This report is committed last.

Three read-only reviewers covered jobs, core modules and the e2e suite.
The operator agent reviewed CLI, daemon, configuration, user docs and the
merged findings. The trace covered configuration, commands/tools, emitted
reasons/events and every guarantee's Shown-by clauses. The previous pass's
rejected list was read first. All 37 CLI help pages were read from the built
binary. Review of the resulting changes found no additional defect.

## Measurements

There is no tag. The release base is root commit
`a482f4ebea20d3a457e405c26f02c0b682cf6253`. Before is `800f311`;
after includes this report. The report contribution is separated below.

| Measurement | Before | After |
|---|---:|---:|
| Root-to-head additions / deletions | 22405 / 264 | 22755 / 267 |
| Rust and TypeScript lines in crates | 19415 | 19363 |
| Largest source: jobs/supervise.rs | 1490 | 1490 |
| Second largest: jobs/admit.rs | 1310 | 1307 |
| Third largest: G8 test file | 1219 | 1202 |
| ARCHITECTURE.md lines | 806 | 805 |
| Direct dependencies removed | 0 | 0 |

Root-to-head delete/add ratio: 0.0118 before, 0.0117 after.

Pass excluding this report: +137/-190, delete/add ratio 1.39, net -53.
This report adds 400 lines. Including it: +537/-190,
delete/add ratio 0.35, net +347.

Source alone is +129/-181, delete/add ratio 1.40, net -52. Binary source is
+13/-36; e2e source is +116/-145. Guarantee test files alone are +82/-61:
coverage repairs grew tests while fixture duplication came out. These are
net diff counts from the pass base, not summed commit churn.

## Commits

| Commit | Removed or repaired | Guarantee mechanism |
|---|---|---|
| f3369a3 cli | Duplicate string-list conversion; reuse names | None; rendering only |
| 22fadcd admit | RefCell around proposal stop list; transaction accepts FnOnce | G2/G15 proposal dispatch; same stop list and transaction |
| 65e9dd8 harness | Three pinned-version readers; trait default preserves lookup | Session continuity's version input; identical lookup/fallback |
| 4de96ef store | Creation audit-text parameter always set to None | G7 audit data unchanged; proposal text remains on attention resolution |
| bd1ba12 G3 | Bare refusal-code probe and stale comment | Test only; held intent, red item and exactly-once recovery remain |
| 76e604a G13 | Operating-system error-prose assertion | Test only; failed write status, payload and writable control remain |
| 4ea2408 model | Two SSE formatters duplicating existing sse | Fixture only |
| 6ca4ef6 harness | Duplicate config and shell-command templates | Fixture only; byte-identical outputs |
| e89cd85 README | Release link to the previous repository | None |
| 6d11b75 G5 | Unobserved timestamp claim; crash-state observation now precedes restart | Test/spec only; invariant unchanged |
| 8d3b1aa MCP | Duplicate tools/list connection writer | G8 registration; initialize is authoritative |
| a53d513 G7 | Execution/check audit coverage gap | Test only; exact event identities and cardinality |
| c06bfb8 G8 | Exit-shape and round-limit prose probes; separate control project | Test only; same-project linked-skill control |
| 3f2f186 G14 | Unnamed malformed-candidate check and symbolic-link prose | Test only; fixture-reported corrupt OID must appear in refusal |
| a2d7a3b docs | Resolved Codex question and contradictory CLI wording | Read-only discovery exception; no capability change |
| 09458be G8 | Wrong Codex message selection in the rewritten fixture | Test only; choose linked script from the actual ticket brief |
| Final report commit | Records decisions, measurements and validation | None |

Each commit passed `cargo fmt --check`,
`cargo clippy --all-targets --locked -- -D warnings` and `cargo build --locked`.
The baseline passed the same checks. Binary and fixture consolidation commits
changed no test assertion. Scratch Rust comparisons checked old/new config
and command bytes for empty, quoted, multiline and Unicode inputs; they were
not promoted to permanent tests.

## Mac and CI validation

The initial instruction waived local e2e for a runtime-less VM. The operator
then explicitly required thorough Mac validation. Live inspection found this
host is Darwin 27.0.0 arm64, macOS 27.0 build 26A428, with Apple container
1.4.1 running and pinfold 0.0.9. The profile image exists. Pinfold's harness
pins are Pi 0.87.1, Claude 2.1.283 and Codex 0.158.0.

- Full suite at e89cd85: 74 passed, 0 failed, 0 ignored, 0 filtered out;
  461.55 seconds (7m 42s), exit 0, no retries.
- Focused G5 after repair: 4 passed in 18.33 seconds.
- Focused G7: 2 passed in 18.79 seconds.
- Initial focused G8: 18 passed, 1 failed. The new same-project fixture
  searched Codex's first user message for the ticket title, so both tickets
  created regular skills. The test correctly rejected the unexpected approval.
  Selecting last_user fixed the fixture; the linked-skill test passed in
  7.28 seconds. No guarantee assertion was weakened or deleted to fix it.
- Focused G14: 3 passed in 19.10 seconds.
- Final full suite at 09458be: 74 passed, 0 failed, 0 ignored, 0 filtered out;
  377.64 seconds (6m 18s), exit 0, no retries.
- Linux CI at 09458be: x86_64 and arm64 both passed in
  [run 36588012761](https://github.com/adamaltmejd/switchyard-v3/actions/runs/36588012761).
  See the PR checks for the final report-only push.

Mac command: `cargo test -p e2e --locked -- --test-threads=2`. Two scenario
threads bound concurrent VMs without serializing races inside a scenario.
The full run exercises G1–G15, including native flock across daemon death,
ps process birth identity, killpg, FIFOs, lsof, all three real harnesses,
credentials and sessions, host/boxed gates, proof mounts, queue races and
proposals. G12's /proc reads occur inside Linux boxes, not on the Mac host.

Scope limit: tests launch daemon run directly with the caller's PATH. They
do not exercise launchd install/uninstall/restart. The generated plist sets
no PATH while the daemon resolves pinfold through its environment; installed
service behavior remains unverified. No existing service was changed.
Deferred sabotages below remain separate, unrun experiments.

Local evidence: `/private/tmp/yard-macos-e2e-2026-09-29.log`,
`yard-macos-g05-cleanup.log`, `yard-macos-focused-cleanup.log`,
`yard-macos-focused-repair.log` and `yard-macos-final-cleanup.log` in the same
directory. These temporary logs are not repository artifacts.

## Mechanical checks and spec pass

- Yard's twelve normal dependencies match the Code section. E2e uses only
  rusqlite and serde_json for external observation and fixture messages.
- Three too_many_arguments suppressions in supervise.rs explain their
  independent execution inputs. No TODO/FIXME/XXX or fired revisit trigger.
- Git/pinfold spawn only in git.rs/box.rs, with cwd, scrubbed environment,
  output bounds and time bounds. Host gates retain their cwd, scrubbed env,
  lock, handle-before-command ordering, capped output pipe and timeout.
- The ps identity probe has cwd, scrubbed env and ten-second timeout, but
  output() has no byte cap: a checklist gap, not a reproduced failure.
- Service-manager helpers remain outside the literal git/pinfold rule.
  The previous operator pass retained install/uninstall without full child
  bounds. Restart/status add time bounds but inherit cwd/environment, and
  status collects uncapped output. No control was silently weakened.
- Audit events, attention kinds/exits, implementer reasons, configuration
  fields and worker grants match the spec vocabulary.
- Fixed README's old repository link, removed the resolved Codex-return
  question, and reconciled the CLI's read-only discovery contract. Remaining
  replay/batching exclusions and open questions were retained.

## Test sweep dispositions

All fifteen guarantee files and both fixture files were read. Five assertion
blocks were removed; no test function or guarantee clause was deleted:

- G3 bare refused code: the previous pass already cut its Shown-by clause.
  Held intent, red item, canonical effect and exactly-once recovery survive.
- G8 advertised approve exit: subsumed by actual successful approve/landing.
- G8 round-limit prose: extra synced-limit scenario absent from the row.
  Gate-digest stale result, no-write observation and override remain.
- G13 filesystem prose: failed write, read payload and writable control remain.
- G14 symbolic-link prose: offending entry name, refused outcome, gate
  exclusion and regular-file positive control remain.

G5 now describes its held-before-effect observation accurately and proves the
crash state before restarting. G7 compares execution/check rows with their
exact audit events after cleanup while another attempt is live. Agreement is
explicitly required by G7, so these expected identities may come from public
store rows. G8's linked skill and regular-directory control now share one
project and unchanged canonical. G14's expected corrupt OID comes from Git
output observed by the model fixture, independently of Yard's refusal.

Retained cross-file cases: G2/G10 repair judgments use different entry paths;
G8's three sessions use different real clients; G12's credentials use different
routes; G13/G15 read-only checks exercise different mounts and access decisions.
No whole scenario was safely deletable as a twin.

## Operator decisions and answers

The operator approved recommendations 1 and 3–7, then approved investigating
and landing 2 during this pass rather than deferring it. All are resolved:

1. Remove obsolete Codex-return question: approved, done.
2. Remove duplicate MCP registration writer if sound: approved, done.
   Only initialize creates the random session ID. It marks the connection
   before returning that ID; every later tools/list requires it. The removed
   writer cannot add a valid connection absent from initialize. Revocation
   remains unchanged; Pi/Claude's tool-list proofs and Codex's terminal
   connection check remain intact. Spec and implementation changed together.
3. Describe G5's held box-up observation instead of claiming timestamps:
   approved, done; invariant unchanged.
4. Complete G7 execution/check audit cardinality: approved, done. A duplicate
   reviewer-statistics block was withdrawn because both workers share a writer.
5. Same-project G8 linked-skill positive control: approved, done.
6. Assert G14's malformed-candidate identity: approved, done.
7. Document existing CLI read-only discovery exception: approved, done.
   CLI store access and repository mutation remain forbidden.

## Tickets to file

These are proposals, not filed tickets or claimed CLI reproductions.

- **Validate yard_context arguments (G6).** Its dispatch ignores arguments
  while its schema disallows extra properties. Reproduce malformed input
  through a real harness with a successful empty-object control; remove the
  validation bypass if confirmed. No new guarantee is needed.
- **Consolidate shared harness validation.** Remove duplicate model/prompt
  validators while retaining Pi's @ refusal and harness-specific session
  rules. G6 control review is required; this is a rework, not an incidental cut.
- **Resolve child-process checklist scope.** Decide whether ps and service
  helpers require the skill's all-spawn bounds, or align the stated scope.
  Preserve controls; no output-exhaustion failure was reproduced.
- **Verify installed Mac service behavior.** Run launchd install/restart in
  an isolated host account, including discovery of pinfold/git from service
  PATH. The direct-daemon suite does not establish this release boundary.

## Rejected findings and re-admission conditions

- Drop resolve_reference's Y- precheck: bare numbers also name proposal
  siblings. Re-admit only after an explicit unambiguous reference contract.
- Drop queue approval rereads or combine reconcile loops: no new evidence.
  Re-admit with a reproduced redundant-read defect or a third recovery loop.
- Replace supervise's argument lists with structs: removes no concept.
  Re-admit when a common object removes actual duplicated state.
- Remove a proof-walk bound: sibling materialization and recursive totals
  are different bounds. Re-admit if another layer enforces both.
- Add reviewer statistics assertions: same worker-statistics mechanism.
  Re-admit with a separate writer or reviewer-specific failure.
- Other previous rejections stand; their conditions did not change. No
  deadline, Latch or last_tool_result change. G9 stop's previously recorded
  deadline-only sabotage remains a known limitation.

## Deferred host sabotages

These patches are experiments only. Never commit them. Apply one at a time to a clean host checkout matching this branch, run its named test, save the outcome, then reverse that patch before the next experiment. All five patches passed `git apply --check` against final source `09458be`; none was applied and none was compiled or executed here.

### G7 check audit cardinality

Removes check.recorded events while preserving checks. At 09458be the test includes exact identity/cardinality assertions. Expect a direct failure at the check-to-event equality assertion because check rows remain while their matching audit events are absent. This expected outcome has not been run. The retained execution lookup preserves its existing error boundary and avoids an unused-variable warning.

Save the following diff as `/private/tmp/yard-sabotage-g7-audit.patch`

```sh
git apply --check /private/tmp/yard-sabotage-g7-audit.patch
git apply /private/tmp/yard-sabotage-g7-audit.patch
cargo test -p e2e --locked a_ticket_lands_end_to_end_and_leaves_only_rows
git apply --reverse /private/tmp/yard-sabotage-g7-audit.patch
```

```diff
--- a/crates/yard/src/store/checks.rs
+++ b/crates/yard/src/store/checks.rs
@@ -128,25 +128,12 @@
 }

 pub fn record(tx: &Connection, record: Record) -> Result<i64, Fail> {
-    let row = super::executions::get(tx, record.execution)?;
+    super::executions::get(tx, record.execution)?;
     tx.execute(
         "INSERT INTO \"check\" (execution, kind, verdict, created_at) VALUES (?1, ?2, ?3, ?4)",
         params![record.execution, record.kind, record.verdict, now()],
     )?;
     let id = tx.last_insert_rowid();
-    audit(
-        tx,
-        "check.recorded",
-        Target {
-            ticket: Some(record.ticket),
-            attempt: Some(row.attempt),
-            execution: Some(record.execution),
-            attention: None,
-        },
-        None,
-        json!({ "check": id, "kind": record.kind, "name": row.name, "verdict": record.verdict,
-                "head": row.head.unwrap_or_default(), "proof": row.proof.unwrap_or_default(), "round": row.round }),
-    )?;
     Ok(id)
 }

```

### G7 watch --since

Expected failure: either the still-running assertion or the later answer assertion expecting both Y-1 and Y-2. The separate status round trip does not synchronize the spawned watch request, so the observed outcome remains unverified. Do not add a sleep to settle it.

Save the following diff as `/private/tmp/yard-sabotage-g7-since.patch`

```sh
git apply --check /private/tmp/yard-sabotage-g7-since.patch
git apply /private/tmp/yard-sabotage-g7-since.patch
cargo test -p e2e --locked status_watch_returns_on_open_attention
git apply --reverse /private/tmp/yard-sabotage-g7-since.patch
```

```diff
--- a/crates/yard/src/daemon.rs
+++ b/crates/yard/src/daemon.rs
@@ -408,7 +408,7 @@
 /// until one is open, or with `since` until one raised after it is.
 async fn attention(daemon: &Daemon, params: &Value) -> Result<Value, Fail> {
     let project = project(daemon, params)?;
-    let since = params["since"].as_i64();
+    let since = None::<i64>;
     let deadline = tokio::time::Instant::now() + Duration::from_secs(25);
     loop {
         let notified = project.events.notified();
```

### G13 approval reread after merging

Skips both approval checks in queue::admit after the held merge, leaving queue::next intact. The superseded second candidate should run a host gate and fail the assertion that exactly one gate ran. This outcome has not been run on the host.

Save the following diff as `/private/tmp/yard-sabotage-g13-approval.patch`

```sh
git apply --check /private/tmp/yard-sabotage-g13-approval.patch
git apply /private/tmp/yard-sabotage-g13-approval.patch
cargo test -p e2e --locked a_host_landing_gate_runs_on_the_merged_ref_only_when_approved
git apply --reverse /private/tmp/yard-sabotage-g13-approval.patch
```

```diff
--- a/crates/yard/src/jobs/queue.rs
+++ b/crates/yard/src/jobs/queue.rs
@@ -218,7 +218,7 @@
             attempt,
         ))
     })?;
-    let lapse = holds(daemon, project, &loaded, &approval, &attempt, &ticket).await?;
+    let lapse = None;
     let now = Current {
         loaded,
         approval,
@@ -226,14 +226,7 @@
         ticket,
     };
     let admitted = project.tx(|tx| {
-        let unchanged = checks::approval(tx, now.approval.id)?.state == now.approval.state
-            && !attempts::superseded_by_edit(
-                tx,
-                now.attempt.id,
-                now.ticket.id,
-                now.approval.ticket_revision,
-            )?
-            && attempts::get(tx, now.attempt.id)?.state == now.attempt.state;
+        let unchanged = true;
         match lapse {
             Some(lapse) => Ok(Err(lapse)),
             None if !unchanged => Ok(Err(Lapse::Superseded("the approval changed"))),
```

### G14 proof entry bound

Allows the oversized fixture proof. The current test may wait for its second stopped item until the harness deadline, because it does not stop on the unexpected second approval. That is an unverified test failure, never a pass. Record whether it fails at a direct assertion or only the deadline; the latter warrants a fail-fast test fix before claiming the sabotage is adequately detected.

Save the following diff as `/private/tmp/yard-sabotage-g14-bound.patch`

```sh
git apply --check /private/tmp/yard-sabotage-g14-bound.patch
git apply /private/tmp/yard-sabotage-g14-bound.patch
cargo test -p e2e --locked a_bad_proof_entry_is_refused_by_name
git apply --reverse /private/tmp/yard-sabotage-g14-bound.patch
```

```diff
--- a/crates/yard/src/jobs/proof.rs
+++ b/crates/yard/src/jobs/proof.rs
@@ -10,7 +10,7 @@
 use std::path::{Path, PathBuf};

 pub const PROOF_MAX_BYTES: u64 = 64 * 1024 * 1024;
-pub const PROOF_MAX_FILES: usize = 1024;
+pub const PROOF_MAX_FILES: usize = 2048;

 #[derive(Clone, Copy, PartialEq, Eq)]
 enum Kind {
```

### G1 authenticated method table versus session refusal

Adds a success sentinel after session validation. It does not actually approve or land anything. The sessionless probe is expected to remain 404 and the test may still pass, unverified. This settles only whether the comment claiming detection of an exposed method-table arm is supported; it is not a simulation of unauthorized landing.

Save the following diff as `/private/tmp/yard-sabotage-g1-method.patch`

```sh
git apply --check /private/tmp/yard-sabotage-g1-method.patch
git apply /private/tmp/yard-sabotage-g1-method.patch
cargo test -p e2e --locked a_worker_cannot_land_by_push_rpc_or_canonical
git apply --reverse /private/tmp/yard-sabotage-g1-method.patch
```

```diff
--- a/crates/yard/src/mcp.rs
+++ b/crates/yard/src/mcp.rs
@@ -285,6 +285,7 @@
         }
         ("tools/list", Some(_)) => Ok(json!({ "tools": tools(grant.kind) })),
         ("tools/call", Some(_)) => Ok(call(daemon, &grant, &message["params"]).await),
+        ("attempt.approve", Some(_)) => Ok(json!({ "sabotage": "operator-method-served" })),
         _ => Err(json!({ "code": -32601, "message": format!("method {method:?} is not served") })),
     };
     let envelope = match result {
```

### Test selection counts

Counts come from current source declarations, not a test listing or execution. Each mutation selects one independently runnable test. Apply and reverse each patch separately, including the two G7 mutations.

| Host command | Selected tests |
|---|---:|
| `cargo test -p e2e --locked a_ticket_lands_end_to_end_and_leaves_only_rows` | 1 |
| `cargo test -p e2e --locked status_watch_returns_on_open_attention` | 1 |
| `cargo test -p e2e --locked a_host_landing_gate_runs_on_the_merged_ref_only_when_approved` | 1 |
| `cargo test -p e2e --locked a_bad_proof_entry_is_refused_by_name` | 1 |
| `cargo test -p e2e --locked a_worker_cannot_land_by_push_rpc_or_canonical` | 1 |

Five mutation runs, one test per run. The complete source declares 74 tests; `cargo test -p e2e --locked` selects the whole suite. No command in this section was executed here.

### Host results, yard-sthlm

Run on dac65ff with a warm image cache. The whole suite passed, 74 of 74,
in 207 s with default test threads. Each patch was applied alone, run with
its test, and reversed; the tree was clean after each.

| Sabotage | Outcome | Detection |
|---|---|---|
| G7 check audit | failed in 9 s at the check-to-event equality | direct |
| G7 watch `--since` | failed in 15 s, "watch --since returned on an item raised before it" | direct |
| G13 approval reread | failed after 609 s in `watch.until("Y-2 approval again")` | deadline only |
| G14 proof entry bound | failed after 608 s in the decision loop | deadline only |
| G1 method table | passed | none |

G13 and G14 detect their sabotage only by the deadline, which is not a
pass but is not a reason either. G1's sessionless probe is answered 404
before the method table, so its comment's method-table sabotage is not
detected. The three are one ticket.
