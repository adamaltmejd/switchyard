# Proposed Switchyard redesign plan — 2026-10-09

Status: planning only, corrected after Astra review, for Adam's review. This is a dated proposal, not the
binding current spec. Implementation must update ARCHITECTURE.md in the
same commit as each implemented contract. All new tickets are parked at
creation. Nothing in this document authorizes admission or a daemon switch.

## Baseline and authorization

Repository and origin/main are 81da8c677a0ab43966588429c24a8affedb9e8ec,
release 0.0.3. Frozen construction executable: /home/yard/.local/bin/yard.
SHA256: 05ae25832964703c81525e7c5f5be7a8148954517cfddf69ef9846953bf1ebb2.
Daemon boundary: 05ae25832964703c, rediscovered PID 965905. Initial audit seq
3602: no active tickets, attempts, attention, queue or running executions.
History has 53 tickets, all closed (done or abandoned); no duplicate active work.
The host is newer than the earlier Mac/source review. No update-to-Mac ticket.

Adam chose “Stop after the plan and tickets are ready.” Later instructions
also authorize staging build-agent configuration and planning a narrow API
support patch release. They do not start implementation or install a patch
in this turn. The same receiving operator session remains the operator.

## Build configuration and bootstrap

Staged, uncommitted .yard/config.toml selects Codex login, gpt-6.1-sol,
medium for implementation. Temporary review is Pi, opencode-go,
deepseek-v4.1-flash, high. .codex/config.toml stages service_tier="fast"
through Yard's existing trusted-base Codex config path. This does not change
the receiving operator session's model/settings. Gate and lane settings
remain those in the existing build project (max_lanes=2, manual approval).

The intended reviewer is Claude Opus 5.5, medium, direct Anthropic API.
Frozen 0.0.3 Claude::accepts rejects provider and only permits login=true.
Direct API support is the first implementation ticket, followed by a narrow
0.0.4 patch preparation ticket. Future .yard profile is saved in the handoff
as build-config-after-anthropic.toml; it is unsupported by 0.0.3 and is not
loaded now. OPENCODE_API_KEY is present by name in operator.env;
ANTHROPIC_API_KEY is absent. No secret value was copied into these artifacts.

No commit or sync occurred. The daemon still reads the original canonical
configuration. On later authorization, the operator commits/stages the
build configs, syncs with frozen Yard and confirms actual model access and
fast-tier request behavior before admitting the API ticket. No paid model
call or worker was run during planning. The API key is provisioned privately
by the operator before switching to Anthropic review. Upstream catalog/API
access and model entitlement are operational checks, not assumed passes.

The requested quick patch is an explicit future exception to freezing 0.0.3:
only after narrow patch verification and Adam's release/switch authorization
may a verified 0.0.4 replace it. No intermediate development build operates
the backlog. Without that switch, 0.0.3 with the temporary reviewer can still
construct the successor. A release preparation ticket does not itself publish
or install anything.

## Current behavior versus proposed behavior

Current: one project target and config cache, project-wide landing queue,
one no-hardlinks clone per attempt reused across implementation/repair,
max_lanes admission (two here), sequential gates and review seats, one
execution lifecycle, worker-only HTTP MCP, task-management CLI, nullable
usage columns with missing-cost-to-zero adapter paths. Committed-history
sync refuses divergence. Proof/log artifacts disappear at attempt cleanup.

Proposed: per-ticket target override, per-target landing serialization,
trusted operator MCP, resolved revisioned profiles/presets/graphs,
operator-controlled verification and continuation, exact result reuse,
small Durable audit runner, concurrent read branches and independent
writers, explicit candidate selection/integration, individual scoped
execution without synthetic tickets, truthful secondary statistics, then
task CLI retirement. No GitHub prerequisite or new coding harness.

## Proposed contracts and planning choices

Target resolution: default is taken at admission; every attempt freezes its
local branch ref. An explicit target edit on a ticket with a live attempt
or landing intent is refused. Inactive open-ticket edits use revision CAS.
Missing target is a named refusal, with no attempt created; explicit
operator branch sync imports/creates it (Y-82). Canonical symbolic HEAD is
the project-default reading; changing it is an explicit validated sync
selection. Y-82 refuses that change while old single-target work is live;
Y-56 can allow it because its active attempts freeze their branch. Default
edits affect future admissions only.
A ticket is complete when it lands into its named target, even if that
branch is never merged into main. Final feature merge is ordinary operator
Git work; sync only affected committed branch histories.

Target config comes from the selected branch head. Its declared target is
the project default, not a demand that every feature branch rewrite config.
Cache/image identity includes target and real relevant inputs. Each target
has one landing at a time, its own CAS/intent recovery and serialized sync;
an unknown intent blocks only that target. Dependencies still mean prior
tickets completed; where landed work is required, do not substitute manual
close for that acceptance evidence. No feature/group/promotion objects.

Graphs govern eligibility on the existing execution lifecycle. Nodes have
command/agent and read-only/write-allowed axes; roles are presets. Typed
candidate, report/verdict and selection outputs name explicit input refs.
Routes match declared outcomes. Joins name required results. Repeats have
explicit limits, with operator revision/attention on exhaustion. No embedded
expression language. Every retry, repair or resume is a new execution;
history is never reversed. Judgment and infrastructure outcome are distinct.

TOML in tracked .yard stores complete presets and profiles. The trusted MCP
also takes structured one-off graphs and targeted overrides using that same
model. Admission persists an inspectable resolved plan and revision. Template
sync cannot silently rewrite active work. Revision CAS steers remaining work
with explicit invalidation; running inputs are immutable. Source/skill/tool,
proof, configuration/policy and environment/image identities qualify reuse.
Unknown identity means no reuse. Bookkeeping alone does not require reruns.
A different merged tree is a different check input.

Policy precedence (Y-59/Y-62): workflow nodes, profiles, selected gate/review
and Dockerfile/image/tool/skill definitions stay frozen in the admitted plan.
Template sync affects future plans; adopting changes for remaining active work
requires explicit plan revision CAS. This replaces current automatic
verification-digest invalidation on template sync and requires the matching
G2/spec/assertion changes. It does not freeze operator safeguards: approve
and protected_paths are read from the current selected target head before
automatic approval and again before landing gates/intent. A safeguard sync
can withdraw automatic approval and raise attention while leaving the plan
and unchanged checks intact. Manual decisions still bind exact identities.
Project capacity limits and credential availability remain live operational
constraints. Lowered limits block new effects until usage drains; existing
work is not killed. Worker grants and containment are never plan-overridable.
Queued-auto-approval/safeguard-sync and explicit definition-adoption scenarios
are named in the relevant tickets; their observations are distinct.

The trusted operator may omit all reviews/checks, including landing checks,
or explicitly accept failure. Records say skipped, reused or overridden;
none becomes a fabricated pass. G1/G3 and their threat-control text must be
revised explicitly: only the queue lands a precisely authorized candidate
with the selected verification evidence. Worker grants, containment, exact
identity, target CAS and unknown-owner fail-safe remain intact. This is a
spec change, not a hidden exception policy. No second human approval gate.

Keep max_lanes as ticket admission. Y-83 introduces one shared positive
max_executions before Y-57 enables concurrent target landings; Y-67 later
reuses that mechanism for graph fan-out. The shared limit comes from current
project-default config, not separate branch configs. Absent max_executions
defaults to max_lanes+1. A live box/controller occupies one slot; concurrently
acting host work occupies a slot. Dormant landing orchestration awaiting a
child holds no extra slot: a cap of one can complete sequential child gates
and then the landing Git effect. Reserve before effects, release after
checked end; unknown ownership retains its slot. One active writer per
workspace and one landing per target are separate constraints. No general
resource scheduler or machine-wide host semaphore. Runtime tests needing
exclusivity coordinate with host processes.

Reuse one isolated clone per independent writer; sequential implementation,
cleanup and repair can hand it over. Concurrent writers have separate clones
and session state. Reviews/checks consume fresh stable canonical snapshots;
read-only mounts alone do not freeze a changing host tree. Selection returns
one declared candidate; integration writes a new candidate in its own
exclusive workspace. Write access does not confer publication or landing.
Keep only inspection/retention/resume improvements these workflows need.
Measure actual startup components before pooling/sharing optimizations.

Durable placement is decided: controller, model client and tools run inside
one Yard-owned box, preserving existing injecting model routes and worker
MCP grants. The host needs no Node/npm and runs no model/tool extension code.
Operator-owned target images supply a pinned Node 26+ patch and lockfile-backed
pi-durable/pi-ai/chord 1.0.4 runner bundle at a fixed image path, outside the
candidate workspace. No runtime installation, host-client fallback or
candidate executable discovery. Missing assets are runner_unavailable before
model calls; a corrected-image control proves the packaged runner works.
The implementation supplies source/lockfile/scaffold instructions; the operator
makes required installed .yard/image changes through commit+sync.

Y-64 starts with a bounded read-only skill test audit after Y-66 supplies the
usage contract. Trusted selected Markdown/references and stable candidate/proof
mounts are read-only; scratch and node-scoped checkpoint state are separate
writable mounts. The box receives a model-key placeholder, never an upstream
credential. Checkpoints are worker-untrusted data, opened by the boxed
controller, never evaluated on the host or counted as workflow evidence.
Validated execution-scoped MCP publication is authoritative. Yard owns
lifetime, deadline, cancellation, publication and routing; Durable owns
conversation internals. Y-65 proves old owner/box gone and reserves exclusive
state access before a new execution resumes matching inputs. Changed pending
inputs start fresh. Cancel work and check teardown; cancelling a wait is
insufficient. Attribute each new execution's increment. Unsafe interrupted
writes are never automatically replayed; Y-72 proves the write case once that
capability exists. No measured speed/cost/quality benefit is claimed.

Trusted operator MCP is a stdio adapter over the existing unix-socket daemon.
Workers retain their restricted HTTP MCP and execution-bound bearers.
Discovery, validation/resolution, tickets, attempts, graph revisions,
inspection/diffs/results, bounded waits, stop/retry and exact decisions belong
to the operator. Wait cancellation does not cancel work. Individually useful
command/report executions use the same lifecycle without a made-up ticket;
no ticket-free automatic landing. Direct host edits/testing need no Yard
workflow; re-entry uses committed-history sync, never dirty-file merging.

Usage is secondary: unknown versus explicit zero, incomplete versus complete,
estimates versus billing and per-execution increments remain visible in
drilldowns. Y-66 is a vertical contract slice including normal completed Codex
turns: absent monetary cost stays unknown and real zero token totals stay
zero. Legacy nullable records keep their values and legacy provenance. Pi/Claude
retain their old semantics until Y-75/Y-76 normalize them; Y-77 handles Codex's
distinct interrupted/resumed/cache-token cases without repeating Y-66 tests. No mandatory cost/token caps or prominent cost board. Optional
remote aggregate export is deferred and cannot block the first useful
redesign release. It excludes source, prompts, transcripts and private paths;
local rows are authoritative, offline is harmless, receiver keys deduplicate.

## Construction order and concurrency

Propose the existing registered project, current main destination and frozen
Yard 0.0.3 for construction, followed only by the separately verified patch
if Adam authorizes that switch. Do not assume future target/graph support in
construction scheduling. No extra clone/project is needed now. If distinct
construction destinations become necessary before target support, separately
registered independent clones/projects are the explicit workaround; layout
changes require an actual need and operator decision.

The table's phase is a review/concurrency band, not a second scheduler.
Dependencies are real Yard edges. Target queues and MCP can proceed in
parallel after basic targets. Usage contract and linear scheduling can
proceed after resolution; reader tickets are independent mechanisms. Durable
audit can proceed beside optional workflow/usage work once linear results
exist. Shared schema/scheduler/dispatch files are conflict hotspots: finish
shared contracts first, then serialize overlapping candidates at review.
Named-branch sync/default resolution (Y-82) precedes the narrower serial
attempt target-binding slice (Y-56). The early shared capacity mechanism
(Y-83) precedes independent target queues (Y-57). The later graph fan-out
slice (Y-67) reuses it and precedes standalone execution/schema changes,
which precede writer expansion. Prefer at most two independent construction
attempts under existing max_lanes=2; new fan-out is tested only in isolated
successor environments. Dependencies deliberately serialize later schema
and scheduling transitions even where conceptual analysis could overlap.

| Ticket | Scope | Dependencies | Parallel phase |
|---|---|---|---|
| Y-54 | Support direct Anthropic API in the Claude review adapter | none | 0 bootstrap |
| Y-55 | Prepare the direct Anthropic support patch release | Y-54 | 0 bootstrap release |
| Y-82 | Sync named branches and resolve the project default target | Y-55 | 1 operator branch sync |
| Y-56 | Bind a ticket target through the existing serial attempt workflow | Y-55, Y-82 | 1 targets |
| Y-58 | Expose existing operator operations through trusted stdio MCP | Y-56 | 2 operator MCP |
| Y-83 | Bound existing external executions with one shared project limit | Y-56 | 1 shared execution capacity |
| Y-57 | Serialize landing and recovery per target branch | Y-56, Y-83 | 1 targets queue |
| Y-59 | Resolve typed profiles and graph presets into frozen attempt plans | Y-58, Y-57 | 3 plan contracts |
| Y-60 | Drive the ordinary workflow from the resolved graph | Y-59 | 4 linear graph |
| Y-61 | Let the operator omit checks and accept failed results truthfully | Y-60 | 5 operator decisions |
| Y-62 | Persist operator revisions to remaining graph work | Y-61 | 5 revisions |
| Y-63 | Reuse recorded verification only for matching real inputs | Y-62 | 6 result reuse |
| Y-66 | Preserve unknown usage and per-execution provenance | Y-59 | 4 parallel usage contract |
| Y-64 | Run a bounded read-only skill audit with Pi Durable | Y-60, Y-66 | 5 parallel Durable audit |
| Y-65 | Resume Durable checkpoints as new exclusive executions | Y-64, Y-62 | 6 Durable recovery |
| Y-67 | Schedule concurrent read-only branches and explicit joins | Y-63, Y-66, Y-83 | 7 graph concurrency |
| Y-68 | Offer standalone scoped commands and agent reports without tickets | Y-67 | 8 parallel individual capability |
| Y-69 | Isolate concurrent implementations and publish multiple candidates | Y-67, Y-68 | 8 writers |
| Y-70 | Select an existing candidate before checks and landing | Y-69 | 9 selection |
| Y-71 | Integrate independent candidate outputs in a writable workspace | Y-70 | 10 integration |
| Y-72 | Pass one writable workspace through skill cleanup and repair | Y-70, Y-65 | 10 cleanup |
| Y-73 | Route failing checks through an explicit classifier result | Y-72, Y-71 | 11 classification |
| Y-74 | Ship complete role presets and targeted MCP overrides | Y-73, Y-68 | 12 presets |
| Y-75 | Normalize Pi usage without fabricating missing cost | Y-66 | 5 parallel Pi usage |
| Y-76 | Normalize Claude usage without fabricating missing cost | Y-66, Y-54 | 5 parallel Claude usage |
| Y-77 | Keep Codex cost unknown and usage local to executions | Y-66 | 5 parallel Codex usage |
| Y-78 | Expose usage drilldowns and honest partial aggregates | Y-75, Y-76, Y-77, Y-74, Y-65 | 13 operator polish |
| Y-79 | Move operator verification to MCP and retire task-management CLI | Y-78 | 14 CLI retirement |
| Y-80 | Verify redesign migration, recovery and final switchover readiness | Y-79, Y-57, Y-71, Y-72, Y-73 | 15 final readiness |
| Y-81 | Deferred optional export of aggregate usage without project content | Y-78 | deferred after useful release |

## Verification, transition and release

Each ticket names a new guarantee/spec clause, source mechanism, observable
acceptance, sabotage and exclusions. Tests use real daemon/project/Git/boxes
and real harnesses against scripted local model fixtures, external expected
values, positive refusal controls and reason/identity assertions. Use held
box/tool calls or held git wrappers for deterministic crash edges; prove
external state before restart. No binary-only barriers, sleeps or retries.
Do not duplicate existing assertion mechanisms. The test-audit skill gates
any changed assertions. No cargo/e2e run is needed for this planning-only turn.

Development binaries use distinct XDG state/socket, project paths, box/image
names and runtime state where supported. They never register/open the
construction backlog. Runtime-wide prune and exclusive tests require host
coordination; isolated paths alone do not guarantee runtime independence.
Keep sockets short enough for macOS. Candidate fmt/clippy/build run boxed;
real host landing e2e runs on yard-sthlm, Linux x86_64/arm64 CI, and operator
Mac before release. Use no public model endpoint in automated verification.

Move meaningful end-to-end assertions to supported MCP/API seams before
retiring task-management CLI. Update AGENTS.md test seams, shipped skills,
README and architecture together, with assertions preserving revised
contracts. Retain only necessary executable/service/MCP startup entrypoints.
The old operational CLI remains available until final switchover.

Before each release run code-cleanup whole-tree and test-audit sweep, land
admitted deletions one module per commit, then operator spec pass. Dated
record reports delete/add ratio, largest source file, architecture line
count and observed test budget. Follow-up cleanup is admitted by the project
rule; no unverified defect claims or unsolicited reliability expansion.

Migration validates an old-version fixture with historical/parked/live data,
then repeats reopen without duplicated effects. Stop the writer for a
consistent backup of store/WAL, registry, canonical refs and required live
artifacts. Restore an untouched copy with the old binary; never downgrade
an upgraded store. Ambiguous active-plan mapping raises attention, not guessed
continuation. Final switchover is a deliberate later operator action after
successor readiness and backup/recovery rehearsal. No publication, push,
installation or daemon restart is authorized by this planning turn.

## Deferred feedback and open decisions

RRT feedback is prior-user evidence, not accepted instructions. Holds,
read-only host references, path-based gate selection, external host-gate
coordination, question proposals, durable handback archives and automatic
post-landing sync are not committed scope. Re-admit only for Adam's requested
use/new guarantee or a reproduced CLI bug. Existing host gates/env,
manual committed-history sync, progress/body edits, protected-path approval,
review severity and ephemeral proof already cover parts of those needs.
Default/repair reviewer substitution is expressible by the accepted graph;
no extra panel object is needed. Cost prominence is explicitly declined.

Astra's six findings are addressed in the corrected ticket set: early shared
capacity, testable usage scope, explicit policy precedence, split branch sync,
boxed Durable packaging/credentials and matching dependency/phase artifacts.
These are proposed construction decisions for Adam to review, not implemented
contracts. No known blocking planning question remains after making those
choices. Later operational preconditions: commit/sync staged build config after authorization; verify
model entitlements/Pi catalog and fast request tier; privately provide
ANTHROPIC_API_KEY before direct review; explicitly authorize verified patch
and final release/switch. These are not permission to implement now. Remote
endpoint/credentials are deferred deployment inputs, not first-release gates.
If the frozen Pi catalog cannot resolve the documented reviewer model, report
that actual refusal and file its narrowly reproduced fix; do not silently
substitute another model or update Pinfold.

## Source references

- Binding current repository: AGENTS.md, docs/ARCHITECTURE.md and README.md
  at host 81da8c6; mechanism traces are in each ticket body.
- [OpenAI Codex configuration reference](https://learn.chatgpt.com/docs/config-file/config-reference):
  service_tier="fast"; the existing Yard staging preserves that key.
- [Opencode Go model/endpoint documentation](https://docs.opencode.ai/docs/go/):
  deepseek-v4.1-flash over chat completions. This verifies naming/protocol,
  not this host account's access or the installed Pi catalog.
- [Pi Durable v1.0.4](https://github.com/earendil-works/pi/blob/v1.0.4/packages/durable/README.md)
  and [Pi AI v1.0.4](https://github.com/earendil-works/pi/blob/v1.0.4/packages/ai/README.md).
  Durable supports Node storage and persisted internal work; wait cancellation
  does not abort that work. The proposed boxed Yard adapter remains to build.
- Local reference-pinfold-durable.md and reference-rrt-feedback.txt were read;
  the latter is previous-user evidence with the dispositions above. Pinfold
  prototype/architecture source was not available at /home/yard/code/pinfold;
  its supplied README and handoff are evidence, not a verified Yard adapter.

## Review artifacts and resumption

Full handoff: .yard/local/redesign-handoff-20261009/handoff.md.
Ticket map with dependencies, phase and acceptance evidence:
.yard/local/redesign-handoff-20261009/ticket-map.json.
Final readback/baseline verification and stop record:
.yard/local/redesign-handoff-20261009/final-report.md.
Astra findings and correction evidence:
.yard/local/redesign-handoff-20261009/astra-fix-report.md.
The original review draft is preserved locally in before-astra-fixes/.
Saved baseline and per-ticket readbacks stay in that directory. It is ignored
local session context, not a worker input. Tickets carry self-contained bodies
and this plan reference; the operator must commit the reviewed plan/config
before a future worker is admitted so the reference exists in canonical.

Readiness means these documents and parked tickets are ready for review.
STOP. Future operator work begins only after Adam explicitly authorizes it.
