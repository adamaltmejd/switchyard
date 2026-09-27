# Switchyard architecture

Switchyard (`yard`) turns tickets into landed code with coding agents. It
keeps a local backlog, gives one ticket to one agent in one disposable box,
runs the project's gates against exact code, has a second agent review the
result, asks the operator for the decisions that need judgment, and lands
approved work through one verified merge queue.

- **Operator:** a person, or an agent driving `yard` on their behalf. The
  operator is trusted and holds the whole host API.
- **Worker:** an agent in a pinfold box. It holds a lane-scoped MCP surface
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
_Avoid_: lane (the old name), worktree

**Candidate**: an attempt's exact `base..head` at a moment. Every check,
approval and landing binds a candidate, never an attempt.

**Execution**: one unit of external work on the one persisted lifecycle:
an implementer call, one reviewer seat, one gate command, one landing, one
cleanup. It has one identity, one external handle once it has one, and one
terminal outcome. Box-backed executions own a box; host-backed ones
(landing, cleanup) have bounded host effects and are recovered from the
effect they leave. A repair, a nudge, a retry and a
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
| harness | per-harness launch argv, frame normalisation, the staged MCP client for pi | jobs |
| mcp | the lane tool surface workers call | daemon |
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
| One merge path | Only the daemon's queue moves canonical, and only to a ref its gates passed (G1, G4). |
| Exact identity | Every check, approval and landing binds a candidate, the ticket revision and the gate and review digests; a changed identity carries nothing (G2, G3). |
| Lane capability | A worker's MCP bearer is issued per execution, scoped to its kind, revoked when the execution ends. |
| No credential in a box | Upstream keys and login tokens ride pinfold's injecting routes; a box sees a placeholder. The only secret a box holds is its own execution-scoped MCP bearer, which authenticates calls from inside it and authorises nothing else. |
| Box is the sandbox | Harnesses run fully permissive; access is what the box mounts writable. A reviewer's clone is read-only. A gate box has no egress. |
| Worker git is untrusted | A worker's clone shares no objects with canonical and its configuration is never read by the daemon: every daemon git call disables hooks, `core.fsmonitor`, signing and every transport but validated local paths. |
| No checkout writes | Yard never writes the operator's checkout except inside `yard sync` the operator ran. |
| Fail safe | Unknown liveness or outcome never duplicates work, lands, deletes or kills (G6). |

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
| `[isolation]` | `dockerfile`, `egress` (hosts a worker box may reach beyond its model) |
| `[agents.<name>]` | `harness` (`pi`, `claude`, `codex`), `provider`, `model`, `effort`; a key the harness has no control for is a load error |
| `[workflows.<name>]` | `implementer`, `review` (seat names in panel order, or `none`), `instructions`, `access` (`write` or `read-only`), `max_session_executions`, `inactivity_timeout_minutes`, `total_work_timeout_minutes`; every workflow inherits unset keys from `default` |
| `[gates.<name>]` | `command`, `timeout_minutes`, `stage` (`landing` default, or `candidate`) |
| `[review]` | `max_rounds`, `timeout_minutes`, `blocking` (P0–P3) |
| `[review.seats.<name>]` | `agent`, `instructions` |

There is one configuration: canonical's target head's. Every execution
reads it when it starts. An attempt freezes only its implementer at
admission, for session continuity. Two digests bind verification: the
**gate digest** over the Dockerfile and every gate with its `stage`, and the
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

**Connections.** An agent's `provider` names a connection: the upstream
origin, the header its credential rides in, and where the credential comes
from. Pinfold does the injecting; Yard decides the value and hands it to
`box up` as a `from` variable. A key comes from `operator.env`. A login
(Claude's subscription token, Codex's ChatGPT login) is read from the
machine's own credential file when the box comes up, with any extra header
the upstream needs, such as Codex's account id. Yard never refreshes a
login: a token that lapses before the attempt's total-work clock would end
is refused before the box starts, naming the login to renew. Each harness is
pointed at its route with a placeholder credential in its own launch files.

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

**Proposals.** A worker proposes through its lane tool; a proposal is an
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
   runs, in one transaction (G5).
4. **Work.** An execution runs the implementer in a box with the clone
   mounted as the workflow's `access` says and the lane MCP route. A
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
5. **Gate.** Candidate-stage gates run on the head in a gate box, in a
   private writable checkout of the exact commit, in declared order, before
   any review. A failure is believed and buys a repair
   execution; a gate error raises `red`. Every gate runs again at landing.
6. **Review.** One review round per candidate: one execution per seat,
   each in its own box with the clone read-only at the head. A seat publishes once through `lane_publish_review`;
   an execution without a publication is a review error. Findings carry
   priority, file, line, category. The verdict is derived: no finding at or
   above `blocking` is a pass. Blocking findings return to the implementer as
   a repair execution. `max_rounds` counts review rounds; at the limit the
   attempt raises `stopped:limit`. A panel of `none` is unreviewed by policy and
   as clean as a pass.
7. **Approve.** The operator sees ticket, diff, checks, review and
   protected paths. Approval binds the exact candidate and its
   checks (G3). Under `approve = "auto"` a clean, unprotected candidate is
   enqueued in the transaction that verified it, with the setting recorded as
   the actor. Reject records notes and dispatches a repair execution.
   Abandon ends the attempt.
8. **Land.** The queue takes the oldest approved candidate, merges it onto
   canonical's head (a merge commit, or a fast-forward when it already sits
   there), runs the full gate set once on the merged ref, and on green
   fast-forwards canonical to that ref under compare-and-swap, then proves
   canonical contains the candidate head (G4). A landing binds its exact
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
absence.

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
reconciled by asking pinfold (G6).

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
boxes never exceed attempts plus landings.

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
execution, from a build context the daemon materialises from the exact
commit, and records the image id the box reports. A box spec names mounts,
env, the harness, egress and memory.

| Box | `/workspace` | Harness state | Egress |
|---|---|---|---|
| implementer | the attempt's clone, writable, or read-only where the workflow's `access` says so | writable | model route, MCP route, `isolation.egress` |
| reviewer | the clone, read-only | its own, writable | model route, MCP route |
| gate | a private disposable checkout of the exact commit, writable | none | none |

A gate's checkout is fresh: it carries none of the implementer's ignored
files or caches, so a clean `git status` in the clone is not what a gate
runs against. Dependencies a gate needs offline come from the image.

Harness state (a session store, `CODEX_HOME`, Pi's settings) is one
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

## Harnesses

Each harness is one adapter: the launch argv that starts or resumes a
session, the normalisation of its frames into `started`, `progress`,
`finished`, `failed`, and the launch files it needs. Pi gets Yard's staged
MCP client extension; Claude and Codex get an MCP config file. An
execution proceeds to its first turn only once the adapter has evidence that
Yard's server initialised, and each adapter names its evidence: Claude's
`system/init` frame lists the server and its tools; Codex marks the server
`required`, so `thread.started` is the proof and the refusal on stderr is
the other answer; Pi's staged client prints its registration line. Whether
every granted tool is present is proved by the tool list the client fetched,
where the harness reports one, and otherwise by the first refused call. The outcome is read from the harness's terminal
frame, never from its exit status alone. Yard never parses a transcript for
a verdict.

## Lane tools

The MCP listener is HTTP on loopback, reached through a pinfold route,
authenticated by a per-execution bearer. Four tools, granted by execution
kind:

| Tool | Kinds |
|---|---|
| `lane_context` — ticket, brief, base, head | implementation, review |
| `lane_progress` — one bounded note | implementation, review |
| `lane_propose` — a ticket, an edit, a link | implementation |
| `lane_publish_review` — findings; once | review |

Every payload is validated at the boundary (G8). A tool call outside the
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
yard lane start | stop | nudge | approve | reject | abandon | show | diff | tail
yard proposal accept | reject
yard version
```

`status --watch` follows the audit stream from a sequence and prints one
line per event the operator subscribed to; it is the wake primitive and
the only history view. `lane tail` prints the live transcript file as the
harness wrote it. Every
command runs without a terminal and answers `--json`. `doctor` reports what
pinfold says of itself, the service, the project image, which connections
have a credential and when each login lapses.

## Guarantees

These are the invariants: binding, cited elsewhere in this spec as G1 to
G30. Each is shown by one or more end-to-end scenarios; testing policy is in
`AGENTS.md`. Timeouts are configuration, not guarantees, and no scenario
waits one out.

| # | Guarantee | Shown by |
|---|---|---|
| 1 | Only the queue lands | A worker with the lane bearer and the clone: `git push` fails, no RPC lands, `update-ref` on canonical from the box fails. Control: the queue lands the same candidate. |
| 2 | Checks bind a candidate and a digest | A passed gate and review; then a new commit and a ticket edit each leave the candidate unverified; a synced gate change reruns the gate and keeps the review; a synced seat change reruns the review and keeps the gate. |
| 3 | Approval binds the exact candidate | Approve with `--head`; a repair commit later; the queue refuses and raises `approval` again. Under `auto`, a protected path still raises it. |
| 4 | Landing is compare-and-swap and proved | Move canonical by hand between verify and land: the landing retires and re-queues; on green, canonical contains the head and is the verified ref. Kill the daemon after `update-ref` and before the landing is recorded: on restart the landing is recorded once, no merge runs, the ticket closes. A ticket edit racing the landing intent is refused with the intent named. |
| 5 | Admission is atomic | Two `lane start` races for the last slot: one wins; a ticket never has two live attempts. |
| 6 | Intent precedes effect and unknown fails safe | SIGKILL the daemon after intent, before the box reports: on restart the execution is `interrupted`, no second box, the tree kept. |
| 7 | Audit and rows are complete | After a full ticket: every decision has one event with its target and text; execution rows carry tokens, cost, model, reason; no `handle` events exist. |
| 8 | Inputs are validated at the boundary | A malformed tool payload, a TOML with an unknown key, an unknown workflow name: refused by name, nothing written. |
| 9 | Local only | Across the path of G10, the host fixture that stands in for the network receives nothing but the model's requests through their route, and the host's socket table shows the daemon listening on its unix socket and the MCP listener only. |
| 10 | A ticket lands end to end | New ticket → worker commits → candidate gate → review pass → approve → landing green → canonical moves → ticket done → attempt directory gone. |
| 11 | Review rounds are bounded | A seat that always blocks: exactly `max_rounds` reviews, then `stopped:limit` and no further execution. |
| 12 | A nudge waits for the boundary | Nudge mid-execution: the execution ends on its own, the tree is kept, the next execution's prompt carries the message. `stop` delivers it sooner. |
| 13 | The session cap starts fresh | With `max_session_executions = 2`, the third implementer execution resumes nothing and its prompt is the brief; the fourth resumes the third; the fifth resumes nothing. Control: the second resumed the first. |
| 14 | A dirty clone never reaches review | A worker that leaves an untracked file: no review runs; the next execution's prompt lists it. |
| 15 | A review is a publication or an error | A seat that exits 0 without publishing is `review-error`; a seat killed after publishing has published; a second publication is refused; findings below `blocking` pass. |
| 16 | Panel `none` is clean but unreviewed | The candidate reaches approval; status says unreviewed. |
| 17 | The queue lands one at a time | Three approved candidates, the second red on the merged ref: the first lands, the second gets one repair and a second red raises `red`, the third lands on the moved target with its own gate run. A gate box that cannot start ends the landing with `red` and no repair. |
| 18 | Conflicts are re-judged | A candidate that does not merge onto canonical gets a repair execution naming the paths; its next head takes gate, review and approval again. |
| 19 | `.yard` is the operator's | A worker that commits a change under `.yard` gets it back with the reason and no gate runs; the same change through `yard sync` is in force for the next execution. |
| 20 | Boxes hold nothing secret | The key is absent from the box env and from the clone; the fixture behind the injecting route receives it. |
| 21 | A reviewer cannot write, and worker git is untrusted | A seat that writes to `/workspace` fails; the candidate head is unchanged. A worker that plants `core.fsmonitor` and a hook in its clone and corrupts an object: the daemon's fetch runs neither, canonical's objects are intact, and the candidate is refused by name. |
| 22 | Gates have no egress and a fresh tree | A gate that curls the fixture route gets nothing; an ignored file the implementer left in the clone is absent from the gate's checkout; the gate writes `target/` and passes. Control: the worker box reaches the route. |
| 23 | Restart loses only the turn | Kill the daemon mid-execution: on restart the box is gone, the execution `interrupted`, the tree kept, the operator's `start` continues. |
| 24 | Cleanup removes everything and only that | After landing, the attempt directory and its boxes are gone; other attempts' directories and canonical are untouched. |
| 25 | `sync` refuses divergence | Checkout and canonical each with a commit the other lacks: both directions refuse naming both heads; a fast-forward passes. |
| 26 | A read-only workflow plans, then waits | A ticket on `plan`: its worker's clone refuses writes, it proposes children and a body edit; accepting them blocks the parent; the scheduler does not start the parent once its children are done; `yard ticket done` then closes it and its dependents become ready. Closing a ticket with a live attempt is refused. |
| 27 | Proposals resolve siblings | One execution proposes A and B with B depending on A; accepting both mints A first and B's edge names it. |
| 28 | One daemon, many projects | Two registered projects; `YARD_MAX_LANES` bounds attempts across both; each store is its own. |
| 29 | Stale identity mutates nothing | Approve with an old candidate, edit with an old revision: stale result, no row changed. |
| 30 | `--version` and `--help` need no daemon | Both answer with no socket and no store. |

The suite starts real daemons and real boxes, so it runs on a host with
the runtime: GitHub's ubuntu runners (podman) on every push to `main`, and
the operator's Mac (Apple `container`). It cannot run inside a Yard gate
box, which has no runtime, so this repository's own Yard gates are `cargo
fmt`, `clippy` and `cargo build`, and a landing here is proved by those
alone. The suite runs after the operator syncs and pushes to GitHub; a red
suite on `main` is the next ticket.

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
  git.rs box.rs harness/{pi,claude,codex}.rs mcp.rs
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
- Yard-invoked compaction between executions is out until the harness's own
  threshold is measured insufficient; Pi cannot compact out of band.
- The Pi MCP client extension stays TypeScript inside the Rust binary as an
  embedded file; whether pinfold should carry it instead.
