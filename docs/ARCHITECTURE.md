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

**Lane**: one of the places attempts run through. `max_lanes` and
`YARD_MAX_LANES` count them; see Capacity for when an attempt holds one.

**Candidate**: an attempt's exact `base..head` and proof snapshot at a
moment. Every check, approval and landing binds a candidate, never an
attempt.

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

**Agent**: a named way of running a model: harness, provider connection or
subscription login, model, effort.

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
| harness | every harness's launch argv, frame normalisation, registration proof and staged MCP client | jobs |
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
| No credential in a box | Upstream keys and login tokens ride pinfold's injecting and login routes; a box sees a placeholder. The only secret a box holds is its own execution-scoped MCP bearer, which authenticates calls from inside it and authorises nothing else. |
| Box is the sandbox | The harness runs fully permissive; access is what the box mounts writable, and Yard's own writes into a writable mount follow no link the box made. A reviewer reads a fresh read-only checkout of the head. A gate box reaches only the project's allowlist: no model, no MCP, no credential. |
| Worker git is untrusted | A worker's clone shares no objects with canonical and the daemon runs no git command in it; a candidate arrives by a fetch run in canonical, and gates and reviewers read a fresh checkout of it. Every daemon git call disables hooks, `core.fsmonitor`, signing and every transport but validated local paths. |
| Worker proof is untrusted | The proof directory is worker-written. Yard copies it with no link followed, refuses any entry that is not a regular file or a directory by name, and bounds its size and file count. |
| No checkout writes | Yard never writes the operator's checkout except inside `yard sync` the operator ran. |
| Gates run where the project says | A project chooses per gate whether it runs in a box or on the host. A host gate runs agent-written code with the operator's privileges and no containment: before any review at the candidate stage, after approval at landing. Which gates accept that is the project's decision. |
| Fail safe | Unknown liveness or outcome never duplicates work, lands, deletes or kills (G5). |

Everything pinfold's programmatic core guarantees (no network but the proxy,
no privilege, the environment is the spec) Yard relies on and does not
restate. Pinfold's `.git` protection belongs to its interactive layer, not
to caller-owned boxes; Yard's own mounts are the answer above. Image builds
are outside pinfold's sandbox: trusted, with unrestricted egress. Yard
therefore builds only from canonical's target head: `.yard/Dockerfile`,
which only the operator's `yard sync` changes, in a build context the daemon
materialises from that commit. The rest of that context is landed code,
which approval has already passed; the Dockerfile can run it at build time.

## Projects

`yard init` writes `.yard/config.toml`, `.yard/Dockerfile`, a
`.dockerignore` block and the operator skills, and registers the project with
the machine's daemon. `.yard/` is versioned and the operator's alone: it
changes only through `yard sync`, and a candidate whose diff touches it is
refused at the candidate boundary and returned to the implementer with the
reason. A worker that needs a package or a gate proposes the change.
`.yard/local/` is ignored through `.yard/.gitignore` and holds the store and live state; losing it
loses the backlog. The scaffold's `protected_paths` names the files that
guide workers, `AGENTS.md`, `CLAUDE.md`, `.agents/` and `.pi/`, so a
candidate changing them always needs the operator's approval, under
`approve = "auto"` too.

Configuration is one TOML document. Unknown keys are errors. Every
cross-reference resolves at load; an unknown name is a load error.

| Key | Meaning |
|---|---|
| `max_lanes` | live attempts at once on this project |
| `approve` | `manual` (default) or `auto` |
| `[target]` | `ref`, `protected_paths` |
| `[isolation]` | `egress` (hosts implementer and gate boxes may reach beyond their routes, typically package registries) |
| `[agents.<name>]` | `harness` (`pi`, `claude`, `codex`), `provider` or `login`, `model`, `effort`; a key the harness has no control for is a load error |
| `[workflows.<name>]` | `implementer`, `review` (seat names in panel order, or `none`), `instructions`, `access` (`write` or `read-only`), `max_session_executions`, `inactivity_timeout_minutes`, `total_work_timeout_minutes`; every workflow inherits unset keys from `default` |
| `[gates.<name>]` | `command`, `timeout_minutes`, `stage` (`landing` default, or `candidate`), `runs_in` (`box` default, or `host`), `env` (host variables a host gate receives, by name) |
| `[review]` | `max_rounds`, `timeout_minutes`, `blocking` (P0–P3) |
| `[review.seats.<name>]` | `agent`, `instructions` |

There is one configuration: canonical's target head's. Every execution
reads it when it starts. An attempt freezes its workflow's name and its
implementer's settings at admission, for session continuity; every other
workflow key is read current. A sync that removes a workflow an open ticket
names is refused, naming the tickets. Two digests bind verification: the
**gate digest** over the Dockerfile and every gate with its `stage`, `runs_in` and `env` names, and the
**review digest** over the workflow's panel, each seat's agent settings and
instructions, and `review.blocking`. A sync that changes a digest
supersedes every in-flight check taken under the old one, and only those: a
new seat re-reviews queued candidates and reruns no gate; a Dockerfile bump
reruns gates and no review (G2).

**Verification identity.** What a judgment binds is written as data:

```
candidate     = attempt, base, head, proof
gate input    = candidate, ticket revision, gate digest
review input  = candidate, ticket revision, review digest
check         = its kind's input, verdict, image id
approval      = candidate, ticket revision, both digests, the checks it requires, actor
```

A ticket edit or a new head makes every input new; a new digest makes only
its own kind's input new. A changed proof with an unchanged head is a new
candidate. A check whose input no longer matches is
superseded and counts for nothing; one whose input still matches keeps
counting, and its row is never rewritten. An approval is superseded when
any of its parts is.

`approve` and `protected_paths` are policy in no digest. A landing re-reads
its approval against current rows and policy before its first gate and
again when it records its intent: an automatic approval that the current
policy would not give (the candidate now touches a protected path, or
`approve` is now `manual`) is withdrawn, the candidate leaves the queue, and
`approval` is raised. No check reruns.
`.yard/Dockerfile` bytes are policy; the image id a box reports is the
environment used, recorded on every execution, which each check reads through
its execution. A check row holds only its verdict and reads its input from
its execution.

`operator.env` (`$XDG_CONFIG_HOME/yard/operator.env`, `~/.config` when
unset, mode 0600) holds the
machine's credentials, one per connection or login, and the machine's own settings:
`YARD_MAX_LANES` caps live attempts across every project on the machine;
`YARD_BOX_MEMORY` gives each box a share of memory. Neither is a project key.

**Connections.** An agent's `provider` names a connection: an upstream
origin and the header its key rides in. Yard knows two, each with its key
variable: `openrouter` (`https://openrouter.ai/api/v1`,
`OPENROUTER_API_KEY`) and `opencode-go` (`https://opencode.ai/zen/go/v1`,
`OPENCODE_API_KEY`), both as `Authorization: Bearer`. `operator.env` may set
`YARD_ORIGIN_<NAME>` (`YARD_ORIGIN_OPENROUTER`) to another origin, `http`
for a host service such as a local gateway. Yard hands the key to `box up` as a
`from` variable read from `operator.env`, pinfold's injecting route adds it
on the host side, and the harness is pointed at the route with a
placeholder. Pi takes one base URL per provider, so a Pi agent's model must
be one Pi serves over its connection's OpenAI-compatible API; a model Pi's
catalog serves over another API (OpenRouter's `anthropic/*`) cannot run
through the route.

**Logins.** An agent's `login = true` reaches a subscription through a
pinfold login route. Yard knows two. `claude` runs over pinfold's `claude`
route, and `operator.env` holds its `CLAUDE_CODE_OAUTH_TOKEN`, the `claude
setup-token` token. Yard hands the token to `box up` as the route's `from`
variable; pinfold sets `ANTHROPIC_BASE_URL` and a placeholder
`CLAUDE_CODE_OAUTH_TOKEN` in the box and adds `Authorization: Bearer` on
the host side. `operator.env` may set `YARD_ORIGIN_CLAUDE` to another
origin. `codex` runs over pinfold's `codex` route and uses the host's own
Codex login: pinfold's host helper reads `CODEX_HOME` from `box up`'s
environment and resolves the login itself, writes the box's codex config
(`model_provider = "pinfold"`, `wire_api = "responses"`, the route's base
URL, `requires_openai_auth = false`), and the route adds `Authorization:
Bearer` and `ChatGPT-Account-ID` on the host side. `operator.env` may set
`YARD_ORIGIN_CODEX` to another origin. Yard never reads either token, and
pinfold exposes no lapse; an unusable Codex login is `box up`'s `login`
refusal.

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
implementing workflow and starts it. An execution on a read-only workflow
ends its attempt when it stops: its proposals are its outcome, and an
unchanged head raises nothing. The scheduler starts a ticket on a read-only
workflow once; after that only `yard attempt start` does.
A replan is a ticket edit, which supersedes every check that read the old
revision.

**Proposals.** A worker proposes through `yard_propose`; a proposal is an
attention item whose acceptance runs an ordinary command against current
state: create a ticket (possibly parked), edit this ticket, link tickets. An
edit proposal carries the revision it read. A proposal to edit the ticket a
worker is on pauses that attempt until it is decided. Rejections stay in the
audit stream. Sibling references inside one execution's proposals resolve in
acceptance order. A worker that finds its ticket too large proposes the
split and stops.

## The loop

1. **File.** `yard ticket new`, or accept a proposal.
2. **Plan.** A ticket on a read-only workflow is planned: its worker
   proposes children and the body; the operator decides before paying for
   implementation.
3. **Start.** The scheduler admits ready tickets in priority order up to
   `max_lanes`. Admission creates the attempt, freezes its workflow and
   implementer, and records the first execution before anything external
   runs, in one transaction (G4). `yard attempt start` is the same
   admission for one named ticket, now: refused, naming the reason, when
   the ticket is not ready or no lane is free. On a ticket whose live
   attempt has a `stopped` or `red` item it is that item's `start`.
4. **Work.** An execution runs the implementer in a box with the clone
   mounted as the workflow's `access` says and the MCP route. A
   nudge queues one message and delivers it at the next execution boundary;
   it never interrupts. A nudge queued while the implementer runs starts it
   again when that execution ends, before anything judges its candidate. `stop` ends the execution now and keeps the tree.
   The attempt's inactivity and total-work clocks end it the same way. When
   the worker stops, Yard asks git inside its box whether the clone is
   clean before the box comes down: a clean committed head is a candidate;
   a dirty clone goes back to the implementer once with the listing, and
   raises `stopped:dirty` if still dirty; an unchanged head raises
   `stopped`. The next implementer execution resumes the session where the
   harness offers one. Context is the harness's business: Yard stages each
   harness's own automatic-compaction threshold with the launch and never
   compacts a session itself. After `max_session_executions` executions on
   one session the next starts a fresh session from a brief of ticket,
   the candidate's diff stat and the attempt's progress notes, and the count
   starts again. Spend is bounded by the attempt's total-work clock alone:
   it counts wall time while the attempt holds a lane, across every
   execution and every wait, nothing automatic resets it, and only a nudge
   renews it. Expiry ends the running execution as `timeout` and starts
   nothing automatic. An approved candidate holds no lane, so expiry never
   touches an approval, the queue or a landing.
5. **Gate.** Candidate-stage gates run on the head in a private writable
   checkout of the exact commit, in a gate box or on the host as each gate's
   `runs_in` says, in declared order, before any review. A failure is
   believed and buys a repair execution; a gate error raises `red`. Every
   gate runs again at landing.
6. **Review.** One review round per candidate: one execution per seat,
   each in its own box reading a fresh read-only checkout of the head. A seat publishes once through `yard_publish_review`;
   an execution without a publication is a review error. Findings carry
   priority, file, line, category. The verdict is derived: no finding at or
   above `blocking` is a pass. Blocking findings return to the implementer as
   a repair execution. `max_rounds` counts review rounds; at the limit the
   attempt raises `stopped:limit`. A panel of `none` is unreviewed by policy and
   as clean as a pass. A publication counts from the moment it commits, but
   the next seat, the verdict and whatever follows it wait until the
   publishing execution has ended and its box is gone.
7. **Approve.** The operator sees ticket, diff, checks, review and
   protected paths. Approval binds the exact candidate and its
   checks (G2). Under `approve = "auto"` a clean, unprotected candidate is
   enqueued in the transaction that verified it, with the setting recorded as
   the actor. Reject records notes and dispatches a repair execution.
   Abandon ends the attempt.
8. **Land.** The queue lands the oldest approved candidate whose approval
   still holds (see Projects), running the full gate set once on the merged
   ref (see Git, G3). A landing binds its exact
   target and merged head; nothing verified on an older target authorises a
   landing on a newer one. A red landing is that candidate's: the attempt
   gets one repair execution with the failure in its prompt and the target
   made available to it (see Git), and a second consecutive red raises `red`, the count surviving
   the repair that made the new candidate. A gate that could not run is not
   a red: it ends the landing deciding nothing and raises `red` on it. A
   candidate that does not merge cleanly gets the same repair execution
   with the conflicting paths named and the target made available; its next head is a new candidate that
   takes gates, review and approval again. A landing whose target moved
   under it is retired silently and re-queued.
9. **Clean.** A landed or abandoned attempt's clone, boxes, transcripts and
   logs are removed whole. Rows remain. The ticket closes at
   landing.

## Executions

Every unit of external work is an execution with kind, exact input
identity, external handle, status and outcome. Kinds: `implementation`,
`review`, `gate`, `landing`, `cleanup`. There is no other lifecycle: no
owners, leases, receipts or per-feature recovery. One lifecycle is not one
recovery predicate: a box-backed execution is proved by asking pinfold for
its box, a landing by canonical's refs once its host children are over, a
cleanup by the directory's absence, a host execution by its lock (below).

**Host children are accounted for.** Before an execution spawns a host
child (a landing's ref update and its ancestry proof, a host gate), it opens a lock file under the
attempt's or the landing's directory and takes `flock` on it, and every
such child inherits that descriptor. The daemon's copy dies with the
daemon; a child's lives as long as the child. On restart the daemon takes
each unfinished host execution's lock without waiting before reading any
effect: taken proves no old child can still act. Held after the recorded
host gate group is killed, or held by a git child, which is never killed
and ends on its own, leaves the execution unresolved: a landing intent
stays, the queue is refused and `red` is raised. The recorded handle is for
killing; the lock is the proof, so a child spawned but not yet recorded is
not missed. A host gate's command starts only once its handle is recorded,
so a restart kills every group that ran a command. A host gate still alive
after a restart is killed by its verified group and the execution is
`interrupted` like a boxed one; a landing re-queues.

A worker execution records, on its row: agent, harness and its version
(pinfold's pin, read from `pinfold artifacts` once at daemon start, which
fails if a harness has no pin), provider,
model, effort, the execution it resumed and its session id, start
and end, exit cause, tokens in and out, reported cost, and the reason it was
started (`first`, `repair`, `nudge`, `retry`, `dirty`, `restart`).
A gate execution records gate name, verdict, exit code, duration and the
box's OOM count. Every operator decision
records the exact target it acted on and the text it carried. These rows are
the statistics; nothing is derived from the audit stream.

**Intent before effect.** Before a box comes up or a host child starts, the
execution row exists. The box name is the execution id, and a crash between
intent and handle is reconciled by asking pinfold; a host child is found by
its lock (G5).

**Publication is durable output, not outcome.** A review's findings, a
proposal and a progress note are rows the moment the tool call
commits, whatever the execution does afterwards. A review publication is a
decision: one transaction records its findings and the seat's check, with
its audit event. Execution end records only the outcome.

**Capacity.** `max_lanes` counts attempts from admission until approval,
abandonment or a candidate returning from the queue for repair, which must
reacquire a lane before its worker starts. A review round's seats run one
at a time inside the attempt's lane. Each project's landing has its own
lane outside `max_lanes`. `YARD_MAX_LANES` caps attempts across the machine;
boxes never exceed attempts plus landings. When a lane frees, the scheduler
gives it to a live attempt waiting to reacquire one before admitting a new
ticket.

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
| `stopped` | an execution ended without a candidate advancing: `unchanged`, `dirty`, `failed`, `interrupted`, `timeout`, `limit` | start, nudge, abandon; on `timeout` and `limit` only nudge, abandon |
| `red` | a gate error, a review error, a second landing red, a landing that could not run, a landing intent canonical cannot decide, an execution that could not record its own end (`error`) | start, nudge, abandon; on a landing's item only start |

`start` always means "try again from here". After an implementer stop it
starts the next implementer execution. On a gate or review error it reruns
that check on the same candidate. On a landing that could not run it
re-queues the candidate under its existing approval. On an undecided landing
intent it reads canonical against the restart table again, typically after
the operator has repaired canonical by hand; while canonical still decides
nothing the item stays. On `timeout` a nudge renews the total-work clock; on
`limit` it allows one more review round; either way its text reaches the
next implementer execution.

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

The events, each naming its ticket, attempt, execution or attention target:
`sync.imported`, `sync.consumed`; `ticket.new`, `ticket.edited`,
`ticket.parked`, `ticket.unparked`, `ticket.linked`, `ticket.done`,
`ticket.abandoned`; `attempt.admitted`, `attempt.candidate`,
`attempt.stopped`, `attempt.nudged`, `attempt.abandoned`, `attempt.ended`;
`execution.started`, `execution.ended`; `check.recorded`; `approval.given`,
`approval.ended`; `landing.intent`, `landing.recorded`; `attention.raised`,
`attention.resolved`; `tool.refused`.

Tables: `ticket`, `dependency`, `attempt`, `execution`, `check`, `finding`,
`approval`, `attention`, `audit`. Findings, proposals and decision text are
rows and survive cleanup. Forward
migrations only; a store from a newer schema is refused.

Files live outside the clone while an attempt is live, under
`.yard/local/attempts/<id>/`: transcripts, gate logs, harness state, the
worker's live `proof/` directory and its `proof-snapshots/<digest>` copies.
Cleanup removes the directory. A surface that needs a log
after cleanup does not get one; it gets the rows.

## Git

Every git call is a bounded child with an explicit cwd, `GIT_CONFIG_GLOBAL`
and `GIT_CONFIG_SYSTEM` pointed at `/dev/null`, `LC_ALL=C`, a fixed
identity, and `-c` overrides that no repository configuration can undo:
`core.hooksPath=/dev/null`, `core.fsmonitor=false`, `commit.gpgsign=false`,
`tag.gpgsign=false`, `protocol.allow=never`, `protocol.file.allow=always`
(local paths the daemon validated are the only transport),
`protocol.ext.allow=never`. Git and pinfold are found on the daemon's `PATH`. Canonical is a bare
repository under `.yard/local/canonical.git`. An attempt is a
`--no-hardlinks` clone of canonical on branch `yard/<ticket>/<attempt>`
sharing no object store with it. The daemon runs no git command with its
cwd in a clone. It takes a candidate's objects by a fetch run
in canonical from the clone's path, which git serves under its rules for
untrusted repositories, takes nothing in the clone's configuration as
authority, and refuses a malformed clone by name. Workers commit;
they cannot push.

**A repair sees its target.** A repair after a red or conflicting landing
names the target it failed on. The daemon writes a bundle of that target
into the attempt's files, mounted read-only in the implementer's box, and
the worker fetches from it; canonical itself is never mounted. A
candidate's base is the daemon's: the head at admission, then the latest
target handed to the attempt once that target is an ancestor of the head.
The `.yard` refusal and protected paths read `base..head`, so a merge of a
target that carries an operator's `.yard` change passes, and a head that
undoes one is refused.

`yard sync` is the only operator ingress and egress: importing fast-forwards
canonical's target from the checkout's branch after validating the incoming
`.yard`; consuming fast-forwards the checkout's branch to canonical. Both
refuse divergence and name both heads. All canonical mutations serialise on
one queue, and so does each re-read of a landing's policy with what it
admits, so no sync falls between them. Every ref update is compare-and-swap.

**Landing.** Merge the candidate onto canonical's head (a merge commit,
or a fast-forward when it already sits there) and verify. Then, in one
store transaction that is serialised with every command that could
invalidate it (ticket edit, abandon, reject), re-read the approval
against current rows and policy and record a landing intent: expected
old head, verified merged head, the candidate and approval identities. From that commit until the intent is
resolved, those commands are refused for those tickets. Then `update-ref`
with the expected old value; then `merge-base --is-ancestor` for the
candidate head; then record the landing, close the ticket and resolve the
intent. Yard never force-updates a ref and never reads command success as
a landing.

On restart, once the landing's lock proves its git children are over, an
unresolved landing intent is decided by canonical alone:

| Canonical's target | Meaning | Action |
|---|---|---|
| equals the expected old head | nothing landed | retire the intent; the candidate re-queues |
| equals the verified merged head | landed, unrecorded | record the landing once; no merge runs |
| contains the merged head | landed and moved on | record the landing once from the ancestry |
| none of these, or unreadable | unknown | keep every row, refuse the queue, raise `red` on the landing |

## Boxes

Yard builds the project image with `pinfold image build` before an
execution (see Threat model) and records the image id the box reports. A
box spec names mounts, env, the harness, egress and memory.

| Box | `/workspace` | Harness state | Egress | Env |
|---|---|---|---|---|
| implementer | the attempt's clone, writable, or read-only where the workflow's `access` says so; `/yard/proof` writable (no proof mount when read-only) | writable | model route, MCP route, `isolation.egress` | the harness's env, git identity, `YARD_MCP_BEARER` (from) |
| reviewer | a fresh checkout of the head, read-only; `/yard/proof` is the candidate's snapshot, read-only | its own, writable | model route, MCP route | the harness's env, git identity, `YARD_MCP_BEARER` (from) |
| gate | a private disposable checkout of the exact commit, writable; `/yard/proof` is the candidate's snapshot, read-only | none | `isolation.egress` only | `HOME`, `PATH`, `YARD_BASE` |

Every gate gets `YARD_BASE`, the commit its change is judged against: at
the candidate stage the candidate's base, at landing the target head the
candidate was merged onto. The checkout is a full clone, so the base
commit is present and `git diff --name-only "$YARD_BASE" HEAD` lists
exactly the candidate's change at either stage.

Gate and reviewer checkouts are fresh, made by the daemon from canonical's
objects: they carry none of the implementer's ignored files, caches or
excluded guidance, so what they read is exactly the candidate. The image carries the target head's dependencies; one a
candidate adds is fetched by the gate through the allowlist.

A host gate runs in the same kind of private checkout, made on the host
under the attempt's or the landing's directory, as a child process in its own group with an
explicit cwd, an environment of `PATH`, `HOME`, `YARD_BASE`, `YARD_PROOF` and the
variables its `env` names, bounded output and its `timeout_minutes`. It exists for gates that
need what a box cannot give, such as a container runtime. `YARD_PROOF` is
the host path of the proof snapshot this execution judges: the candidate's
at the candidate stage, the approved candidate's at landing.

Harness state (a harness's session store and launch files) is one
directory per attempt under its files, mounted at one fixed path, written
by the attempt's implementer executions, and removed by cleanup. A review seat gets a fresh one per execution. A fresh session
is one that resumes no execution; a harness version change ends session
continuity and the next execution is fresh.

The model route is an injecting route: the box sees a placeholder and
pinfold's proxy adds the key. The MCP route names the daemon's listener. The
harness runs from `/opt/pinfold/<name>` at the version pinfold carries.
Where `YARD_BOX_MEMORY` is set, each box gets that share, swap equal. When
a worker's stream ends without a terminal frame, `box stat` is read: an OOM
count that rose is recorded as `oom`, a kill by the share, and the worker
stops as `failed`; a `null` count (Apple `container`) is recorded as
unknown; neither is a harness failure.

The daemon holds every box it starts. On start it prunes boxes whose owner
is gone and marks each execution that recorded one as interrupted.

## Harness

One adapter interface serves every harness: `name`, `version` (pinfold's
pin), `login` where the harness is a subscription login, `accepts`,
`stage`, which writes the launch files, including the harness's
automatic-compaction config, into the attempt's harness state and read-only
input dir, `env`, `route`, `argv` and `reader`, a per-run reader over stdout
and stderr yielding `Started`, `Finished`, `Failed`, `Registered` and
`Refused`. Pi, Claude and Codex are the three.

Pi's launch argv starts or resumes a session, the normalisation of its
frames yields `started`, `finished` and `failed`, and its launch files
include Yard's staged MCP client extension and `agent/models.json`, which
moves the provider onto its route. Extension discovery is off, and so are
prompt templates and themes: the staged client is the only code that loads
in the harness, so nothing a candidate commits runs with a worker's bearer.
Context files (`AGENTS.md`, `CLAUDE.md`) load as Pi finds them in
`/workspace`, so every worker reads the project's own rules. Project skills
and `.pi/` resources stay behind Pi's project trust, which Yard never
grants. Pi's user level is the attempt's harness state, which Yard owns and
leaves empty.

Claude runs as a login: its launch files hold a `--mcp-config` naming only
the `yard` HTTP server, whose bearer comes from the env, and its
automatic-compaction setting. `--strict-mcp-config` and `--setting-sources
user` keep every committed MCP config, setting and hook from loading; a
project plugin loads only through those settings and is kept out with them.
`--setting-sources user` also stops Claude loading the project's own memory,
so Yard reads the checkout's root `AGENTS.md` and `CLAUDE.md` on the host,
follows no link, refuses a non-regular file and bounds each to 64 KiB, and
appends them to Claude's system prompt. A seat reads them from its fresh
checkout of the head; the implementer reads them from its clone. The
`system`/`init` frame
names the session id and the tools the server registered, and that is the
registration proof. The `result` frame is the outcome. The model and effort
ride the argv, `--resume` continues a session, and the token stays a pinfold
placeholder in the box.

Codex runs as a login too: pinfold's config carries the model provider and
route, and the launch stages the required `yard` HTTP MCP server, whose
bearer comes from the env, and its automatic-compaction setting as config
overrides, so a worker's own files cannot replace the provider. Codex reads
`AGENTS.md` from `/workspace` itself and no committed project config, MCP
server, hook or profile. Its `exec --json` stream starts a
thread with `thread.started`, which names the session and proves the
required server connected; `turn.completed` is the outcome and `turn.failed`
or `error` names the failure. `resume <thread>` continues a session, `-s
danger-full-access` is the sandbox, and the model and effort ride the argv.
A panic exits 0, so the outcome is the terminal frame.

An execution proceeds to its first turn only once the registration proof is
seen: the reader yields it only when the session id is non-empty, every
granted tool is in the fetched list and, for a login harness, the Yard
server is `connected`. A harness whose MCP server is `required` lists no
tools; it records the tools the server serves, and the terminal turn checks
that the harness reached the server, since neither the harness's stdout nor
Yard's listener orders the other. Yard records the proof on the
execution row at once;
a refused proof ends the run. A review check counts only from an execution
row that records the proof, so a seat that never proved itself never counts,
before or after a restart, and a seat interrupted after its proof keeps its
publication. The outcome is read from the terminal frame, never from the
exit status alone. Yard never parses a transcript for a verdict.

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
at `$XDG_STATE_HOME/yard/yard.sock`, which must stay under macOS's 104-byte
socket path limit. The daemon passes the host's `XDG_RUNTIME_DIR` and
`DBUS_SESSION_BUS_ADDRESS` to every pinfold child. On start the daemon reconciles
every project, the landing restart table and the box prune included, and
serves no command until that has committed. `yard daemon run` then prints
one JSON line, `{"event": "serving", "socket", "pid", "boundary"}`, to
stdout. `yard daemon install` writes and starts the
service: a systemd user unit on Linux, a launchd agent on macOS, both
running `yard daemon run` from the installed binary's absolute path. On
Linux it enables lingering so the daemon outlives the login session, and
says so where it cannot. `yard daemon uninstall` stops and removes it;
`yard daemon restart` goes through the service manager.

The daemon owns the project registry, a list of absolute project paths in
`$XDG_STATE_HOME/yard/projects`. `yard init` registers a project and `yard
project forget` removes one; a registered path that is gone is skipped and
reported by `doctor`. The CLI resolves its project from the working
directory upward to the nearest `.yard/config.toml`, or from `--project`;
a linked git worktree resolves to its main checkout when that is
registered, and `yard init` refuses a linked worktree, naming the main
checkout.
Each project has its own store and its own audit sequence, so `status`,
`status --watch` and `status --history` are per project.

The API is `POST /rpc` with
`{method, params}` and one envelope `{boundary, ok, result | error}`.
Client and daemon ship as one binary; a boundary mismatch is refused with
the instruction to restart the daemon. Every mutation carries the smallest
expected identity (ticket revision, candidate, attention id) and a mismatch
is a machine-readable stale result.

```
yard init | doctor | sync | status [--watch | --history [--since SEQ]] [--json]
yard daemon run | install | uninstall | status | restart
yard project list | forget
yard ticket new | show | edit | park | unpark | depend | list | done | abandon
yard attempt start | stop | nudge | approve | reject | abandon | show | diff | tail
yard proposal accept | reject
yard version
```

`status --watch` returns as soon as an attention item is open, prints the
open items and exits; it is the wake primitive. `status --history` follows
the audit stream from a sequence and prints one line per event; it is the
only history view. `attempt tail` prints the live transcript file as the
harness wrote it. Every
command runs without a terminal and answers `--json`. `doctor` reports what
pinfold says of itself, the service, the project image, which connections
have a credential, and each configured login's pin and, where Yard holds its
token, whether it is set.

## Guarantees

These are the invariants: binding, cited elsewhere in this spec as G1 to
G15. Each is shown by one or more end-to-end scenarios; testing policy is in
`AGENTS.md`.

| # | Guarantee | Shown by |
|---|---|---|
| 1 | Only the queue lands | A worker with its MCP bearer and its clone: `git push` fails, no RPC lands, canonical is not reachable from the box. Control: the queue lands the same candidate. |
| 2 | Judgments bind exact identity | A passed gate and review, then a new commit and a ticket edit each leave the candidate unverified; a synced gate change reruns the gate and keeps the review, a synced seat change the reverse. A repair that changes only the proof snapshot, with an unchanged head, is a new candidate: its gate reruns and the old check does not count. An approve or reject names the head and, when the candidate has a proof snapshot, its digest; a delayed answer naming the old digest is stale and writes no row, and the new digest answers. An approval given with `--head`, then a repair commit: the approval does not carry and `approval` is raised again; under `auto` a protected path still raises it. An automatic approval, then a sync that protects its path or sets `approve = "manual"`: the landing withdraws it, raises `approval` and reruns no check; with the attempt abandoned while the landing re-reads it, nothing is raised. An approve naming an old candidate, an edit naming an old revision, and an accepted edit proposal whose ticket moved each get a stale result and change no row. |
| 3 | Landing is compare-and-swap and proved | Canonical moved by hand between verify and land: the landing retires and re-queues. On green, canonical is the verified ref and contains the head. The daemon killed after `update-ref`, before the landing is recorded: before restart canonical is the merged head, the intent is unresolved and no landing is recorded; on restart it is recorded once, no merge runs, the ticket closes. The daemon killed while its `update-ref` is held: restart keeps the intent and raises `red`; once the command is released, `start` records the landing once. A ticket edit racing the landing intent is refused naming the intent. |
| 4 | Capacity holds | Two `attempt start`s race for the last lane: one wins and a ticket never has two live attempts. Two registered projects: `YARD_MAX_LANES` bounds attempts across both, and each keeps its own store; an execution ending in one leaves the other's execution of the same id its bearer. A candidate returned from the queue for repair takes a freed lane before a ready ticket is admitted. |
| 5 | Intent precedes effect, and a restart loses only the turn | Intent is ordered before effect: an execution's audit event precedes its box's creation time in `pinfold box list` and the fixture's first request. The daemon killed mid-execution: on restart the execution is `interrupted`, no second box exists, the tree is kept, and `start` continues. The daemon killed while a host landing gate runs: the gate's group is gone after restart and the landing re-queues. No command is answered before reconciliation has committed. |
| 6 | Inputs are validated at the boundary | A malformed tool payload, a TOML with an unknown key, an unknown workflow name, a sync that removes a workflow an open ticket names: refused by name, nothing written. |
| 7 | A ticket lands end to end and leaves only rows | New ticket, worker commit, candidate gate, review pass, approval, green landing: canonical moves and the ticket is done. The proof snapshot on the host holds what the worker wrote before landing. Afterwards every decision has one audit event naming its target and text, the execution rows carry tokens, cost, model and start reason, every event is one the Store section names, the attempt directory with its proof snapshot, boxes and proof copies are gone, the approval row and each check's execution carry the proof digest, and another live attempt's directory and canonical are untouched. A `status --watch` returns an approval already open when it starts at once, and one started with nothing open returns when the next item is raised. |
| 8 | Review is a publication, and bounded | A seat that exits 0 without publishing is a review error; a seat killed after publishing has published; a second publication is refused; findings below `blocking` pass; a panel of `none` reaches approval marked unreviewed; a seat that publishes a block and is then held by the fixture: no repair starts until its box is gone; a seat that always blocks gets exactly `max_rounds` rounds, then `stopped:limit`. A candidate that commits a `.pi` extension which publishes a pass: nothing loads it, and the seat's own publication is the one recorded. Control: the seat's prompt carries a rule from the project's `AGENTS.md`. A candidate that commits a Claude plugin, settings hook and a `.mcp.json` server which publish a pass: nothing loads them, and the seat's own publication is the one recorded. Control: the seat's prompt carries the committed `CLAUDE.md` rule. A candidate that commits a `.codex/config.toml` naming an MCP server that publishes a pass: nothing loads it, the seat's own publication is the one recorded, and the rogue server leaves no marker. Control: the seat's context carries the committed `AGENTS.md` rule. A seat that publishes with no registration proof does not count, and a fresh seat runs. A gate error's `start` reruns that gate on the same head. A seat or gate that ends after its attempt was abandoned ends `abandoned`, records no check and raises no item. |
| 9 | Each implementer execution starts from the right place | A nudge mid-execution lets the execution end on its own and reaches the next prompt; `stop` delivers it sooner. A worker that leaves an untracked file gets no review and the next prompt lists the file; left again, `stopped:dirty`. With `max_session_executions = 2`: the second execution resumes the first, the third resumes nothing and its prompt is the brief, the fourth resumes the third, the fifth resumes nothing. |
| 10 | The queue lands one at a time and re-judges what does not merge | Three approved candidates, the second red on its merged ref: the first lands, the second gets one repair and a second red raises `red`, the third lands on the moved target with its own gate run. A candidate that does not merge gets a repair naming the paths, and its next head takes gates, review and approval again. A conflict against a target that changed `.yard/config.toml`, in a clone made before it: the worker fetches the target from its bundle, merges, and the new candidate's base is the target, so it passes the `.yard` refusal and lands with the operator's configuration intact. |
| 11 | Only the operator's sync changes `.yard` and canonical from outside | A worker commit under `.yard` comes back with the reason and no gate runs; the same change through `yard sync` is in force for the next execution. A checkout and canonical that each hold a commit the other lacks: both directions refuse naming both heads; a fast-forward passes. |
| 12 | Boxes hold nothing secret | The key is absent from the box's environment and clone; the fixture behind the injecting route receives it. The claude token is absent from the box's environment, files and clone; the fixture behind the login route receives it as `Authorization: Bearer`. The codex host login is absent from the box's environment, files and clone; the fixture behind the login route receives it as `Authorization: Bearer` with `ChatGPT-Account-ID`. |
| 13 | Every gate and seat runs where its row says | A seat that writes to `/workspace` fails and the head is unchanged. A gate box and a seat read the candidate's proof snapshot at `/yard/proof` and their writes there fail; control: the implementer wrote the live proof. An `AGENTS.override.md` the implementer leaves in its clone, excluded through `.git/info/exclude`: the seat's prompt lacks its rule and carries the committed guidance's. A `CLAUDE.md` override the implementer leaves, hidden from git's status, does not reach the seat; the committed `CLAUDE.md` does. A gate box that calls the model or MCP route gets nothing, and an ignored file the implementer left is absent from its checkout; control: the worker box reaches the route. A host candidate gate runs on the head before any review and sees `YARD_BASE`, `YARD_PROOF`, the snapshot it judges, and only the variables its `env` names; a host landing gate runs on the merged ref, reads the approved candidate's proof snapshot and never for a candidate whose approval was superseded, even by an edit while its landing merges. |
| 14 | Worker git is untrusted, and Yard stays local | A worker that plants `core.fsmonitor`, a hook, a clean filter and a remote in its clone, corrupts an object and links its harness's models file to a host path: none of them acts on the host, canonical's objects are intact, the candidate is refused by name. A worker that leaves a symlink in `/yard/proof`, or passes the proof entry bound, is refused by name and no gate runs; a regular file is accepted. Across G7's path every request the model's fixture receives came through the model route, and the daemon listens on its unix socket and the MCP listener only. |
| 15 | Plans and proposals resolve | A ticket on `plan`: its clone refuses writes, it proposes children and a body edit, and the attempt ends with nothing raised; accepting them blocks the parent, which the scheduler does not start once they are done; `yard ticket done` then closes it and its dependents become ready. Closing a ticket with a live attempt is refused. One execution proposing A and B, B depending on A: accepting both mints A first and B's edge names it. |

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
  store/    mod.rs (open, migrate, audit) schema.rs tickets.rs attempts.rs executions.rs checks.rs
  jobs/     mod.rs (load, step, advance) admit.rs supervise.rs review.rs queue.rs reconcile.rs cleanup.rs proof.rs
  git.rs box.rs harness.rs pi.rs pi-mcp-extension.ts claude.rs codex.rs mcp.rs
  daemon.rs api.rs cli.rs config.rs main.rs
crates/e2e/  src/lib.rs (harness, helpers) model.rs (fake model fixture), tests/guarantees/main.rs g*.rs
```

## Open questions

- `replay` is out: whole-attempt disposal keeps no diff to seed from. The
  branch stays in canonical; an operator who wants it cherry-picks.
- Batching several candidates into one landing, with bisection, is out
  until queue wait is measured.
- Whether Yard later links pinfold as a library instead of a process.
- Codex returns as a harness before the cutover, once pinfold's login route
  carries its subscription login.
- The Pi MCP client extension stays TypeScript inside the Rust binary as an
  embedded file; whether pinfold should carry it instead.
