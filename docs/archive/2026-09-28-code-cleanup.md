# Code cleanup and spec pass, 2026-09-28

The first whole-tree pass, from 4e69782 on `thin-path`. It covers the
`code-cleanup` read with `test-audit`'s sweep, and the operator's spec
pass. 17 read-only reviewers read the tree: nine modules, a trace, the
user docs and six test groups. There is no tag yet, so the base is the
root commit.

## Numbers

| | Before | After |
|---|---|---|
| `git diff --shortstat` root..HEAD | +15923 −217 | +15494 −234 |
| Rust and TS lines in `crates/` | 14031 | 13615 |
| Largest source files | `supervise.rs` 1169, `admit.rs` 1135 | `supervise.rs` 1106, `admit.rs` 1091 |
| `docs/ARCHITECTURE.md` | 693 lines | 662 lines |

The pass itself is +1193 −1639, a delete/add ratio of 1.37. By area:

- `crates/yard`: +760 −1216.
- `crates/e2e`: +389 −349.
- Docs: +44 −74.

Tests grew where a rewrite replaced an assertion that could not fail with
one that can.

The suite passes 55 of 55 on yard-sthlm with pinfold 0.0.7. The binary
changes ran in 4.9 min, and the final tree, with the rewritten tests, in
5.2 min. That is within the run-to-run spread the 0.0.7 note reports.

## Commits

| Commit | Removed |
|---|---|
| aa24e9f e2e harness | `scratch`, `ModelRequest::contains`, `Reply::Status` and the separate `tools()`; one stdout reader and one read-only store open where there were two |
| eba412a box, git | `UpError`, `BuildError`, `Built`, `Listed`, `Stat`, `StatLine`, `ListLine`, `Collected`, `RunError`, `cpus`; `Opts` and `is_ancestor_with`; `set_head`; `delete_ref` (later); a second close-on-exec block; git's own undrained capped read, which left a large writer blocked until its timeout; `doctor`'s unbounded `pinfold --version` |
| 692b26c pi | the `Progress` event and `Finished.text` nobody read, `state_files`, a second effort check, `Normalizer::new`, `or_none`; the MCP client's content adapters and hand-rolled `AbortSignal.any` and byte join |
| fd1526b store | `ticket.started_once`, `attempt.branch` (now derived), `Start.resumed`; `lane_since` as TEXT; eleven prepare/query/collect triples; the exits table restated in `answer_start`; a leaked lock clone |
| 00d15da jobs | a second reading of a current approval item; three branch-then-head reads; repeated `Start` fields; five `map_err`s in reconcile; a per-attempt busy scan |
| 84f5499 review, queue | the seat's own box-up sequence; the `summary` field accepted and dropped; three of `returned()`'s parameters and its lint allow. Bug: a review check recorded `image_id` NULL |
| 503f632 admit | readiness derived a second time in Rust, now one SQL `BLOCKER`; the `refs/yard/import` temporary ref and its deletion; `text` parameters only proposal acceptance set, which `attention.resolved` already records |
| 3450667 supervise | an unbounded host gate output read; restated tool lists, one of which missed `yard_progress`; a handle write that reset the box name; the `dev.yard.execution` label; an `Option` deadline; `nudge_pending`; two prompt parameters |
| f523f1f config | eight `default_*` functions and a hand-written `Default` |
| b9dc225 cli | 83 lines of `project_call` restating every subcommand's fields; a duplicate `--version`; `--since` without `--watch` |
| e75c4fb daemon | a duplicate open-reconcile-insert path; service paths built three times, with `uninstall` ignoring `XDG_CONFIG_HOME`; a leaked lock clone; `events`' unused `wait` |
| fc10ed7 … 3f671f3 | test-audit, one commit per guarantee file (below) |
| 29048fb | the spec pass (below) |

## Test audit

Reviewers judged 186 assertion blocks.

**Deleted, about 35 blocks:**
- Detectors of state tokens, reason prose, counts and data shapes the rows don't name.
- Twins of a surviving block.
- Comparisons of yard's own outputs.
- Probes that failed for an unrelated reason: G1's `push-origin` and fake-tool probes.

**Rewritten, because they could not fail:**
- G5: "no second box" is now read from pinfold's box list.
- G7: every audit event must be one the Store section names.
- G8: the seat's box is read as gone from pinfold when the repair starts.
- G9: a nudged execution ends with its own outcome. The brief is checked against fixture values.
- G11: a candidate gate exists, and never runs on the refused head.
- G13: the head is read by git in the clone.
- G15: the read-only mount has a writable control. The live-attempt close refusal is taken while the attempt waits on approval.

**Sabotage comments added or corrected:** G1, G2, G3, G4, G6, G10, G12, G13 and G14. The G13 landing sabotage must skip `holds` in both `queue::next` and `queue::land`. That was reasoned from the code and not applied.

## Operator's list

The operator said that not everything the spec promises has to be built yet, and that the suite should stay lean. What was not built but is still intended stays in the spec as tickets. Only vestiges came out.

- **Spec fitted to decided code:**
  - `red` with reason `error`.
  - A landing's `red` item exits only by `start`.
  - The landing lock covers the ref update and its ancestry proof.
  - `status --watch` prints every event.
- **G3:** "refuses the queue" is cut from the row. Its only block checked `code == "refused"` on `attempt start`.
- **G7's candidate gate:** no new block. The approval it asserts carries the gate check.
- **Restatement cuts** across ARCHITECTURE.md, and one move into AGENTS.md: accepted.
- **`yard daemon install` spawns:** `launchctl`, `systemctl` and `loginctl` run without a scrubbed environment or time bound. They are outside the git/pinfold child-process rule, run in the operator's CLI, and are left as they are.

## Tickets

- **`fresh` start reason:** an execution that starts a new session after `max_session_executions` records `restart`, `retry` or `nudge` instead.
- **Harness version:** record it on worker rows. A version change ends session continuity.
- **Compaction threshold:** stage the harness's automatic-compaction threshold with the launch.
- **`doctor`:** report the service, when each login lapses, and registered paths that are gone.
- **`check` rows:** a `check` repeats its execution's input: attempt, name, base, head, revision, digest, round and image. Keeping the execution's columns as the only copy removes the two `Input` blocks. This is a rework of `store/checks.rs` and G2's reads.

## Rejected

Each with the condition that would re-admit it.

- **`now()` via SQLite `strftime`** (−28). Re-admit when a second timestamp form is needed or the store's writes are next reworked.
- **Filling an audit event's ticket from its attempt** (−25). It changes what existing events name. Re-admit with a guarantee about event targets.
- **Deriving `attempt.cleaned` from the cleanup execution.** Re-admit if cleanup gains a second outcome.
- **Reconcile's two loops as one.** It is a crash path and the ordering is subtle. Re-admit if a third loop appears.
- **Dropping `queue::next`'s `holds`.** The pre-gate read in `land` would then create a withdrawn landing row for every lapsed approval. Re-admit if the double read causes a bug.
- **Building the approval payload inside `raise_approval`.** Its eight parameters replace five lines. Re-admit if a third caller appears.
- **Serde `deny_unknown_fields` for findings.** `strict` names the key the same way for every tool. Re-admit if `strict` goes.
- **`clone_detached` as init plus fetch.** Small gain, and a spec wording change. Re-admit when cloning is next touched.
- **`operator.env` accepting only `NAME=VALUE`.** The operator writes this file by hand. Re-admit when the spec states its syntax.
- **`daemon.status` listing the registry.** It reports what is served, and `project list` reports what is registered.
- **`Latch::new`, `last_tool_result` and `HOLD_DEADLINE` in the harness.** Used widely, and a shorter deadline fails a held-model test sooner.
- **Deleting the `## Code` tree.** The cleanup skill groups modules by it. Its drift was fixed instead.
- **AGENTS.md's "Jobs, not states" and store-writer rules.** They are the working checklist, and the spec states the mechanism.
- **`--project` and `--json` help on commands that ignore them.** They are clap global flags. Re-admit if they confuse a user.
- **G9 `stop` delivers sooner:** its sabotage fails only at the deadline. No event exposes a worker that was never stopped.
