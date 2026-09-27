# Switchyard architecture

Switchyard (`yard`) turns tickets into landed code with coding agents. It
keeps a local backlog, gives one ticket to one agent in one disposable box,
runs the project's gates against exact code, has a second agent review the
result, asks the operator for the decisions that need judgment, and lands
approved work through one verified merge queue.

- **Operator:** a person, or an agent driving `yard` on their behalf. The
  operator is trusted and holds the whole host API.
- **Worker:** an agent in a pinfold box. It holds an execution-scoped MCP surface
  and nothing else.

This is the current spec. Evidence and rationale are in `docs/archive/`.
Yard is a local tool for one developer and trusted colleagues. It contains
accidents, not a hostile process running as the same user.

## Terms

**Ticket**: one piece of work: title, body, priority, workflow name,
dependencies. `Y-<n>`. Its body is its plan.
_Avoid_: issue, task, story

**Attempt**: one try at a ticket: a clone of canonical on its own branch, a
candidate, and the executions that produced and judged it. A ticket has at
most one live attempt and any number of ended ones.
_Avoid_: worktree

**Lane**: one of the slots attempts run through. `max_lanes` and
`YARD_MAX_LANES` count them; an attempt holds one from admission until
approval or abandonment.

**Candidate**: an attempt's exact `base..head` at a moment. Every check,
approval and landing binds a candidate, never an attempt.

**Execution**: one unit of external work on the one persisted lifecycle:
an implementer call, one reviewer seat, one gate command, one landing, one
cleanup. It has one identity, one external handle once it has one, and one
terminal outcome. Box-backed executions own a box; host-backed ones
(landing, cleanup, a host gate) have bounded host effects and are recovered
from the effect they leave. A repair, a nudge, a retry and a
resumed session are each a new execution; statistics are per execution.
_Avoid_: generation, call, task, run, step

**Box**: a pinfold box (its term). Yard brings one up per box-backed
execution and holds it for exactly that long.

**Landing**: an execution that merges one approved candidate onto
canonical's head, verifies the merged ref with the full gate set, and moves
canonical to it; its gate runs are its child executions. The queue is the
approved candidates in approval order, landed one at a time.
_Avoid_: batch

**Canonical**: the daemon-owned bare repository a project lands on. The
operator's checkout is an input to it and never authoritative.

**Seat**: one named reviewer: an agent plus instructions. A workflow selects a
panel of seats.

**Agent**: a named way of running a model: harness, connection, model, effort.

**Workflow**: a named binding of knobs Yard has: implementer, panel,
instructions, access, budgets. `default` is required.

**Attention**: one question only the operator can answer, with the exact
identity it was asked on and the commands that answer it.
_Avoid_: alert, notification, blocker

**Machine**: the host one daemon serves; its own settings live in
`operator.env`.

## Layers

| Layer | Owns | Used by |
|---|---|---|
| store | schema, transactions, audit, statistics rows, the single writer | daemon |
| jobs | admission, the execution state machine, the merge queue, reconcile-on-start | daemon |
| git | canonical, clones, candidates, landing; git shelled out under a scrubbed environment | jobs |
| box | the pinfold process interface; specs, mounts, routes, stat | jobs |
| pi | Pi's launch argv, frame normalisation, the staged MCP client | jobs |
| mcp | the tool surface workers call | daemon |
| daemon | one process per machine: the unix-socket API, the MCP listener, the scheduler tick | `yard` |
| cli | argument parsing, one call per command, rendering | the operator |

The store knows nothing of processes. `git` and `box` return data and never
touch the store. The CLI never opens the store or a repository; it talks to
the daemon.

## Threat model

An agent, or a prompt injection in one, lands code nobody approved, spends
credentials, or damages the host.

| Control | Effect |
|---|---|
| One merge path | Only the daemon's queue moves canonical, and only to a ref its gates passed (G1, G3). |
| Exact identity | Every check, approval and landing binds a candidate, the ticket revision and the gate and review digests; a changed identity carries nothing (G2). |
| Worker capability | A worker's MCP bearer is issued per execution, scoped to its kind, revoked when the execution ends. |
| No credential in a box | Upstream keys and login tokens ride pinfold's injecting routes; a box sees a placeholder. The only secret a box holds is its own execution-scoped MCP bearer, which authenticates calls from inside it and authorises nothing else. |
| Box is the sandbox | The harness runs fully permissive; access is what the box mounts writable. A reviewer's clone is read-only. A gate box reaches only the project's allowlist: no model, no MCP, no credential. |
| Worker git is untrusted | A worker's clone shares no objects with canonical and its configuration is never read by the daemon: every daemon git call disables hooks, `core.fsmonitor`, signing and every transport but validated local paths. |
| No checkout writes | Yard never writes the operator's checkout except inside `yard sync` the operator ran. |
| Gates run where the project says | A project chooses per gate whether it runs in a box or on the host. A host gate runs agent-written code with the operator's privileges and no containment: before any review at the candidate stage, after approval at landing. Which gates accept that is the project's decision. |
| Fail safe | Unknown liveness or outcome never duplicates work, lands, deletes or kills (G5). |

Everything pinfold's programmatic core guarantees (no network but the proxy,
no privilege, the environment is the spec) Yard relies on and does not
restate. Pinfold's `.git` protection belongs to its interactive layer, not
to caller-owned boxes; Yard's own mounts are the answer above. Image builds
are outside pinfold's sandbox: trusted, with unrestricted egress. Yard
therefore builds only the Dockerfile canonical's target head holds, which
only the operator's `yard sync` can change, from a build context the daemon
materialises from that commit.

## Projects

`yard init` writes `.yard/config.toml`, `.yard/Dockerfile`, a
`.dockerignore` block and the operator skills, and registers the project with
the machine's daemon. `.yard/` is versioned and the operator's alone: it
changes only through `yard sync`, and a candidate whose diff touches it is
refused at the candidate boundary and returned to the implementer with the
reason. A worker that needs a package or a gate proposes the change.
`.yard/local/` is ignored and holds the store and live state; losing it
loses the backlog.

Configuration is one TOML document. Unknown keys are errors. Every
cross-reference resolves at load; an unknown name is a load error.

| Key | Meaning |
|---|---|
| `max_lanes` | live attempts at once on this project |
| `approve` | `manual` (default) or `auto` |
| `[target]` | `ref`, `protected_paths` |
| `[isolation]` | `dockerfile`, `egress` (hosts implementer and gate boxes may reach beyond their routes, typically package registries) |
| `[agents.<name>]` | `harness` (`pi`), `provider`, `model`, `effort`; a key the harness has no control for is a load error |
| `[workflows.<name>]` | `implementer`, `review` (seat names in panel order, or `none`), `instructions`, `access` (`write` or `read-only`), `max_session_executions`, `inactivity_timeout_minutes`, `total_work_timeout_minutes`; every workflow inherits unset keys from `default` |
| `[gates.<name>]` | `command`, `timeout_minutes`, `stage` (`landing` default, or `candidate`), `runs_in` (`box` default, or `host`), `env` (host variables a host gate receives, by name) |
| `[review]` | `max_rounds`, `timeout_minutes`, `blocking` (P0–P3) |
| `[review.seats.<name>]` | `agent`, `instructions` |

There is one configuration: canonical's target head's. Every execution
reads it when it starts. An attempt freezes only its implementer at
admission, for session continuity. Two digests bind verification: the
**gate digest** over the Dockerfile and every gate with its `stage`, `runs_in` and `env` names, and the
**review digest** over the workflow's panel, each seat's agent settings and
instructions, and `review.blocking`. A sync that changes a digest
supersedes every in-flight check taken under the old one, and only those: a
new seat re-reviews queued candidates and reruns no gate; a Dockerfile bump
reruns gates and no review (G2).

**Verification identity.** What a judgment binds is written as data:

```
candidate       = attempt, base, head
judgment input  = candidate, ticket revision, gate digest, review digest
check           = judgment input, kind, verdict, image id
approval        = judgment input, the checks it requires, actor
```

A ticket edit, a new head, or a new digest each make a new judgment
input; a check or approval recorded under an old one is superseded and
counts for nothing.
`.yard/Dockerfile` bytes are policy; the image id a box reports is the
environment used, recorded on every check.

`operator.env` (`~/.config/yard/operator.env`, mode 0600) holds the
machine's credentials, one per connection, and the machine's own settings:
`YARD_MAX_LANES` caps live attempts across every project on the machine;
`YARD_BOX_MEMORY` gives each box a share of memory. Neither is a project key.

**Connections.** An agent's `provider` names a connection: an upstream
origin and the header its key rides in. Yard hands the key to `box up` as a
`from` variable read from `operator.env`, pinfold's injecting route adds it
on the host side, and the harness is pointed at the route with a
placeholder. Subscription logins are not a Yard concern: they arrive as
pinfold login routes (pinfold #62), which Yard names and never reads.

## Tickets

A ticket is `open`, `done` or `abandoned`, and may be `parked`. It becomes
`done` when its candidate lands, or when the operator closes it with `yard
ticket done` and a reason, which is recorded like every decision. Closing is
refused while the ticket has a live attempt or a landing intent; abandon the
attempt first. A done ticket satisfies its dependents either way. It carries
`depends_on` edges to other tickets; that is the only scheduling relation.
Edits bump its revision; a check that read an older revision no longer
counts (G2).

A ticket is **ready** when it is open, not parked, every dependency is done,
it has no live attempt, and no open proposal blocks it.

**Planning** is a workflow, not a kind: a ticket on a workflow whose
`access` is `read-only` (the scaffold names one `plan`) runs a worker that
changes no code and instead proposes child tickets and an edit to its own
body. Accepting a child adds the `parent depends_on child` edge; accepting
the edit makes the body the plan. Once the children are done the operator
either closes the ticket with `yard ticket done` or moves it to an
implementing workflow and starts it; the scheduler never starts a ticket
whose workflow is read-only after its children exist.
A replan is a ticket edit, which supersedes every check that read the old
revision.

**Proposals.** A worker proposes through `yard_propose`; a proposal is an
attention item whose acceptance runs an ordinary command against current
state: create a ticket (possibly parked), edit this ticket, link tickets. A
proposal to edit the ticket a worker is on pauses that attempt until it is
decided. Rejections stay in the audit stream. Sibling references inside one
execution's proposals resolve in acceptance order. A worker that finds its
ticket too large proposes the split and stops.

## The loop

1. **File.** `yard ticket new`, or accept a proposal.
2. **Plan.** A ticket on a read-only workflow is planned: its worker
   proposes children and the body; the operator decides before paying for
   implementation.
3. **Start.** The scheduler admits ready tickets in priority order up to
   `max_lanes`. Admission creates the attempt, freezes its workflow and
   implementer, and records the first execution before anything external
   runs, in one transaction (G4).
4. **Work.** An execution runs the implementer in a box with the clone
   mounted as the workflow's `access` says and the MCP route. A
   nudge queues one message and delivers it at the next execution boundary;
   it never interrupts. `stop` ends the execution now and keeps the tree.
   The attempt's inactivity and total-work clocks end it the same way. When
   the worker stops, a clean committed head is a candidate; a dirty clone
   goes back to the implementer with the listing; an unchanged head raises
   `stopped`. The next implementer execution resumes the session where the
   harness offers one. Context is the harness's business: Yard stages each
   harness's own automatic-compaction threshold with the launch and never
   compacts a session itself. After `max_session_executions` executions on
   one session the next starts a fresh session from a brief of ticket,
   the candidate's diff stat and the attempt's progress notes, and the count
   starts again. Spend is bounded by the attempt's total-work clock alone:
   it counts wall time from admission across every execution and every
   wait, nothing automatic resets it, and only a nudge renews it.
5. **Gate.** Candidate-stage gates run on the head in a private writable
   checkout of the exact commit, in a gate box or on the host as each gate's
   `runs_in` says, in declared order, before any review. A failure is
   believed and buys a repair execution; a gate error raises `red`. Every
   gate runs again at landing.
6. **Review.** One review round per candidate: one execution per seat,
   each in its own box with the clone read-only at the head. A seat publishes once through `yard_publish_review`;
   an execution without a publication is a review error. Findings carry
   priority, file, line, category. The verdict is derived: no finding at or
   above `blocking` is a pass. Blocking findings return to the implementer as
   a repair execution. `max_rounds` counts review rounds; at the limit the
   attempt raises `stopped:limit`. A panel of `none` is unreviewed by policy and
   as clean as a pass.
7. **Approve.** The operator sees ticket, diff, checks, review and
   protected paths. Approval binds the exact candidate and its
   checks (G2). Under `approve = "auto"` a clean, unprotected candidate is
   enqueued in the transaction that verified it, with the setting recorded as
   the actor. Reject records notes and dispatches a repair execution.
   Abandon ends the attempt.
8. **Land.** The queue takes the oldest approved candidate, merges it onto
   canonical's head (a merge commit, or a fast-forward when it already sits
   there), runs the full gate set once on the merged ref, and on green
   fast-forwards canonical to that ref under compare-and-swap, then proves
   canonical contains the candidate head (G3). A landing binds its exact
   target and merged head; nothing verified on an older target authorises a
   landing on a newer one. A red landing is that candidate's: the attempt
   gets one repair execution with the failure and the target head in its
   prompt, and a second consecutive red raises `red`, the count surviving
   the repair that made the new candidate. A gate that could not run is not
   a red: it ends the landing deciding nothing and raises `red` on it. A
   candidate that does not merge cleanly gets the same repair execution
   with the conflicting paths named; its next head is a new candidate that
   takes gates, review and approval again. A landing whose target moved
   under it is retired silently and re-queued. Approval frees the attempt's
   slot; the landing has a slot of its own.
9. **Clean.** A landed or abandoned attempt's clone, boxes, transcripts and
   logs are removed whole. Rows remain. The ticket closes at
   landing.

## Executions

Every unit of external work is an execution with kind, exact input
identity, external handle, status and outcome. Kinds: `implementation`,
`review`, `gate`, `landing`, `cleanup`. There is no other lifecycle: no
owners, leases, receipts or per-feature recovery. One lifecycle is not one
recovery predicate: a box-backed execution is proved by asking pinfold for
its box, a landing by canonical's refs, a cleanup by the directory's
absence, a host gate by its process group and that group's birth identity:
one still alive after a restart is killed by verified group and the
execution is `interrupted` like a boxed one; a landing re-queues.

A worker execution records, on its row: agent, harness and version,
provider, model, effort, the execution it resumed and its session id, start
and end, exit cause, tokens in and out, reported cost, and the reason it was
started (`first`, `repair`, `nudge`, `retry`, `dirty`, `restart`, `fresh`).
A gate execution records gate name, verdict, exit code, duration and the
box's OOM count. Every operator decision
records the exact target it acted on and the text it carried. These rows are
the statistics; nothing is derived from the audit stream.

**Intent before effect.** Before a box comes up, the execution row exists.
The box name is the execution id. A crash between intent and handle is
reconciled by asking pinfold (G5).

**Publication is durable output, not outcome.** A review's findings, a
proposal and a progress note are rows the moment the tool call
commits, whatever the execution does afterwards. A reviewer that publishes
and then crashes has published; an execution that ends without a
publication is an error. The two are recorded separately.

**Capacity.** `max_lanes` counts attempts from admission until approval,
abandonment or a candidate returning from the queue for repair, which must
reacquire a slot before its worker starts. A review round's seats run one
at a time inside the attempt's slot. The landing has one slot per project
outside `max_lanes`. `YARD_MAX_LANES` caps attempts across the machine;
boxes never exceed attempts plus landings. Host gates run one at a time
across the machine.

**Cancellation** is taking the box down. Yard never signals a process it did
not start on the host, and never signals a process inside a box. Host work
in a blocking thread cannot be aborted; it is bounded by its own timeout and
its child process is what gets killed.

## Attention

One table, four kinds. Each kind declares its payload and its exits; every
surface reads that declaration.

| Kind | Raised when | Exits |
|---|---|---|
| `approval` | a candidate is verified and `approve` is `manual`, or it touches a protected path | approve, reject, abandon |
| `proposal` | a worker proposes | accept, reject |
| `stopped` | an execution ended without a candidate advancing: `unchanged`, `failed`, `interrupted`, `timeout`, `limit` | start, nudge, abandon |
| `red` | a gate error, a review error, a second landing red, a landing that could not run, a landing intent canonical cannot decide | start, nudge, abandon |

`start` always means "try again from here". On an attempt it starts the
next implementer execution. On a landing that could not run it re-queues
the candidate under its existing approval. On an undecided landing intent it
reads canonical against the restart table again, typically after the
operator has repaired canonical by hand; while canonical still decides
nothing the item stays. A nudge applies only to an attempt.

A candidate waiting in the queue raises nothing. A landing whose target
moved raises nothing. An item is resolved in the transaction of the command that
answers it.

## Store

SQLite, WAL, foreign keys, one writer: the daemon holds an exclusive
`flock` on `store.lock` for its life. Every decision is one short
transaction that also appends its audit event (G7): admissions, execution
starts and ends, checks, approvals, landings, operator commands.
Observational updates (handles, progress, usage) are column writes with no
event. No transaction is held across external work. Nothing replays the
stream.

Tables: `ticket`, `dependency`, `attempt`, `execution`, `check`, `finding`,
`approval`, `attention`, `audit`. Findings, proposals and decision text are
rows and survive cleanup. Forward
migrations only; a store from a newer schema is refused.

Files live outside the clone while an attempt is live, under
`.yard/local/attempts/<id>/`: transcripts, gate logs, harness state.
Cleanup removes the directory. A surface that needs a log
after cleanup does not get one; it gets the rows.

## Git

Every git call is a bounded child with an explicit cwd, `GIT_CONFIG_GLOBAL`
and `GIT_CONFIG_SYSTEM` pointed at `/dev/null`, `LC_ALL=C`, a fixed
identity, and `-c` overrides that no repository configuration can undo:
`core.hooksPath=/dev/null`, `core.fsmonitor=false`, `commit.gpgsign=false`,
`tag.gpgsign=false`, `protocol.allow=never`, `protocol.file.allow=always`
(local paths the daemon validated are the only transport),
`protocol.ext.allow=never`. Canonical is a bare repository under
`.yard/local/canonical.git`. An attempt is a `--no-hardlinks` clone of
canonical on branch `yard/<ticket>/<attempt>` sharing no object store with
it; the daemon fetches a candidate's objects from the clone into canonical
and reads nothing else of the clone's repository. Workers commit; they
cannot push.

`yard sync` is the only operator ingress and egress: importing fast-forwards
canonical's target from the checkout's branch after validating the incoming
`.yard`; consuming fast-forwards the checkout's branch to canonical. Both
refuse divergence and name both heads. All canonical mutations serialise on
one queue and every ref update is compare-and-swap.

**Landing.** Merge the candidate onto canonical's head (a merge commit,
or a fast-forward when it already sits there) and verify. Then, in one
store transaction that is serialised with every command that could
invalidate it (ticket edit, abandon, reject), re-read the approval's
judgment input against current rows and record a landing intent: expected
old head, verified merged head, the candidate and approval identities. From that commit until the intent is
resolved, those commands are refused for those tickets. Then `update-ref`
with the expected old value; then `merge-base --is-ancestor` for every
candidate head; then record the landing, close the ticket and resolve the
intent. Yard never force-updates a ref and never reads command success as
a landing.

On restart, an unresolved landing intent is decided by canonical alone:

| Canonical's target | Meaning | Action |
|---|---|---|
| equals the expected old head | nothing landed | retire the intent; the candidate re-queues |
| equals the verified merged head | landed, unrecorded | record the landing once; no merge runs |
| contains the merged head | landed and moved on | record the landing once from the ancestry |
| none of these, or unreadable | unknown | keep every row, refuse the queue, raise `red` on the landing |

## Boxes

Yard builds the project image with `pinfold image build` before an
execution, from canonical's target head, in a build context the daemon
materialises from that commit, and records the image id the box reports. A box spec names mounts,
env, the harness, egress and memory.

| Box | `/workspace` | Harness state | Egress |
|---|---|---|---|
| implementer | the attempt's clone, writable, or read-only where the workflow's `access` says so | writable | model route, MCP route, `isolation.egress` |
| reviewer | the clone, read-only | its own, writable | model route, MCP route |
| gate | a private disposable checkout of the exact commit, writable | none | `isolation.egress` only |

A gate's checkout is fresh: it carries none of the implementer's ignored
files or caches, so a clean `git status` in the clone is not what a gate
runs against. The image carries the target head's dependencies; one a
candidate adds is fetched by the gate through the allowlist.

A host gate runs in the same kind of private checkout, made on the host
under the attempt's or the landing's directory, as a child process in its own group with an
explicit cwd, an environment of `PATH`, `HOME` and the variables its `env`
names, bounded output and its `timeout_minutes`. It exists for gates that
need what a box cannot give, such as a container runtime.

Harness state (Pi's session store and settings) is one
directory per attempt under its files, mounted at one fixed path, written
by the attempt's implementer executions, and removed by cleanup. A review seat gets a fresh one per execution. A fresh session
is one that resumes no execution; a harness version change ends session
continuity and the next execution is fresh.

The model route is an injecting route: the box sees a placeholder and
pinfold's proxy adds the key. The MCP route names the daemon's listener. The
harness runs from `/opt/pinfold/<name>` at the version pinfold carries.
Where `YARD_BOX_MEMORY` is set, each box gets that share, swap equal. When
a worker's stream ends without a terminal frame, `box stat` is read: an OOM
count that rose is reported as a kill by the share; a `null` count (Apple
`container`) is reported as unknown; neither is a harness failure.

The daemon holds every box it starts. On start it prunes boxes whose owner
is gone and marks each execution that recorded one as interrupted.

## Harness

Pi is the one harness. Its adapter is the launch argv that starts or
resumes a session, the normalisation of its frames into `started`,
`progress`, `finished`, `failed`, and the launch files it needs, including
Yard's staged MCP client extension. An execution proceeds to its first turn
only once the staged client has printed its registration line; whether every
granted tool is present is proved by the tool list it fetched. The outcome
is read from the terminal frame, never from the exit status alone. Yard
never parses a transcript for a verdict. A second harness brings the adapter
interface with it.

## Worker tools

The MCP listener is HTTP on loopback, reached through a pinfold route,
authenticated by a per-execution bearer. Four tools, granted by execution
kind:

| Tool | Kinds |
|---|---|
| `yard_context` — ticket, brief, base, head | implementation, review |
| `yard_progress` — one bounded note | implementation, review |
| `yard_propose` — a ticket, an edit, a link | implementation |
| `yard_publish_review` — findings; once | review |

Every payload is validated at the boundary (G6). A tool call outside the
grant is refused and recorded.

## Daemon and CLI

One daemon per machine, run by the user's service manager, serving every
registered project from one process and one scheduler tick. One unix socket
at `$XDG_RUNTIME_DIR/yard.sock`. `yard daemon install` writes and starts the
service: a systemd user unit on Linux, a launchd agent on macOS, both
running `yard daemon run` from the installed binary's absolute path. On
Linux it enables lingering so the daemon outlives the login session, and
says so where it cannot. `yard daemon uninstall` stops and removes it;
`yard daemon restart` goes through the service manager.

The daemon owns the project registry, a list of absolute project paths in
`$XDG_STATE_HOME/yard/projects`. `yard init` registers a project and `yard
project forget` removes one; a registered path that is gone is skipped and
reported by `doctor`. The CLI resolves its project from the working
directory upward to the nearest `.yard/config.toml`, or from `--project`.
Each project has its own store and its own audit sequence, so `status` and
`status --watch --since` are per project.

The API is `POST /rpc` with
`{method, params}` and one envelope `{boundary, ok, result | error}`.
Client and daemon ship as one binary; a boundary mismatch is refused with
the instruction to restart the daemon. Every mutation carries the smallest
expected identity (ticket revision, candidate, attention id) and a mismatch
is a machine-readable stale result.

```
yard init | doctor | sync | status [--watch --since SEQ] [--json]
yard daemon run | install | uninstall | status | restart
yard project list | forget
yard ticket new | show | edit | park | unpark | depend | list | done | abandon
yard attempt start | stop | nudge | approve | reject | abandon | show | diff | tail
yard proposal accept | reject
yard version
```

`status --watch` follows the audit stream from a sequence and prints one
line per event the operator subscribed to; it is the wake primitive and
the only history view. `attempt tail` prints the live transcript file as the
harness wrote it. Every
command runs without a terminal and answers `--json`. `doctor` reports what
pinfold says of itself, the service, the project image, which connections
have a credential and when each login lapses.

## Guarantees

These are the invariants: binding, cited elsewhere in this spec as G1 to
G15. Each is shown by one or more end-to-end scenarios; testing policy is in
`AGENTS.md`. Timeouts are configuration, not guarantees, and no scenario
waits one out.

| # | Guarantee | Shown by |
|---|---|---|
| 1 | Only the queue lands | A worker with its MCP bearer and its clone: `git push` fails, no RPC lands, canonical is not reachable from the box. Control: the queue lands the same candidate. |
| 2 | Judgments bind exact identity | A passed gate and review, then a new commit and a ticket edit each leave the candidate unverified; a synced gate change reruns the gate and keeps the review, a synced seat change the reverse. An approval given with `--head`, then a repair commit: the queue refuses and raises `approval` again; under `auto` a protected path still raises it. An approve naming an old candidate and an edit naming an old revision get a stale result and change no row. |
| 3 | Landing is compare-and-swap and proved | Canonical moved by hand between verify and land: the landing retires and re-queues. On green, canonical is the verified ref and contains the head. The daemon killed after `update-ref`, before the landing is recorded: on restart it is recorded once, no merge runs, the ticket closes. A ticket edit racing the landing intent is refused naming the intent. |
| 4 | Capacity holds | Two `attempt start`s race for the last slot: one wins and a ticket never has two live attempts. Two registered projects: `YARD_MAX_LANES` bounds attempts across both, and each keeps its own store. |
| 5 | Intent precedes effect, and a restart loses only the turn | The daemon killed after an execution's intent, before its box reports, and again mid-execution: on restart the execution is `interrupted`, no second box exists, the tree is kept, and `start` continues. The daemon killed while a host landing gate runs: the gate's group is gone after restart and the landing re-queues. |
| 6 | Inputs are validated at the boundary | A malformed tool payload, a TOML with an unknown key, an unknown workflow name: refused by name, nothing written. |
| 7 | A ticket lands end to end and leaves only rows | New ticket, worker commit, candidate gate, review pass, approval, green landing: canonical moves and the ticket is done. Afterwards every decision has one audit event naming its target and text, the execution rows carry tokens, cost, model and start reason, no handle event exists, the attempt directory and its boxes are gone, and another live attempt's directory and canonical are untouched. |
| 8 | Review is a publication, and bounded | A seat that exits 0 without publishing is a review error; a seat killed after publishing has published; a second publication is refused; findings below `blocking` pass; a panel of `none` reaches approval marked unreviewed; a seat that always blocks gets exactly `max_rounds` rounds, then `stopped:limit`. |
| 9 | Each implementer execution starts from the right place | A nudge mid-execution lets the execution end on its own and reaches the next prompt; `stop` delivers it sooner. A worker that leaves an untracked file gets no review and the next prompt lists the file. With `max_session_executions = 2`: the second execution resumes the first, the third resumes nothing and its prompt is the brief, the fourth resumes the third, the fifth resumes nothing. |
| 10 | The queue lands one at a time and re-judges what does not merge | Three approved candidates, the second red on its merged ref: the first lands, the second gets one repair and a second red raises `red`, the third lands on the moved target with its own gate run. A candidate that does not merge gets a repair naming the paths, and its next head takes gates, review and approval again. |
| 11 | Only the operator's sync changes `.yard` and canonical from outside | A worker commit under `.yard` comes back with the reason and no gate runs; the same change through `yard sync` is in force for the next execution. A checkout and canonical that each hold a commit the other lacks: both directions refuse naming both heads; a fast-forward passes. |
| 12 | Boxes hold nothing secret | The key is absent from the box's environment and clone; the fixture behind the injecting route receives it. |
| 13 | Every gate and seat runs where its row says | A seat that writes to `/workspace` fails and the head is unchanged. A gate box that calls the model or MCP route gets nothing, and an ignored file the implementer left is absent from its checkout; control: the worker box reaches the route. A host candidate gate runs on the head before any review and sees only the variables its `env` names; a host landing gate runs on the merged ref and never for a candidate whose approval was superseded. |
| 14 | Worker git is untrusted, and Yard stays local | A worker that plants `core.fsmonitor`, a hook and a remote in its clone and corrupts an object: the daemon's fetch runs none of them, canonical's objects are intact, the candidate is refused by name. Across G7's path the fixture standing in for the network receives only the model's requests through their route, and the daemon listens on its unix socket and the MCP listener only. |
| 15 | Plans and proposals resolve | A ticket on `plan`: its clone refuses writes, it proposes children and a body edit; accepting them blocks the parent, which the scheduler does not start once they are done; `yard ticket done` then closes it and its dependents become ready. Closing a ticket with a live attempt is refused. One execution proposing A and B, B depending on A: accepting both mints A first and B's edge names it. |

The suite starts real daemons and real boxes, so it runs on a host with the
runtime and never inside a box. In this repository it is the landing-stage
host gate, on yard-sthlm (podman) and the operator's Mac (Apple
`container`), beside `cargo fmt`, `clippy` and `cargo build` as boxed
candidate gates. GitHub runs it only in the release workflow, on both
platforms, before publishing.

## Code

Rust. One binary, `yard`, client and daemon. Linux builds are static musl.

| Target | Role |
|---|---|
| `aarch64-apple-darwin` | Mac |
| `aarch64-unknown-linux-musl`, `x86_64-unknown-linux-musl` | Linux hosts |

Dependencies: `tokio`, `hyper` with `hyper-util` and `http-body-util`,
`serde`, `serde_json`, `toml`, `rusqlite` (bundled), `nix`, `sha2`,
`getrandom` (MCP bearers), `clap`. Git and pinfold are child processes.

```
crates/yard/src/
  store/    schema.rs migrate.rs tickets.rs attempts.rs executions.rs checks.rs audit.rs
  jobs/     admit.rs supervise.rs review.rs queue.rs reconcile.rs cleanup.rs
  git.rs box.rs pi.rs mcp.rs
  daemon.rs api.rs cli.rs config.rs main.rs
crates/e2e/  src/lib.rs (harness, fake model fixture, helpers), tests/*.rs
```

## Open questions

- `replay` is out: whole-attempt disposal keeps no diff to seed from. The
  branch stays in canonical; an operator who wants it cherry-picks.
- Attachments (a writable directory a worker leaves screenshots or logs in
  for the reviewer) are out until a reviewer is seen to need one.
- Batching several candidates into one landing, with bisection, is out
  until queue wait is measured; every landing the old Yard ran held one
  candidate.
- The e2e budget with real boxes: pinfold's five minutes is the target to
  measure against.
- Whether Yard later links pinfold as a library instead of a process.
- Claude and Codex return as harnesses before the cutover, once pinfold's
  login routes (#62) carry their subscription logins. The old adapters hold
  the evidence each needs (see the rebuild plan).
- Yard-invoked compaction between executions is out until the harness's own
  threshold is measured insufficient; Pi cannot compact out of band.
- The Pi MCP client extension stays TypeScript inside the Rust binary as an
  embedded file; whether pinfold should carry it instead.
