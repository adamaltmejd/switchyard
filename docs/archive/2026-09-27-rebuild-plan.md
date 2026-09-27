# The rebuild: plan, decisions and evidence — 2026-09-27

Switchyard is rebuilt from scratch in Rust in this repository. The previous
implementation, TypeScript on Bun at `../switchyard`, keeps running every
project until the cutover and is then archived. This file records why, what
was decided, and the order of work. The spec is `docs/ARCHITECTURE.md`.

## Why

Two reasons, of equal weight.

1. **Leanness.** The old tree is 76k lines of source and 112k of tests for
   a loop whose core is about 24k lines: the store's lane transitions
   (11.3k), the supervisor (6.6k), git (4.8k) and startup reconciliation
   (2k). Forty-five days of dogfooding landed 770 tickets and accreted
   features whose use never justified them (below). The audit stream is
   87–89% process-handle bookkeeping on both hosts; the store is 207 MB and
   evidence 1.8 GB for one project.
2. **Runtime.** One static binary with bundled SQLite on Linux and macOS,
   native `flock`, `setsid` and rlimits instead of FFI and a shell gate,
   exhaustive types over the execution kinds and decisions, and no exposure
   to Bun point releases (1.4.1's unix-socket pooling broke a release; macOS
   SIGKILLs Bun's compiled payload without a re-sign).

Containerisation was already moved out to pinfold on 2026-09-26, and
autoreview was retired on 2026-09-27 in favour of review seats running in
pinfold boxes. Both were preconditions: the new spec carries review as it
now works and calls pinfold for everything about a box or a harness.

## Decisions ratified with the operator

| Decision | Reason |
|---|---|
| No migration, no import. New board, new schema, cutover on an empty board on every machine. | Byte-compatible migration of the live store was the single most expensive item; both boards can be drained. |
| Pinfold's docs structure and testing rules, adapted in `AGENTS.md`. | One spec, one guarantees table that is also the invariant list, e2e only, no stubs. |
| TDD-refactor: the guarantees table and the scenario inventory first, ignored until their slice exists; the thin path goes green first; a ticket turns rows green. | Completion is the table green, nothing else. |
| Two levels: attempt, then execution. "Generation" is gone. | One execution is one unit of external work: a box, or a bounded host effect for landing and cleanup. Statistics are per execution row. |
| Audit stream holds decisions only; statistics are rows. | Disk, and `status --watch` pages over signal. |
| Operator behaviour is analysable: every decision names its exact target and keeps its text; tickets keep origin, workflow, plan-first, body size, edit count. | "Are we filing the right tickets?", "why do we nudge?" |
| No evidence after cleanup: transcripts, logs, bundles and the clone go at land or abandon; rows remain. | 1.8 GB vs 198 MB. Git holds what landed. |
| Compaction is the harness's: Yard stages the native auto-compaction threshold and never compacts. `max_session_executions` per workflow starts a fresh session from a brief; the total-work clock is the only spend bound. | Pi cannot compact out of band; the old between-rounds compaction was deleted for that on 2026-09-25 (Y-744). ~45 attempts per project ran past five executions, tail to 23. |
| Auto-approve stays. | 46 uses here; one config line and an actor value. |
| One daemon per machine, one scheduler, every registered project. `YARD_MAX_LANES` is a machine setting. | Box slots and memory are machine resources; the operator runs several projects on one Mac. |
| `preflight` becomes a minimal `doctor`. | Pinfold owns harness versions and the runtime. |
| Pinfold owns everything about harnesses; Yard only calls it. | |
| Build starts outside Yard; moves into Yard at the first red test, with the old Yard on yard-sthlm running lanes on this repo. An Opus orchestrator with subagents drives it. | The spec, table and skeleton have no ticket boundaries; "make guarantee N green" is exactly one lane's worth. |
| Mac cuts over on a clean Yard; no coexisting binaries. | |
| E2e budget: about five minutes, measured not asserted. | Real boxes are slower than the old stubbed 90-second gate. |
| This repo takes the `adamaltmejd/switchyard` name after the old one is renamed. | |

## Features: keep, retire, defer

Evidence: `2026-09-27-usage-stats.ts` in this directory, run read-only over
both live stores on 2026-09-27; the outputs are beside it (770 tickets over
45 days here; 254 tickets on the Mac's registry-research-toolkit).

| | Switchyard | RRT | Verdict |
|---|---|---|---|
| nudges | 44 | 88 | keep |
| lane rejections | 144 | 119 | keep |
| tickets born from proposals | 159 | 35 | keep |
| plan-first tickets | 24 | 31 | keep |
| named workflows in use | 3 | 10 | keep, first-class |
| triaged tickets | 46 | 0 | defer |
| auto-approved | 46 | 0 | keep |
| residual acceptance | 6 | 2 | retire |
| candidate-config attention | 7 | 1 | retire |
| ticket-edit-proposed | 2 | 4 | fold into proposal |
| project relocate | 2 | 0 | retire |
| implementer executions past 5 per attempt | 57 | 44 | `max_session_executions` |

Also retired, with the reason:

- **Triage and report/export.** An experiment; zero use on the Mac. Report
  is a query over the statistics rows when it returns.
- **Seat `when` scopes.** They needed the TypeSafe judgment triage brought.
- **JUnit evidence and gate artifacts.** Built 2026-08-13 so a gate could
  tell "ran, one skipped" from "never ran" on Yard's own release axis.
  Gates are pass/fail commands.
- **Worker CLI probes, the Pi probe extension, preflight's gate runs.**
  Pinfold reports harness versions.
- **Repository observation.** Existed because git on the Bun event loop
  blocked the supervisor; a blocking thread has no such problem.
- **Memory share computation.** Pinfold's box spec takes a memory string;
  `YARD_BOX_MEMORY` gives it.
- **`[workspace] build_artifacts`.** Disposal is whole-clone.
- **The per-project auto-spawned daemon and the registry/single split.**
- **Twenty-four attention kinds** become four with reason tokens.

The open questions are kept in ARCHITECTURE.md.

## What the old tree is good for

Read, never port. The mechanisms below are where the old code holds
lessons a guarantee test should reproduce:

| Old file | Lesson |
|---|---|
| `src/process/spawn-gate.ts`, `birth.ts`, `kill.ts` | intent before spawn, birth identity against pid reuse, verified process-group reap |
| `src/pinfold.ts` | the box spec Yard sends, the JSON lines it validates, exit 3 for an absent box |
| `src/provider/{pi,claude,codex}.ts`, `pi-mcp-extension.ts` | launch argv, the frames each harness emits, registration proof, session-id capture; the Pi extension is embedded verbatim |
| `src/provider/connection-credentials.ts`, `connections.ts` | where each login's token and extra headers come from, and the lapse refusal |
| `src/daemon/reconcile.ts` | what a restart must prove and what it must leave alone |
| `src/git.ts` `HARDENING`, `updateCanonicalTargetWithLease`, the containment proof | the git contract behind G4 and G9 |
| `src/daemon/mcp.ts` | the four JSON-RPC methods and the per-execution bearer |
| `DESIGN.md` §The loop | the reasoning behind nudge delivery, the dirty-clone rule, the queue's bisect and retirement |
| `docs/history/` | every failure the old tests encoded |

Nothing under `test/` is ported. The pinfold stub and screenplay protocol
are replaced by one host fixture: a scripted fake model behind a pinfold
route that real harnesses talk to.

## Order of work

1. **Spec.** `docs/ARCHITECTURE.md` and this file. Done 2026-09-27, with
   the readiness review below.
2. **Inventory and harness.** `crates/e2e`: the harness (a temp machine:
   `XDG_*` dirs, `operator.env`, a registered project with a Containerfile,
   real pinfold), the fake-model fixture, and the scenario inventory: every
   guarantee row as named tests, `#[ignore]`d until their slice exists. CI
   runs the crate on Linux. The fake model is the largest piece: it speaks
   Anthropic Messages for Claude, OpenAI Responses for Codex and chat
   completions for Pi, each streamed, scripted per scenario. Pinfold's own
   suite proves only Pi against a fake model, so a spike runs Claude and
   Codex against it before their adapters are planned. Orchestrator work.
3. **The thin path, one harness.** The first executable milestone is
   guarantee 10 plus the landing-crash row of 4: register a project, file a
   ticket, a real Pi worker commits against the fake model, a gate runs,
   approve, land, kill the daemon after `update-ref` and recover, clean.
   That path settles the store, process, git, box, identity and recovery
   boundaries together, and measures the budget. Orchestrator work, with
   subagents.
4. **Green, outward from the path, one guarantee per ticket where
   possible:** the other harnesses (20), review and panels (15, 16, 21),
   the queue (1, 2, 3, 17, 18, 19), executions and reconcile (6, 23, 12,
   13, 14), planning and proposals (26, 27), sync and git (25, 9), machine
   and CLI (28, 29, 30), cleanup and audit (24, 7), the bounds (11). Once
   the path is green the project gets its own `.yard` and the old Yard on
   yard-sthlm runs these as lanes, gated by fmt, clippy and build. The e2e
   suite is not a lane gate, because a gate box has no container runtime:
   it runs on GitHub after the operator pushes, and a red suite is the next
   ticket.
5. **Cutover, per machine.** Pause admission and drain the board. Run the
   old `yard sync` so the checkout holds canonical's target head, and
   compare the two heads by hand. Stop the old daemon and move `.yard/local`
   to an archive directory outside every project and build context. Start
   the new daemon, `yard init` the project, `yard sync` from the checkout,
   and verify the new canonical's target equals the archived one's. Keep the
   archived state, the old runtime, its config and its service unit until
   rollback is no longer wanted; rollback is moving the directory back. Rename the repositories.
6. **Release.** `cargo build --release` per target, ad-hoc `codesign` on
   the Mac as today, one workflow that tests, builds and publishes.

## The 2026-09-27 design review

An independent agent reviewed the three files. Taken into the spec the same
day: a landing intent serialised with the commands that could invalidate it
and a restart table decided by canonical alone; verification identity
written as data, attachments excluded from it, image id recorded as the
environment used; the full git hardening list from the old wrapper
(`protocol.file.allow=always` is what lets a local clone work at all), a
no-hardlinks clone and a fetch-only read of worker clones; pinfold's `.git`
protection and image-build trust stated correctly; a gate box as a private
writable checkout, as the old one was; a per-attempt harness-state mount;
per-adapter MCP evidence instead of a frame shape Codex never emits;
`max_session_executions` as a session rollover with the total-work clock as
the only spend bound; capacity accounting; batch subsets bound to their
target and conflicts resolved against canonical, both cut later that day; publication separate from
outcome; OOM `null`; `stopped:limit`; `access` on the mount; decisions as
the audited transactions; plans and findings as rows, the plan table cut later that day; replay dropped; the
suite runs on hosts, never in a gate box; several scenarios per guarantee;
the budget measured; the thin path before the full red crate; the cutover
that archives rather than deletes.

Declined: unit or property tests beside the e2e crate (pinfold's rule
holds until the e2e crate proves it cannot cover a decision).

The configuration-barrier question was answered by removing its premise
(operator decision, same day): `.yard` changes only through `yard sync`. In
this project 13 of 1771 commits were lanes touching `.yard`, toolchain
bumps and one package. A candidate touching `.yard` is refused at the
boundary; one configuration exists; two digests (gate, review) supersede
exactly the in-flight checks a sync invalidates. Gone with it: candidates
binding their own configuration, verified-alone batches, the `.yard`
approval special case, builds from candidate Dockerfiles.

## The leanness pass, same day

Two more store measurements: every one of the 75 landings the old queue ran
covered exactly one candidate, and `lane_report_too_large` was used once.
Cut from the spec, each returnable as one ticket when a need is measured:

- **Batching and bisection.** The queue lands one candidate at a time on
  its own full gate run; the ref, the compare-and-swap and the containment
  proof are unchanged, so batching later is additive.
- **Planning as a kind, and the plan table.** Planning is a workflow with
  read-only access whose worker proposes children and a body edit; the body
  is the plan and a replan is a ticket edit.
- **The conflict job.** A candidate that does not merge gets a repair
  execution in its own clone naming the paths.
- **`lane_report_too_large`**, folded into `lane_propose`.
- **Descriptive ticket links** (11 of 302 links). `depends_on` only.
- **`history` and transcript rendering.** `status --watch --since` is the
  history; `lane tail` prints the raw file.
- **Attachments**, deferred: landed two days before the review, no use to
  point at.

Result: five execution kinds, cleanup included; four lane tools; four
attention kinds; nine tables; 30 guarantees still but eight of them smaller.

## The readiness review, same day

A second read asked whether the plan was ready to build. Taken, with the
operator's decisions:

- **Invariants are the guarantees.** The separate I-1 to I-9 list was never
  written down; the guarantees table is the one list, cited as G1 to G30.
- **`start` answers every `red`.** On a landing that could not run it
  re-queues under the existing approval; on an undecided landing intent it
  reads canonical against the restart table again.
- **Per-project status.** Each project has its own store and audit
  sequence. The daemon owns a registry of project paths; the CLI resolves
  its project from the working directory or `--project`.
- **No test waits out a timeout.** Guarantee 11 keeps only the review-round
  limit; the clocks are configuration and untested.
- **Manual close.** `yard ticket done` with a reason, refused while an
  attempt or landing intent is live; it is how a planning parent ends.
- **A daemon installer.** `yard daemon install` writes a systemd user unit
  or a launchd agent and enables lingering on Linux.
- **Logins.** Pinfold injects; Yard sources the value. A key comes from
  `operator.env`; a subscription login is read from the machine's
  credential file at box start, never refreshed, refused if it would lapse
  within the total-work clock. Carried from the old design of 2026-09-26.
- **The suite runs after the push.** Yard lands into its own canonical, so
  GitHub's merge queue never sees a Yard landing; this repository's lanes
  are proved by fmt, clippy and build, and a red suite on `main` is the
  next ticket.
- Guarantee 13's resume arithmetic and guarantee 9's observation were
  corrected; the dependency list gained the hyper glue crates and
  `getrandom`.

Left open for the operator: whether gate boxes keep no egress, which decides
where an image's build context comes from; and whether the CLI and tools
keep the word "lane" for an attempt.

## Effort

Around 60k lines of Rust and an e2e crate of a few thousand. Two to four
months of calendar time with agents on the mechanical crates and an
orchestrator on the core. A guess, recorded so it can be checked.
