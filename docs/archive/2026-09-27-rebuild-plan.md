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
| Build starts outside Yard; moves into Yard once the thin path is green, with the old Yard on yard-sthlm running lanes on this repo. An Opus orchestrator with subagents drives it. | The spec, table and skeleton have no ticket boundaries; "make guarantee N green" is exactly one lane's worth. |
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
| `src/provider/pi.ts`, `pi-mcp-extension.ts` | Pi's launch argv, frames, registration proof, session-id capture; the extension is embedded verbatim |
| `src/provider/{claude,codex}.ts` | for the adapters after v1: Claude's `system/init` frame lists the server and its tools; Codex marks the server `required`, so `thread.started` is the proof and the refusal on stderr the other answer; Codex needs `-s danger-full-access` and exits 0 on panic |
| `src/provider/connection-credentials.ts`, `connections.ts` | where each login's token and extra headers come from; input to pinfold #62, not to Yard |
| `src/process/host-gate.ts`, `spawn-gate.ts`, `birth.ts`, `kill.ts` | the host gate's environment, and the verified process-group reap a restart needs |
| `src/daemon/reconcile.ts` | what a restart must prove and what it must leave alone |
| `src/git.ts` `HARDENING`, `updateCanonicalTargetWithLease`, the containment proof | the git contract behind G3 and G14 |
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
   its own `XDG_STATE_HOME`, `XDG_CONFIG_HOME` and `XDG_CACHE_HOME`, never
   `XDG_RUNTIME_DIR`, which rootless podman needs as the host's;
   `operator.env`; a registered project with a `.yard/Dockerfile`;
   real pinfold), the fake-model fixture, and the scenario inventory: every
   guarantee's scenarios as named tests, `#[ignore]`d until their slice
   exists. The fake model speaks the one wire protocol Pi uses for the test
   connection, streamed and scripted per scenario; pinfold's own suite
   already runs Pi against such a fixture. Orchestrator work.
3. **The thin path.** The first executable milestone is G7 plus G3's
   landing-crash scenario: register a project, file a ticket, a real Pi
   worker commits against the fake model, a gate runs, approve, land, kill
   the daemon after `update-ref` and recover, clean. That path settles the
   store, process, git, box, identity and recovery boundaries together, and
   measures the budget. Orchestrator work, with subagents.
4. **Green, outward from the path, one guarantee per ticket where
   possible:** identity (2), capacity (4), restart (5), inputs (6), review
   (8), the implementer's next start (9), the queue (1, 10), sync (11),
   boxes and gates (12, 13), worker git (14), plans and proposals (15), and
   the rest of G3 and G7. Once the path is green the project gets its own
   `.yard` and the old Yard on yard-sthlm runs these as lanes: `cargo fmt`,
   `clippy` and `cargo build` as boxed candidate gates, and the e2e suite as
   a landing-stage host gate, which the old Yard also supports. That gate's
   `env` names `XDG_RUNTIME_DIR` and `DBUS_SESSION_BUS_ADDRESS`, or podman
   falls back to a cgroup manager pinfold refuses.
5. **Claude and Codex, before the cutover.** Each harness returns as its own
   adapter once pinfold's login routes (#62) carry subscription logins; the
   second adapter brings the adapter interface and a second protocol in the
   fake model. The machines' live agents run through these two harnesses,
   so the cutover waits for them.
6. **Cutover, per machine.** Pause admission and drain the board. Run the
   old `yard sync` so the checkout holds canonical's target head, and
   compare the two heads by hand. Stop the old daemon and move `.yard/local`
   to an archive directory outside every project and build context. Start
   the new daemon, `yard init` the project, `yard sync` from the checkout,
   and verify the new canonical's target equals the archived one's. Keep the
   archived state, the old runtime, its config and its service unit until
   rollback is no longer wanted; rollback is moving the directory back. Rename the repositories.
7. **Release.** `cargo build --release` per target, ad-hoc `codesign` on
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

- **The guarantees are the invariants, fifteen of them.** The separate
  I-1 to I-9 list was never written down. The thirty rows were one
  mechanism split across several rows in many places; they merged into
  fifteen with every scenario kept, cited as G1 to G15, and the
  `--version` row went.
- **`start` answers every `red`.** On a landing that could not run it
  re-queues under the existing approval; on an undecided landing intent it
  reads canonical against the restart table again.
- **Per-project status.** Each project has its own store and audit
  sequence. The daemon owns a registry of project paths; the CLI resolves
  its project from the working directory or `--project`.
- **No test waits out a timeout.** The clocks are configuration and
  untested; the review-round limit stays a guarantee.
- **Manual close.** `yard ticket done` with a reason, refused while an
  attempt or landing intent is live; it is how a planning parent ends.
- **A daemon installer.** `yard daemon install` writes a systemd user unit
  or a launchd agent and enables lingering on Linux.
- **Gates reach the project's allowlist.** No model route, no MCP route, no
  credential. The image builds only from canonical's target head, so no
  worker-edited file drives a build outside pinfold's sandbox; a dependency
  a candidate adds is fetched by the gate.
- **Each gate chooses box or host.** `runs_in = "host"` runs a gate on the
  host at either stage, as the project's decision: at the candidate stage
  that is unreviewed agent code with the operator's privileges. A
  landing-only restriction was proposed and dropped, since the old Yard
  allowed candidate host gates without an observed failure and auto-approve
  or a `none` panel would void it anyway. This replaces any post-push
  suite: Yard's gates decide what lands. Here the e2e suite is that host
  gate; GitHub runs it only when releasing.
- **A lane is a slot, an attempt runs through it.** `max_lanes` keeps its
  name; the CLI noun is `attempt` and the worker tools are `yard_context`,
  `yard_progress`, `yard_propose`, `yard_publish_review`.
- **Pi only in v1.** One adapter and one protocol in the fake model; Claude
  and Codex return before the cutover as step 5 of the order of work.
- **Logins are pinfold's.** Knowing where a harness keeps its login, which
  headers carry it and how the harness is pointed at a route belongs with
  the harness pins, so it was filed as pinfold #62. Yard keeps API keys
  from `operator.env` only.
- Two guarantee scenarios were corrected: the session cap's resume
  arithmetic and the observation for staying local. The dependency list
  gained the hyper glue crates and `getrandom`.

## The second design review, same day

A fresh reviewer read the package at 118ed94 against the old code and
pinfold 0.0.6. Taken:

- **The socket moved out of `XDG_RUNTIME_DIR`**, to
  `$XDG_STATE_HOME/yard/yard.sock`. Relocating the runtime directory for a
  test machine makes rootless podman fall back to `cgroupfs`, which
  pinfold's preflight refuses; pinfold's own harness relocates only state,
  config and cache for that reason. The daemon passes the host's runtime
  directory and session bus to pinfold.
- **Pi's discovery is off.** Pi loads a project's `.pi/` and skills by
  default, so a candidate could carry an extension that a reviewer's Pi
  loads and that publishes a pass with the seat's own bearer. The old
  adapter passed `--no-extensions` and every sibling flag; the spec had
  dropped it. Project guidance reaches workers through `instructions`.
- A read-only attempt ends when its execution stops, and the scheduler
  starts a read-only ticket once.
- `start` on a gate or review error reruns that check; `timeout` and
  `limit` exit through a nudge, which renews the clock or allows one more
  round.
- `yard attempt start` has a contract: the scheduler's admission for one
  named ticket, refused with the reason.
- G5's intent-before-effect crash window was not a provable point; it is
  shown by ordering instead.
- A review publication is an audited decision holding the findings and the
  check, so tests wait on it through `status --watch`.
- The daemon serves nothing until reconciliation has committed.
- Minor: the host-gate machine mutex was cut, no failure named it; OOM is
  `failed` with cause `oom`; a nudge is refused on a landing's item.

## Effort

Around 60k lines of Rust and an e2e crate of a few thousand. Two to four
months of calendar time with agents on the mechanical crates and an
orchestrator on the core. A guess, recorded so it can be checked.
