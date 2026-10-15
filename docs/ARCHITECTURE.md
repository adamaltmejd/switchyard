# Switchyard architecture

Switchyard (`yard`) turns tickets into landed code with coding agents. It
keeps a local backlog, runs selected agents in isolated disposable boxes,
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
dependencies and an optional local target branch. `Y-<n>`. Its body is its plan.
_Avoid_: issue, task, story

**Attempt**: one try at a ticket: private writer clones of canonical,
published candidates, and the executions that produced and judged them. A ticket has at
most one live attempt and any number of ended ones.
_Avoid_: worktree

**Lane**: one of the places attempts run through. `max_lanes`
counts them; see Capacity for when an attempt holds one.

**Candidate**: an exact `base..head` and immutable proof snapshot, published
by one execution in an attempt. Independent-writer publications name their producer,
writer workspace, recorded agent settings and selected profile definition.
Every check, approval and landing binds a candidate, never an attempt.

**Execution**: one unit of external work on the one persisted lifecycle:
an implementer call, one reviewer seat, one gate command, one landing, one
cleanup. It has one identity, one external handle once it has one, and one
terminal outcome. Box-backed executions own a box; host-backed ones
(landing, cleanup, a host gate) have bounded host effects and are recovered
from the effect they leave. A repair, an edit, a retry and a
resumed session are each a new execution; statistics are per execution.
_Avoid_: generation, call, task, run, step

**Box**: a pinfold box (its term). Yard brings one up per box-backed
execution and holds it for exactly that long.

**Landing**: an execution that merges one approved candidate onto
the attempt's frozen canonical target branch, verifies the merged ref with
the selected landing gates, and moves that branch to it; its gate runs are its child executions. The queue is the
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
| operator_mcp | trusted stdio tool schemas and forwarding to the unix socket | cli |
| daemon | one process per machine: the unix-socket API, the MCP listener, the scheduler tick | `yard` |
| cli | argument parsing, project discovery, RPC dispatch, rendering | the operator |

The store knows nothing of processes. `git` and `box` return data and never
touch the store. The CLI never opens the store. It uses read-only git metadata
and `project.list` to resolve a linked worktree, then sends the command to
the daemon; it never mutates a repository itself.

Git's bounded output collection future is heap-pinned. The daemon uses the
runtime's normal worker stack settings.

## Threat model

An agent, or a prompt injection in one, lands code nobody approved, spends
credentials, or damages the host.

| Control | Effect |
|---|---|
| One merge path | Only the daemon's queue moves canonical, after authorization and selected landing gates passing or explicit operator acceptance of their exact eligible failures, with actual merged-ref evidence and compare-and-swap (G1, G3). |
| Trusted verification selection | The operator may omit candidate gates, seats and landing gates before admission, including all of them, or select operator handling for retained failures. Exact eligible failures and named admitted candidate or prepared landing omissions may be explicitly accepted with nonempty text; omitted nodes stay skipped, empty panels unreviewed and failures failed. Ordinary omission records no explicit acceptance. Auto policy still requires operator approval for protected paths. Workers cannot select, accept, approve or land (G1, G8). |
| Exact identity | Every check, acceptance, approval and landing binds a candidate, ticket revision, admitted plan revision and selected check, node, execution and result identities. Landing decision attention carries the selected choice identity, including omitted-gate and failed-gate decisions. Merged failure acceptance or repair also binds the original landing, exact old target and pinned merged head. Explicit skipped acceptance binds named candidate or landing omissions and the complete admitted skipped selection; landing omissions also bind the original preparation, old target and pinned merged head, without an execution or result; an omitted required proof is stale, and changed identity carries nothing. Answers and landing re-read those identities (G2). |
| Worker capability | A worker's MCP bearer is issued per execution, scoped to its kind, revoked when the execution ends. It binds to the first MCP session opened with it, so a process that inherits it cannot act. It grants no operator tool; trusted operator stdio uses the host unix socket, never a worker bearer. |
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
therefore builds only from the admitted canonical image commit: `.yard/Dockerfile`,
which only the operator's `yard sync` changes, in a build context the daemon
materialises from that commit. The rest of that context is landed code,
which approval has already passed; the Dockerfile can run it at build time.
Shared image preparation records its owning execution before touching the
context. Cancellation or failed preparation retains that association across
restart. Context reuse and image retirement wait for the execution's checked
reservation release, including its inherited host-child lock. Successful
preparation removes the association after the child completes.
Waiting executions report `image_preparation` with the owning execution ID.

## Projects

`yard init` writes `.yard/config.toml`, `.yard/Dockerfile`, a
`.dockerignore` block and the operator skills, and registers the project with
the machine's daemon. `.yard/` is versioned and the operator's alone: it
changes only through `yard sync`, and a candidate whose diff touches it is
refused at the candidate boundary and returned to the implementer with the
reason. A worker that needs a package or a gate proposes the change.
`.yard/local/` is ignored through `.yard/.gitignore` and holds the store and live state; losing it
loses the backlog. The scaffold's `protected_paths` names every path Yard stages into a
worker's harness, `AGENTS.md`, `CLAUDE.md`, `.agents/`, `.claude/`, `.codex/`
and `.pi/`, so a
candidate changing them always needs the operator's approval, under
`approve = "auto"` too.

Configuration is one TOML document. Unknown keys are errors. Every
cross-reference resolves at load; an unknown name is a load error.

| Key | Meaning |
|---|---|
| `max_lanes` | live attempts at once on this project |
| `max_executions` | positive shared project limit for external execution effects; absent means `max_lanes + 1` |
| `approve` | `manual` (default) or `auto` |
| `[target]` | `ref` (project default branch), `protected_paths` |
| `[isolation]` | `egress` (hosts implementer and gate boxes may reach beyond their routes, typically package registries) |
| `[agents.<name>]` | `harness` (`pi`, `claude`, `codex`), `provider` or `login`, `model`, `effort`; a key the harness has no control for is a load error |
| `[workflows.<name>]` | `implementer`, `review` (seat names in panel order, or `none`), `instructions`, `access` (`write` or `read-only`), `max_session_executions`, `inactivity_timeout_minutes`, `total_work_timeout_minutes`; unset keys inherit from `default`, except optional `profile`, which explicitly selects a packaged audit and never inherits |
| `[gates.<name>]` | `command`, `timeout_minutes`, `stage` (`landing` default, or `candidate`), `runs_in` (`box` default, or `host`), `env` (host variables a host gate receives, by name), optional `closed_inputs` (nonempty, non-secret operator identity for closed inputs and relevant runtime configuration) |
| `[review]` | `max_rounds`, `timeout_minutes`, `blocking` (P0–P3) |
| `[review.seats.<name>]` | `agent`, `instructions` |
| `[profiles.<name>]` | typed node defaults: `runner`, `access`, `outputs`, `outcomes` |
| `[presets.<name>]` | typed graph: declared external `inputs` and named `nodes` |

Typed plans are inspectable through trusted operator MCP `yard_plan_resolve`
and `yard_plan_validate` (daemon methods `plan.resolve` and `plan.validate`).
Both take `project`, optional `target` (otherwise the current default),
one of `preset`, `workflow` or structured `graph`, and optional `overrides`
keyed by existing node name. With no selection they translate workflow
`default`. They read the selected canonical target configuration and return
its branch and exact `config_head`, the resolved graph and configuration
bindings. They create no tickets, attempts, executions or audit rows.

A node's `runner` is `{kind = "agent", agent = "NAME", instructions = "TEXT",
skills = ["PATH"], references = ["PATH"]}`
or `{kind = "command", gate = "NAME"}`. Names refer to existing agents and
gates; plans introduce no command text or tool grants. `access` is
`read-only` or `write-allowed`. `outputs` maps names to `candidate`, `report`,
`verdict`, `selection` or `finding-selection`. `outcomes` declares route names. A preset node can
select a `profile` and replace fields. Overrides replace only supplied
fields; a profile override replaces runner, access, outputs and outcomes,
preserving the node's topology. Maps and lists replace whole fields.

In attempts, ordinary agents may select skills and references explicitly; both lists default
to empty. Skills are Markdown paths. All selected paths are distinct plain
relative paths to canonical regular Git files: at most 512 per node, 32 KiB
per skill, 1 MiB per reference and 8 MiB total per node. Admission freezes
path, blob and executable identities per node from its staging commit.
Every attempt agent launch stages those blobs under the existing read-only
`/yard/input/trusted` mount and names the selected paths in its prompt.
This mount is separate from writable candidate content. Later canonical or
candidate edits cannot change admitted bytes. Unselected guidance is not
exposed there; references are never followed automatically. Selected inputs
enter the consumer definition. Explicit revision rebinds changed selections
from the recorded staging commit, or an explicitly adopted staging commit.

The optional packaged runner is `{kind = "packaged", package = "durable-audit",
agent = "NAME", instructions = "TEXT", skill = "PATH", references = ["PATH"],
files = [{root = "source", path = "PATH"}]}`. Its agent must be configured
with Pi. `skill` names operator-trusted canonical Markdown; `references`
names its explicitly selected regular files. Neither is candidate launch
data. `files` declares bounded source or proof reads. No reference is followed
automatically. A profile, preset or operator graph may select this runner.
A read-only workflow with no seats may explicitly set `profile = "NAME"`
to select a packaged profile on ordinary CLI or scheduler admission. This
field is optional and is not inherited from the default workflow.
Its standalone supported plan is one read-only `implementer` node matching a
configured read-only workflow's agent, with `result: report` and `complete`.
It has no graph inputs, topology or operator-failure selection. Its runtime
is `packaged-audit`. The existing exclusive implementer, cleanup and repair
path also accepts a selected packaged `cleanup` with write-allowed access,
report output and `complete`. Implementer and repair remain ordinary agents;
repair alone produces the final candidate. Other packaged shapes and delegation
remain refused. Cleanup package and selected-file identities bind its node;
the ordinary roles retain their own selected inputs.

The selected controller and its box occupy one existing execution slot.
Stop cancels work; the reservation remains until the controller, box and
inherited host children are proved gone. Native publication and terminal
success alone do not make a report eligible. Failed ownership proof keeps
the report ineligible and blocks continuation; checked release admits the
same bound report without replay.
Inspection exposes the selected canonical input identities; admission checks
them again against its frozen configuration commit.
Refused selected regular-file or size checks leave that ticket without an
attempt, execution or reservation. Explicit start returns `invalid_input`,
the path and reason `selected_regular_file` or `selected_input_bound`.
Scheduler admission continues with other tickets, queue and cleanup.

Node `inputs` map names to `{node, output, kind}` references. Omitted `node`
refers to a declared graph input. Each output must exist with the same type.
`requires` names the distinct inputs whose results a join requires.
`routes` maps declared outcomes to lists of `{to, repeat}` edges. Destinations
must exist. `repeat`, when present, is a positive integer limit. Removing
bounded repeat edges must leave an acyclic graph. No expression language is
accepted. Profiles declare no topology. All preset references validate at
configuration load; one-off graphs and overrides validate at resolution.
Invalid resolution returns `invalid` with the exact `input` and a reason
token: `unresolved_reference`, `output_type_mismatch`, `undeclared_outcome`,
`unbounded_repeat`, `unresolved_or_duplicate_result`, `required`, `empty_name`,
`empty_or_duplicate_outcome`, `invalid_input` or `exclusive_selection`.

Structural validity and runnable support are separate. Resolution can inspect
a valid unsupported graph. Runnable validation refuses it as `refused`, naming
`bounded-repeat`, `fan-out`, `required-result-join` or `custom-graph` when the
graph falls outside the supported shapes. An
ordinary linear graph is `ordinary-linear` and runnable: implementer,
selected candidate-stage gates, panel seats, then selected
landing gates on the merged landing ref. With no selection the full chain
remains the default. A candidate-stage gate has two
instances, `candidate-gate-NAME` and `landing-gate-NAME`; a landing-stage gate
has only `landing-gate-NAME`. Candidate gates and seats reference
`implementer.result`. Each retained seat after the first also requires the preceding
retained seat's verdict as `panel`. Landing gates reference the declared external input
`merged`, of type `candidate`, supplied by the native merge queue. It is a
different check input from the implementer's candidate. No merge node or
execution is introduced by this declaration. Ordinary gate nodes are `write-allowed`
on their private checkouts; seat nodes are `read-only`. A read-only workflow
has only its implementer. Ordinary candidate gates and seats schedule from
this admitted graph and eligible execution results. `complete` advances;
a candidate gate's `failed` route returns to the implementer once per
eligible failing result (`repeat = 1`). Seats carry blocking check identities
through the panel. The final seat's `blocked` route returns to the implementer,
bounded by `max_rounds` plus operator-granted extra rounds. A passing final
seat cannot discard an earlier block. Repairs are new executions; prior
results never reverse. Optional external `feedback`, of type `verdict`,
binds repair findings or gate evidence to the new implementer's inputs.
Pending edits take precedence over routed repairs. A deliberate operator
restart retains its reason and captures the current ticket body and revision.
Existing execution limits, sync, operator exits and identity invalidation
still apply. Landing remains queue-controlled. It drives selected landing
command routes on the exact merged input, after approval, and advances only
from eligible ended results after checked teardown. Infrastructure errors
produce no judgment and take no failed route. The operator may omit any
candidate gates, seats and landing gates, including all of them. Admission
records and exposes the omitted node identities as `skipped` and an empty
panel as `unreviewed`; neither records a passing result. An empty landing
selection still records the actual merged ref and uses authorized queue
landing and compare-and-swap. Selection alone accepts no failed result.
Retained candidate gate, seat and landing gate nodes may set `operator_failure = true`
before admission. The default is false: automatic bounded repair. This choice
is part of the immutable resolved plan, never a worker grant.
A selected failed gate, blocking seat or final accumulated blocking
result raises `candidate` attention only after eligible ended evidence and
checked teardown. `accept` records the operator's nonempty text on that exact
evidence and continues through every remaining selected candidate check.
`repair` takes the existing bounded route. A result accepted earlier in the
panel stays blocked in later seats' immutable results, but cannot trigger
repair again. Infrastructure errors and unconsumed publications cannot be
accepted. At approval the trusted operator may explicitly accept named
candidate-stage omitted nodes with nonempty text. The approval records their
exact candidate, ticket, plan and skipped-selection identities without an
execution, result or passing check. All retained candidate checks finish first;
an empty panel stays unreviewed. An omitted required proof returns `stale`
with identity `proof`, expected empty string and current digest; malformed
proof types remain `invalid`. Ordinary omission records no such decision.
A selected eligible failed landing gate raises `candidate:landing` without
retiring approval. Later approvals on that target wait for its answer.
Its failed check and result stay unchanged. `accept`
continues every remaining selected landing requirement on the pinned input;
`repair` takes the ordinary returned-landing path. Both answers additionally
name `landing`, `child`, `node`, candidate `base`, ticket `revision`, exact
target `old` and `merged` head. Answers and consumption revalidate canonical,
the prepared ref, current admission and eligible evidence. Unknown ownership,
open intents, infrastructure errors and unchecked teardown cannot be accepted.
At approval `observe_landing_skipped` names admitted landing omissions to
observe after preparation. This is a request, never acceptance. The resolved
approval attention stores the request and its approval identity. The queue
pins the merge, then raises `candidate:landing` on the original preparation
before any retained landing child or mutation intent, even for an empty
landing selection. Its exits are accept, edit and abandon. Acceptance names
the requested nodes with `accept_skipped`, original `landing`, candidate
`base`, ticket `revision`, target `old` and `merged`, plus the ordinary
candidate bindings and nonempty text. It binds the complete admitted landing
skipped selection. No child, check, result or passing judgment exists for
an omission. The existing attention resolution is the sole decision truth.
Answers prove checked release of the question's parent and predecessors.
An earlier preparation release does not satisfy a later decision's parent release.
Continuation proves every predecessor owner released before consuming the
decision, revalidates its exact identity and runs every retained gate.
Restart preserves decisions; fresh preparation requires a fresh answer.
Ordinary reduced and empty selections require no request or acceptance.
The complete route must visit every retained node once in these phases.
Candidate and landing command order may differ from configuration order.
Node names, access and outputs match one configured workflow's ordinary
definitions. Runners match those definitions; reviewer runners may also
match declared read-only agent profiles
with verdict outputs and ordinary complete/blocked outcomes.
Profile outcomes describe capabilities. The ordinary seat position supplies
its runtime outcomes: non-final seats keep `complete`, and the final seat
keeps the bounded `blocked` route. Profile selection preserves panel order,
typed inputs, repair bounds and review grants.
Retained seats keep panel order; their predecessor
inputs are rewired to the retained panel and its final seat retains the
bounded blocking route and accumulated verdict. Other inputs and repair
bounds stay ordinary. Other runner/output shapes, cross-phase routes and
other repeat edges remain refused by the shape tokens above.
Inspection freezes no active plan.

A selected `read-only-graph` retains the ordinary implementer and named
command/reviewer definitions. Its candidate nodes are read-only: boxed
commands and review seats, with verdict outputs. Host command gates keep
their ordinary sequential contract. Candidate `complete` routes may fan out
to independent readers. Every selected node is reachable in the complete
DAG. Each reader retains its required `implementer.result` input. Additional
required verdict inputs name ancestor nodes and their exact `result` output.
Multiple incoming complete edges require named inputs for every predecessor.
All predecessors must complete before a join starts; every required supplier
must have an eligible ended result on the same candidate, ticket and frozen
plan revision. The execution ID identifies its immutable result. The join's
persisted inputs name those exact IDs, outputs and supplier plan revisions.
Publication alone supplies no eligible result. A revision invalidates graph
checks from older revisions, preserving their evidence without consuming it.
Candidate graph commands run rather than borrowing earlier gate judgments;
native landing commands retain their declared reuse policy.
Checked restart preserves completed branches and retries incomplete readers
only after their old owners are proved gone. Each branch is recorded before
effects and reserves the existing shared execution cap independently.

Every read-only graph reader declares `complete` and its ordinary `failed` or `blocked`
outcome, with the ordinary bounded repair route to the implementer.
Infrastructure interruption without output takes no successful or failed
route; an unpublished interrupted reader is retried after checked teardown.
Repairs and ticket edits wait for all prior readers to release ownership.
Review results carry the distinct blocking checks from all verdict inputs.
Exact selected failure acceptance still preserves failed evidence.
All retained candidate checks finish before approval. Landing remains one
linear command suffix reached through one candidate boundary and driven by
the native queue on its merged input.

A single-candidate read-only graph may append a terminal agent `selector`
after an exact review join with at least two declared reviewer inputs. It
matches a configured read-only profile with `result: finding-selection` and
only `complete`. Its required inputs include `implementer.result` as
`candidate`, the join's verdict, and selected ancestor reviewer verdicts,
each exactly once. The candidate, reviewers and join retain the controls
above. Each bound review input exposes its immutable result, raw public
findings, check, frozen node, settings and inputs.

The existing Selector grant and `yard_publish_selection` accept exactly
`{groups: [{reason: "TEXT", sources: [SOURCE, ...]}, ...]}` for this node.
There are at most 200 groups; an empty selection is allowed. Each group has
a nonblank reason of at most 32768 bytes and at least one exact public
finding source object from a declared eligible review input. A source may
occur only once across all groups. Combining duplicate findings retains every
chosen contributor. The picker supplies no source author, finding prose,
index or latest-check binding. Missing, foreign, stale or repeated sources
and malformed groups refuse atomically as `selection_input` with identity,
expected and eligible source evidence. These are infrastructure/identity
refusals, never review verdicts. MCP refusals retain their identity data in
the reply and audit evidence.

Publication uses the existing execution report/result and checked teardown.
Successful registered completion and ownership release make the typed
finding selection inspectable, with groups, frozen inputs and picker identity.
Reuse requires those exact eligible source execution/results, candidate
head/proof, ticket and plan/input revisions. Current templates or newest
review prose do not qualify it. Restart settles a committed selection once;
a changed candidate or revision requires fresh inputs and selection. Raw
source findings, verdicts, results and usage remain unchanged and inspectable.
An interrupted or superseded picker follows the existing checked review
recovery; infrastructure failure retains the report without an eligible
selection. This output changes no candidate choice, check, repair, acceptance,
approval or landing authority. Candidate selection below is unchanged.

A selected `writer-graph` declares independent writable agent roots,
including `implementer`, on the same attempt, frozen ticket and target.
There are at least two roots; the graph declares the count and the shared
`max_executions` bounds acting effects. Each root matches a configured
write-allowed candidate profile, has no inputs, publishes `result: candidate`
and declares only `complete`. Complete routes may name read-only reviewers
`review-N` from the selected writable workflow's seat positions. Each
reviewer requires exactly one named root's candidate output, publishes a
verdict and has complete/blocked outcomes. Without a selector it has no
outgoing routes, external graph inputs or repeat edges. Writer graph node names
contain only ASCII letters, digits, underscores and hyphens.
Names identify workspaces, not roles. Admission and consumers distinguish
candidate producers, reviewers, selectors and native gates by admitted runner,
access and output. Candidate-only roots may be named `selector`, contain
`-gate-` or start with `review-`; those names supply no consumer authority.

The writer node name is its declared workspace identity. Each workspace owns
one reusable full clone, index, harness state, input and live proof directory.
Filesystem and branch components encode the node's bytes as lowercase hex,
so case-folding filesystems do not alias distinct declared names.
An execution records its workspace and frozen settings before effects.
A running execution or retained reservation excludes a second writer in
that workspace, including a predecessor whose workspace is historically
unknown. Independent workspaces remain separate. Stop and restart preserve
partial trees. A successor reuses its declared workspace only after checked
predecessor release. Sessions resume only within that workspace with identical
recorded agent settings, consumed definitions and a compatible harness
version, within the existing session execution bound. Historical unknown
settings start a fresh session.
Ordinary repair and settings changes keep their retained clone behavior;
incompatible recorded settings start a fresh session.
Independent writer workspaces cannot transfer between graph nodes.

A selected ordinary linear graph may declare the fixed source-writing path
`implementer -> cleanup -> repair`. All three agent nodes explicitly name
`workspace: implementer` and match configured profiles. They share the
ordinary attempt's clone. Ordinary roles share input, state and live proof;
packaged cleanup gets fresh execution input/state and read-only admitted proof.
The implementer has no
inputs and publishes `result: candidate`. Cleanup requires that candidate and
publishes only `result: report`. Repair requires cleanup's report and accepts
external verdict `feedback`; it publishes `result: candidate`. All three
declare only `complete`. Each complete route hands off after the predecessor's
eligible result and checked release. The existing workspace claim fences
automatic starts and explicit retries before any workspace mount.
`yard_attempt_start` with `workspace: implementer` retries the current role;
an owner refuses as `workspace_owned` with its workspace and execution.
Settings and consumed definitions choose session reuse independently of clone
retention. Different selected settings start fresh sessions in the same clone.
Ticket or plan revision changes restart this prefix at the implementer.

The suffix retains the selected ordinary native gates, reviewers and landing
policy. Its candidate inputs and bounded repair routes name `repair`.
Only the repair publication reaches native judgment and landing. Rejection,
gate repair and landing return continue repair in the same workspace.
Each candidate publication has its own execution, immutable result and Git
ref. Native consumers bind that exact producer. New publications get new
judgments; this path does not reuse checks across publications.
Source access grants neither candidate publication nor operator landing.
Cleanup must publish its declared report; clean source changes alone supply
no candidate or successful result. Interrupted or failed cleanup supplies no
eligible result and takes no successful handoff.

Every writer publication has its own execution identity and canonical Git
ref. Its immutable result records exact base, head, bounded proof digest and
producer execution/node/settings/profile. A retry does not overwrite earlier
publications. The attempt has no implicit latest candidate in this shape.
Continuation records that workspace's prior candidate base, or its prior
execution base before publication. A recreated clone starts at its recorded
head or base. Another workspace's selection never advances that base.
A reviewer binds its named producer's eligible execution and immutable result,
then uses a detached clone from canonical and the published proof snapshot.
Reader admission revalidates that exact eligible producer and result inside
the execution-start transaction. Its current declared candidate input must
name that producer and result, with base, head and proof matching the checkout
columns. A superseded source starts no reader.
It never reads a writable writer tree. Edits and definition handoffs drain
prior owners before successors reread the frozen revision in their retained
workspaces. Completed reader results remain history; a new candidate gets a
new reader. Infrastructure interruptions take no verdict route.

Trusted MCP `yard_attempt_stop` accepts an optional exact `execution`.
Multiple running writers without that identity refuse as `ambiguous_execution`.
Trusted MCP `yard_attempt_start` accepts `workspace` for explicit continuation
of a declared writer on a live writer attempt. Undeclared or omitted ambiguous
reuse refuses as `undeclared_workspace`; an owned workspace refuses as
`workspace_owned`, naming its owner execution. An exact stopped/red attention
may also identify its writer continuation. Continuation preserves the frozen
graph and requires a lane and the shared execution capacity. A refused
continuation commits no lane or clock change. Workspace and attention starts
validate, reserve the lane and create the writer in one transaction under the
admission lock. Continuation does not clear another workspace's attention.
Each accepted writer start records its
own execution immediately, including while another writer's stopped attention
remains open. It never replaces an attempt-wide pending continuation.
A ticket edit discharges superseded stopped/red items after prior executions
and reservations drain. A discharged timeout renews the existing attempt clock.
Ownership items resolve only through checked release. Every declared writer
then consumes the edited ticket in its retained workspace.
Without a selector, once all declared writers and readers
finish, `stopped:candidates` releases the lane and names the published candidates.
Inspection retains all candidate identities and execution settings.
This boundary supplies no approval or landing.

A writer graph may include one integration writer besides its independent
roots. It matches a write-allowed candidate agent profile and requires at
least two distinct named root candidates. Each source has a complete edge
to it. `implementer` remains a root. The integration writer starts only when
all named publications are eligible on the current ticket and plan revision.
Its execution binds those immutable producer results before effects.
The input name `feedback` is reserved for internal repair on candidate-producing
writers. A declared source with that name is invalid at
`nodes.<writer>.inputs.feedback`, with reason `invalid_input`, before admission
creates an attempt or execution. Ordinary external repair feedback is unchanged.
Changing a source invalidates consumers of the old integration result.
An explicit integration start before its inputs are ready refuses as
`integration_inputs`, naming its workspace, without taking a lane or owner.

The integration writer starts at the frozen target in its own reusable clone.
It receives fresh detached source snapshots, proof snapshots and validated
Git bundles at `/yard/candidates/<hex-input>/source`, `/proof` and
`/bundle/candidate.bundle`, read-only. Each bundle is exposed through its own
directory mount, separate from the source and proof snapshots.
Bundles export the exact canonical publication
ref `refs/yard/candidates/<attempt>/<producer-execution>` and its ancestry.
Canonical and other writers' live clones are never mounted. The agent fetches
and combines the inputs in its own clone. A conflict left dirty follows the
existing bounded dirty return and attention; it supplies no candidate or
selection. Stop and daemon restart retain partial work for a new execution
after checked release. There is no separate integration lifecycle.

Integration publishes a new candidate with its own base, head, proof,
producer settings and profile. Input checks, judgments and approvals do not
carry. With a selector, consumed source candidates are excluded from its
declared inputs; only terminal candidates and bound reviews reach selection.
Selected downstream checks and approval bind the integrated output through
the existing selection path. Without a selector the graph stops at candidates.
`docs/examples/integration.toml` supplies an operator configuration fragment
with integration profiles and a preset selecting the combined candidate and
a candidate diff check. Further verification is an explicit plan selection.

A writer graph may declare one read-only agent node named `selector`, matching
a configured profile with `result: selection` and only `complete`.
Every unconsumed writer and named reviewer has a complete edge to it. Its required inputs
name every unconsumed candidate and bound review exactly once. The same frozen reviewer
profile may review every independent candidate. Review inputs carry the raw
findings, check, reviewer definition, settings and frozen declared inputs, bound to that candidate's
immutable producer result. The selector reads detached candidate clones and
proof snapshots at `/yard/candidates/<hex-input>/source` and `/proof`, read-only.
It receives only context, progress and `yard_publish_selection`.
That tool accepts exactly `{candidate: "INPUT"}` once. An undeclared or
non-candidate input refuses as `selection_input`; changed frozen inputs refuse
by identity. Successful registered completion and checked ownership release
publish a selection result naming the existing input reference and picker
execution/settings/source. Selection never creates an implementation or verdict.
Automatic picker activation and startup revalidate the observed choice,
candidate, proof, plan, ticket revision, absence of pending continuation and
current declared inputs in one transaction. They require the same latest
selector execution and recheck its ownership and result eligibility before
activation or startup. Superseded scheduling leaves a newer operator choice
and its approvals unchanged and starts no picker.
Publication followed by infrastructure failure retains the raw report but no
selection result. It raises ordinary `red:review` attention; only its supported
`start` exit retries the picker. Stopped or superseded runs raise no failure.

The selector's complete route may lead through a linear native command suffix.
Candidate gates require its `selection` result and consume the chosen candidate;
landing gates consume the ordinary external `merged` input. Only the chosen
candidate's own declared reviews count toward approval. Blocking reviews retain
the ordinary repair bound and exact failure acceptance after candidate gates.
Candidate-gate admission revalidates the observed choice, candidate and proof
and current declared input eligibility inside the execution-start transaction.
Its base, head and proof must match the immutable selected candidate input;
superseded admission creates no execution.
Trusted operator failure policy may select those reviewers and native candidate
or landing gates. Chosen eligible failures raise the ordinary exact candidate
accept/repair item; landing failures bind the prepared input and original child.
Producers and the picker cannot select that policy or gain decision authority.
Initial approval and live-policy reapproval mark the chosen candidate unreviewed
when it has no declared reviewer, even if a losing candidate has one.
Approval at the review limit selects only admitted agent verdict reviewers
bound to the chosen producer; a candidate's review-shaped name grants no role.
Admission and plan revisions record a verification node as skipped when its
name lacks the configured runner role and verdict output. A writer shadowing
that name leaves the omission visible and eligible for exact operator decisions.
Repairs use the selected producer's workspace. Automatic candidate-gate repair
revalidates the observed choice, candidate, proof and exact eligible failed check
and decision inside the writer-start transaction. A superseded trigger starts
no writer and changes no choice or approval.
Landing returns stage the target
bundle in that producer's input mount under the existing landing reservation.
The landing-bound approval and immutable choice identify that producer. A
superseded return withdraws without delivering a bundle or installing repair.
The queue validates the current approval, choice, candidate, proof, ticket and
plan before staging under the landing, then again before delivering the bundle
and recording repair together. A continuation before mutation intent supersedes
the old landing; an open intent still refuses continuation.
While a continuation is pending, replacing its choice with a different exact
candidate refuses as `pending_continuation`, naming both candidate references.
Selecting the same candidate preserves those instructions and its target bundle.
Explicit workspace or writer-attention start refuses `pending_continuation`
while a continuation is pending, naming its attempt, choice and instructions.
It cannot consume, relabel or reassign automatic dispatch, even to the same writer.
Ticket edits retire superseded pending instructions before scheduling fresh
producers. Steering validates the live choice, never a prior choice snapshot.
Pending dispatch revalidates its observed ticket, plan, choice, candidate, proof
and instructions inside the lane-reservation and writer-start transaction.
It also requires the observed unread-edit status. A deliberate restart consumes
already observed edits or plan handoffs; a later change supersedes dispatch.
An obsolete dispatch has no lane, clock, writer, choice or approval effect.
Dispatch resolves the producer from that validated choice inside its transaction.
The new producer execution
freezes exact failure feedback or pending rejection/landing-return instructions
in its inputs before invalidating the choice and clearing pending continuation.
Failure source references stay unchanged; this feedback grants no judgment or
approval authority. Approval and queue verification
bind the exact choice execution beside candidate, proof, ticket, plan and target.
CLI approval and rejection name `--choice` when a choice exists. Selection grants no write,
approval or landing authority; the existing queue and CAS remain authoritative.

Trusted MCP `yard_attempt_select` (daemon `attempt.select`) selects directly
with exact `ticket`, `attempt`, `revision`, `plan_revision`, `target`, `choice`
(zero when absent), complete current `inputs` and declared candidate input name.
It refuses stale or foreign references before any store write, and refuses
acting owners and unresolved landing intents. A valid choice records an ordinary
ended execution with operator provenance and no box, grant or owner.
Every producer start, including attention continuation, transactionally refuses
an unresolved landing intent before creating a writer or retiring its choice.
Starting an allowed replacement producer retires the current choice and approval. Its
new immutable publication requires fresh affected reviews, selection and checks.
Superseded candidate, choice, result and raw review evidence remains inspectable
until ordinary attempt cleanup removes files; their rows remain history.

Trusted operator MCP `yard_attempt_start` (daemon `attempt.start`) accepts
`preset`, `workflow`, structured `graph` and targeted `overrides` for a new
attempt on a ready ticket. Its target comes from the ticket or project default.
The same resolver validates the selection before any attempt, execution or
lane reservation is written. A valid unsupported graph is refused with
`input: plan`, its shape, `structurally_valid: true` and `runnable: false`.
The ordinary linear subset, its selected packaged cleanup, selected read-only
and writer graphs, and single-node packaged audit above run using the existing lifecycle.
With no selection,
admission resolves the ticket's workflow; overrides alone patch that workflow.
Selection on a live attempt or a stopped/red retry is refused with
`input: plan`, `reason: active_plan`. Retry consumes the existing frozen plan.
Admission records selection names, overrides or the one-off graph alongside
the resolved plan and revision. Attempt inspection returns this persisted
evidence. Provider and login bindings contain names, never secret values.
Recorded selected plans are re-resolved against their immutable admitted
source and definitions on reading. They are not rewritten as ordinary chains.
Historical ordinary translation stays in memory and leaves stored evidence
unchanged.

Trusted operator MCP `yard_attempt_revise` (daemon `attempt.revise`) adjusts
live ordinary work, selected read-only or writer graphs, or a running single-node packaged audit with expected
`plan_revision` CAS. It accepts `preset`,
`workflow`, `graph` and node `overrides` against the current recorded definitions.
During admitted implementation or at a quiescent ordinary candidate
it also accepts exact canonical
`definition_commit`, `staging_commit` and `image_commit` sources independently.
Omitted sources stay frozen. Image adoption records the Dockerfile blob and
whole context tree and builds that exact committed context. Definition adoption
re-resolves the recorded selection unless a replacement selection is supplied.
Selected workflow, gate, seat, review and agent evidence and staged blob/mode
identities are rebuilt from those commits. Overrides alone patch the current plan.
Selections retain the admitted workflow and runtime shape. Packaged audits
retain their package identity, selected paths and read/publication grants;
declared profile instructions and agent model/effort may change within the
recorded package launch capability. Explicit source
adoption also accepts a stopped packaged implementer ended as `unchanged`
with recorded packaged revision, retained lock-contract reservation and its
open stopped and exact ownership items. Adoption preserves both items and
does not launch work. Reviewer nodes may select a declared
read-only agent profile with verdict outputs and ordinary complete/blocked
outcomes. Implementers, including selected read-only implementers, may select
admitted profile instructions and selected skill/reference paths with the
same agent, access, outputs and outcomes. Worker grants and target stay frozen.
Template sync alone keeps active
workflow, gate, reviewer, tool, skill and image inputs unchanged. Explicit source
selection adopts committed changes. Implementation agent
model and effort may be adopted explicitly. Its harness, connection and egress
grants stay frozen. Candidate gate definitions may change;
future landing requirements may change only before any landing intent.
This includes candidate-stage gate definitions selected at landing and
resolved intents. Changes outside selected landing requirements keep the
existing adjustment rules.
Landing gates execute and bind their recorded revision's selected definitions. Source
adoption refuses running gate or review work. Open landing intents,
pending repairs not yet executing and questions other than approval,
candidate, ownership or that stopped packaged item refuse adjustment.

Original admission stays unchanged. Each adjustment appends a
`plan_revision` row and `plan.revised` event. Inspection exposes original
`admission`, current `plan`, `plan_revision` and ordered `plan_revisions`.
Unrecorded history stays unknown. Adjustment never edits tickets or
historical effects. The writer rechecks CAS and landing intent in the same
transaction. Old approvals are withdrawn; approval and candidate questions
are superseded. Ownership questions and reservations remain until existing
checked teardown proves every owner gone. No replacement overlaps them.
Stale edits and decisions name expected/current `plan_revision` and write
no row. Approve and reject on revised candidates name `plan_revision`.
Selected decision evidence reports the execution's recorded consumed definition.
Retained candidate checks require that definition to equal the current selection;
landing decisions require the original approval to remain active on that revision.
Raw execution and check evidence remains unchanged.

Startup and result recording use each execution's frozen revision and
settings, including when adjustment races startup or recovery. Each
implementation derives its launch settings from its revision's selected Agent
or Packaged runner and recorded agent evidence. Original attempt implementer
and admission rows stay unchanged; unrecorded history keeps its original settings.
The packaged runner keeps its package harness.
A matching execution may finish and qualify normally. Affected judging nodes stop;
their old judgment cannot count. Persisted publications and original rows
remain intact. A running implementer may finish. The revision records its
`handoff_from` execution; unread handoff or changed implementation policy
takes precedence over pending repair and starts an operator edit execution
after checked release. Its labelled operator diff includes the plan and changed
consumed definitions, settings and input identities, with no secret values.
Existing stop and start can deliver the handoff sooner. Superseded read-only publications
are refused as `superseded_report`, produce no eligible report output and
cause no `missing_report` failure. Returning paths never rewrite ended history.

Canonical's symbolic HEAD names the project default. A supported linear
attempt freezes its resolved inputs at admission. Every execution uses its
recorded definition commit for configuration, staging commit for trusted
inputs, and image commit for the Dockerfile and entire build context.
A feature config may declare the project default in `target.ref`; execution
does not require that declaration to name the attempt's branch. Import and
explicit default selection validate it under the sync rules. Config caches
bind branch and head. Default selection preserves branch cache entries and
in-flight publications. Image construction binds the admitted whole build-context commit.
Observed runtime image IDs belong to executions; a rebuilt reference supplies
no evidence about an earlier runtime image. Project capacity reads the current default's `max_lanes` and
`max_executions`, never the selected target's limits. Execution allocation
reads the validated current default entry in the one configuration map;
it launches no Git. Startup, scheduler refresh, sync and explicit attempt start
publish that entry under canonical's lock. Explicit start uses the published
reading for admission before spawning or releasing the lock to answer a live
stopped/red item. Mutations of the current default ref or symbolic HEAD invalidate it before
acting, then publish the validated current input or leave it unavailable.
Classification and invalidation hold the publication lock. A nondefault ref
mutation preserves that input. Ordinary target ref effects release the
publication lock; default selection stays serialized. A landing's target
lock stays held through ownership release.
Sync refusals before canonical mutation preserve the published input;
checkout-only consume does not invalidate it. A completed failed import
revalidates the actual default; an uncertain mutation leaves it unavailable.
Automatic refresh never restores an unavailable input. Known completed
mutations publish while their target is held; unknown landing ownership
requires the existing reconciliation proof before publication. Startup does
not publish over an open default intent or its retained reservation.
An intent's start exit can prove recovery while default input is unavailable.
Start answers the named attention item; without a name it prefers an open
intent's recovery item over other stopped or red items on that ticket.
An unavailable validated default excludes new effects. Already-admitted
executions waiting for capacity reserve nothing. A landing whose policy
check encounters unavailable publication after reservation waits on the
same project events, retaining its existing reservation and host lock.
If its attempt ends during the policy read or publication wait, the landing
withdraws without further effects or new attention for the ended attempt.
Validated publication wakes waiters to reread the current default. Waiters
remain eligible for their existing stop or abandon exits. Unknown default mutation
ownership keeps that input unavailable until proved resolved. Selected-target cache
reads do not replace the published default input.
An attempt freezes its supported ordinary or packaged workflow inputs under
`attempt.admission` in status and attempt inspection (G6). Revision 1
contains the resolved supported plan, its outcome routes, selected
workflow, gate, seat, review and agent definitions, provider and login names,
and harness version pins. It names the configuration target and commit,
separate exact definition, trusted staging and image commits, configuration
and Dockerfile blobs, build-context Git tree, and regular trusted staging
source paths, blobs and executable bits. Admission selects the same commit
for all three sets. Source validation checks each set against its recorded
commit. An image commit binds the entire context; Dockerfile equality does
not imply context equality. Quiescent ordinary candidate revisions may
explicitly adopt definition and staging commits under plan CAS.
For each packaged node those staging identities contain only selected Markdown
and declared references (Markdown at most 32768 bytes, each reference at most
1 MiB, the set at most 8 MiB). Admission also records the package identity.
Configuration reads inventory only the configuration, Dockerfile and ordinary
agent-environment paths. Packaged resolution reads only selected literal
paths at the canonical commit; it does not inventory unrelated source trees.
The Yard runner's version and executable SHA-256 identify its tool implementation.
These are input identities. Executions separately record observed runtime
image and result identities. Repairs and restarts leave admission evidence
unchanged. Historical rows without recorded evidence report `status: unknown`;
current templates and recovered bases do not supply historical evidence.
Implementation, gates, seats and publication consume these admitted
definitions. Image builds use the recorded image commit; trusted tool and
skill staging uses the recorded staging commit, including after repair
changes the candidate base. Earlier recorded inputs with one source commit
continue to consume that commit for all three sets without rewriting evidence.
Each execution reads its recorded plan revision. Template-only sync
affects future admissions; active plans and matching checks stay unchanged
through sync and restart. No execution row adopts a new plan revision.
Legacy unknown admissions retain current-template behavior and their supported
exits. Missing candidate gates and reviews can start without admitted input
bindings, including after repair or retry. Old execution and approval plan
bindings stay unknown; migration never fabricates historical evidence.
Ordinary candidate checks can retain an older
recorded plan revision only when candidate, ticket, node policy, consumed
definitions and typed producer inputs still match. Node policy includes runner,
access, declared inputs, outputs, outcomes, required inputs, failure handling
and repair routes.
The complete route selects successors and does not change completed evidence.
Digests, registration, eligible results and checked ownership remain required.
Scheduler, panel producer selection, decisions and approval/queue rechecks use
the same current-check reading. A new consumer binds the actual older producer
execution and result. Changed policy or a changed typed supplier invalidates
affected evidence; unrelated evidence stays current. Returning routes never
reverse execution history. Historical unbound checks require exact revision
equality and gain no inferred producer or result.
A sync that removes a workflow an open ticket
names is refused, naming the tickets. Two digests bind verification: the
**gate digest** over the Dockerfile and every gate with its `stage`, `runs_in` and `env` names, and the
**review digest** over the workflow's panel, each seat's agent settings and
instructions, and `review.blocking`. Complete gate and panel digests bind
exact decisions and approvals. Each check has a separate consumer digest.
It binds its runner, its gate definition or agent settings and harness pin,
the Yard executable identity, and, for boxed consumers, the recorded Dockerfile
blob and whole build-context tree. Agent consumers also bind the regular
committed paths, blobs and executable bits staged for their harness, plus
root guidance and their node's explicitly selected skill/reference inputs.
Reviewers bind review policy; packaged consumers bind their selected staged
sources and package identity. Source commit labels alone do not qualify a result.
Changed consumed inputs invalidate affected consumers. An unchanged seat
survives an unrelated gate change, and an unchanged gate survives another
gate's change, only while its own consumed inputs stay identical.
Unrecorded consumed-source proof stays unknown and cannot justify reuse
across revisions. Its historical check digest remains unchanged.
Check recording, qualification and exact decision evidence use these same
recorded inputs. Decision evidence names the original consumer definitions
and typed suppliers without relabeling the execution or result.
A target template change does not change an active attempt's digests
(G2).

**Verification identity.** What a judgment binds is written as data:

Gate, reviewer and read-only audit executions append `verification` once at
the launch boundary. Bound results copy that evidence. It records the checkout
Git tree and base tree, candidate commit provenance and proof identity, typed
suppliers, frozen node and consumed definitions, command bytes or agent argv
digest, and effective staged byte hashes. Shadowed admission sources remain
provenance; byte evidence uses the staging selection. Packaged audits also
record the frozen audit launch digest. Executions record pinfold's observed image ID and the
running daemon executable hash separately from admitted source identities.
Runtime policy records memory, allowed egress and environment and route names;
environment values and route credentials are never recorded or fingerprinted.
Candidate and landing boxed gates may declare `closed_inputs`. This asserts that their
command consumes only the recorded code tree/base, proof, typed suppliers,
consumed definitions, image tools and declared environment sources. Its token
names the operator's relevant runtime configuration. The operator maintains
Pinfold's documented stable launch contract and changes that token when
relevant configuration changes. Yard trusts that contract for image tools,
environment, isolation and runtime behavior. This is an assumption, not
dynamic dependency tracing or complete kernel or host attestation.
Raw kernel policy fingerprints and complete filesystem or mount witnesses
are not prerequisites. The operator must declare relevant runtime changes.

Only a declared boxed gate with no egress or selected host variables,
known typed suppliers, an observed image ID and supported Pinfold/runtime
identity records `reuse = eligible` and a semantic `identity`. The supported
identity is Pinfold's version plus doctor's documented Podman or Apple container
backend and runtime version. Yard keeps only those doctor fields. Environment
identity references the Yard executable's boxed-gate definition, the recorded
base source and observed image defaults; no values or value hashes are kept.
Semantic identity binds trees, the exact base commit source, proof, typed
suppliers, command, consumed definition, observed image, environment sources,
memory and runtime identity.
Consumer labels and box names/IDs are provenance, not semantic inputs.
Changed tracked identities or relevant configuration require fresh evidence.

Untracked external state, model/login/credential-dependent inputs, missing
supported identity, host gates, reviewers and audits remain
unknown and must rerun for reference reuse. No historical identity is inferred.
Worker claims and command output cannot authorize eligibility. Executions still
append launch evidence once and results/checks retain their original rows.
At a gate launch, Yard compares eligible declared identities within
the attempt. A candidate supplier must have an eligible immutable publication
of the launched head, exact base and proof. A landing supplier must name its
parent's exact prepared old target, pinned merged head and proof, including
through continuation provenance. Equal actual code trees and proof
may replace commit and supplier execution labels for this comparison only.
A match references the original passing execution, result and check in the
new execution's frozen verification record. The gate command does not run.
An unchanged declared gate crossing a plan revision also records a reference.
Matching binds the original and current frozen node policy. Consumer names,
stage and proved publication routing do not change consumed inputs. The
`complete` successor and ordinary candidate failure-to-repair edge do not
qualify a passing result. All remaining policy, including operator failure
policy, must match. Missing recorded policy cannot authorize reuse.
After checked teardown that reference selects the original check; the new
execution ends `reused` with no judgment, output or check of its own.
Unknown consumers run normally. Failed checks never supply references;
acceptance stays bound to its exact original evidence. Skipped nodes have no
execution or reference. Decisions and approvals still bind the exact current
base, head, proof, ticket revision and plan revision. Landing may reference a
passing candidate or landing check only when actual prepared inputs match.
The exact consumed base source, including `YARD_BASE`, still binds; equal trees
alone are insufficient. On continuation and before intent, Yard re-reads
references and actual prepared Git trees. Preparation, pinned refs, ownership,
current authorization, exact decisions and target CAS remain authoritative.
Candidate current-check lookup excludes child executions. Reference selection
does not adopt templates or transfer acceptance across identities (G2, G3).

```
candidate     = attempt, frozen target branch, base, head, proof
gate input    = candidate, ticket revision, plan revision, node policy, typed inputs, consumer digest
review input  = candidate, ticket revision, plan revision, node policy, typed inputs, consumer digest
check         = its kind's input, execution identity, verdict, observed image id
acceptance    = candidate, ticket revision, plan revision, both digests, node,
                execution, exact check and result, operator, nonempty text
merged acceptance = acceptance, original landing, approval, exact old target,
                    pinned merged head and child inputs
merged skipped decision = candidate, ticket revision, plan revision, complete
                          admitted landing skipped selection, named omissions,
                          original landing, approval, exact old target,
                          pinned merged head, attention, operator, nonempty text
skipped decision = candidate, ticket revision, plan revision, admitted candidate
                   skipped selection, approval attention, named omitted nodes, operator, nonempty text
approval      = candidate, ticket revision, plan revision, both digests, selected check identities, actor,
                accepted evidence, explicit skipped decision, whether it overrode verification
```

A ticket edit or a new head makes every input new. A changed proof with an unchanged head is a new
candidate. A check whose input no longer matches is
superseded and counts for nothing; one whose input still matches keeps
counting, and its row is never rewritten. An approval is superseded when
any of its parts is.
Acceptance's authoritative persisted reading is resolved `candidate`
attention. Its payload names the evidence and operator; its resolution and
audit carry the decision and text. A delayed answer returns `stale` with
`identity`, `expected` and `current`, without writing any row. Identity tokens
are `attention`, `target`, `base`, `head`, `proof`, `revision`, `plan_revision`,
`gate_digest`, `review_digest`, `selected_result`, `attempt`, `landing`, `child`, `node`,
`old`, `merged`, `approval` and `ownership`. Acceptance never changes
a check verdict or an execution result, including after landing or restart.

`approve` and `protected_paths` remain live selected-target policy in no
plan or digest. Automatic approval reads them under target serialization.
A landing re-reads
its approval against current rows and policy before every landing gate and
again when it records its intent: an automatic approval that the current
policy would not give (the candidate now touches a protected path, or
`approve` is now `manual`) is withdrawn, the candidate leaves the queue, and
`approval` is raised. No check reruns.
Revoked containment narrows admitted egress at box launch. Capacity and
credentials remain live and take precedence over an admitted plan.
`.yard/Dockerfile` bytes are admitted inputs; the image id a box reports is the
environment used, recorded on every execution, which each check reads through
its execution. A check row holds only its verdict and reads its input from
its execution.

`operator.env` (`$XDG_CONFIG_HOME/yard/operator.env`, `~/.config` when
unset, mode 0600) holds the
machine's credentials, one per connection or login, and the machine's own settings:
`YARD_BOX_MEMORY` gives each box a share of memory, and `YARD_ORIGIN_<NAME>`
overrides a connection's origin. Neither is a project key.

**Connections.** An agent's `provider` names a connection: an upstream
origin and the header its key rides in. Yard knows three, each with its key
variable: `openrouter` (`https://openrouter.ai/api/v1`,
`OPENROUTER_API_KEY`) and `opencode-go` (`https://opencode.ai/zen/go/v1`,
`OPENCODE_API_KEY`), both as `Authorization: Bearer`, and `anthropic`
(`https://api.anthropic.com`, `ANTHROPIC_API_KEY`) as `x-api-key`.
Claude accepts `provider = "anthropic"` or `login = true`, exclusively.
Selecting both is `provider_login_conflict`, naming the agent, at load.
The direct API reviewer selection is `model = "claude-opus-5-5"`,
`effort = "medium"`. Pi accepts only `openrouter` and `opencode-go`.
`operator.env` may set
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

`ticket new --target BRANCH` sets an override. Omission resolves the current
project default at admission. `ticket edit --target BRANCH` changes it;
`--default-target` clears it. Target edits require the expected revision,
an open ticket, no live attempt and no landing intent. Branch names are
validated local refs. Admission refuses `missing_target`, naming the branch,
without creating an attempt; named sync supplies the ref.

A ticket is `open`, `done` or `abandoned`, and may be `parked`. It becomes
`done` when its candidate lands on the frozen target, even if that branch
never reaches main, or when the operator closes it with `yard
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
body. Acceptance classifies the proposing attempt through its frozen
target's current config. A new child's omitted target uses the project
default at admission. An ended pre-target attempt with unknown target uses
the current project default to classify its pending proposals; its stored
target stays unknown. Accepting a planner's child adds the
`parent depends_on child` edge; accepting
the edit makes the body the plan. Once the children are done the operator
either closes the ticket with `yard ticket done` or moves it to an
implementing workflow and starts it. An execution on a read-only workflow
ends its attempt after a valid report and successful checked teardown. Its
proposals remain operator decisions. Missing report raises `stopped:failed`
with `missing_report`; no report result advances. The scheduler starts a ticket on a read-only
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
   `max_lanes`. Admission creates the attempt, freezes its effective target branch, workflow and
   implementer, and records the first execution before anything external
   runs, in one transaction (G4). `yard attempt start` is the same
   admission for one named ticket, now: refused, naming the reason, when
   the ticket is not ready or no lane is free. On a ticket whose live
   attempt has a `stopped` or `red` item it is that item's `start`.
4. **Work.** An execution runs the implementer in a box with the clone
   mounted as the workflow's `access` says and the MCP route. A ticket edit steers a live attempt and never interrupts a running implementer:
   when that execution ends the implementer runs again, before anything
   judges its candidate; `stop` then the `stopped` item's `start` delivers it sooner. An edit
   while gates or review run stops them now (boxes down, checks superseded)
   and a stopped review is not a round; while the candidate awaits approval
   or is queued it withdraws the approval in the edit's own transaction. An
   edit resolves every open `stopped` and `red` item of the attempt; no
   pending retry survives it. A pending edit is derived, never stored: the ticket
   is newer than the revision the latest implementer execution read, which its
   start transaction records. Every transaction that decides the next step
   reads it with the current revision and records nothing for a pre-edit
   candidate; a repair or item raised meanwhile keeps it, and the next
   implementer carries every pending change. The timeout wins over a pending
   edit: `stopped:timeout` is raised, and the edit on that item renews the clock.
   A fresh session gets the whole body; a resumed one gets the
   body diff since the revision it last read, labelled as the operator's
   edit; seats always get the whole body. `stop` ends the execution now and keeps the tree.
   A stop while awaiting execution capacity ends the same row as `unchanged`
   and raises `stopped:unchanged`; freeing capacity does not launch it.
   Stop and reservation decide against the current row in one transaction.
   The same stop handle covers reservation, preparation and the worker.
   During preparation an acting host child may finish before the execution
   ends as `unchanged` with `stopped:unchanged`. No subsequent worker turn
   starts; capacity remains held until external ownership is proved gone.
   A reserved implementer without a current stop handle refuses `stop`,
   naming the execution and any ownership attention. It records no successful
   stop and changes no execution, attempt or reservation.
   The attempt's inactivity and total-work clocks end it the same way. When
   the worker stops, Yard asks git inside its box whether the clone is
   clean before the box comes down: a clean committed head is a candidate;
   a dirty clone goes back to the implementer once with the listing, and
   raises `stopped:dirty` if still dirty; an unchanged head raises
   `stopped`, except that an edit-started execution ending clean on the attempt's
   current head sends that candidate to judgment at the current revision. A writable implementer's next execution resumes its workspace's session where the
   harness offers one, after checked predecessor release and only with identical
   recorded settings, consumed definitions and a compatible harness version. Context is the harness's business: Yard stages each
   harness's own automatic-compaction threshold with the launch and never
   compacts a session itself. After `max_session_executions` executions on
   one session the next starts a fresh session from a brief of ticket,
   the candidate's diff stat and the attempt's progress notes, and the count
   starts again. Spend is bounded by the attempt's total-work clock alone:
   it counts wall time while the attempt holds a lane, across every
   execution and every wait, nothing automatic resets it, and only an edit
   on a `timeout` item renews it. Expiry ends the running execution as `timeout` and starts
   nothing automatic. An approved candidate holds no lane, so expiry never
   touches an approval, the queue or a landing.
5. **Gate.** Candidate-stage gates run on the head in a private writable
   checkout of the exact commit, in a gate box or on the host as each gate's
   `runs_in` says, in declared order, before any review. A failure is
   believed and buys a repair execution, or its admitted operator handling
   raises `candidate` attention; a gate error raises `red`. Every
   gate runs again at landing.
6. **Review.** One review round per candidate: one execution per seat,
   each in its own box reading a fresh read-only checkout of the head. A seat publishes once through `yard_publish_review`;
   an execution without a publication is a review error. Findings carry
   priority, file, line, category and body. Each raw finding exposed by
   `attempt show` or a selector review input carries the same daemon-authored
   `source`: the durable finding row ID, check ID, review execution ID,
   attempt ID, ticket name, frozen base, head, proof, ticket revision, plan
   revision and execution input `digest`. IDs are local to the project store.
   Missing historical provenance stays null. Identity never derives from
   prose, list position or the current check. Review publication accepts only
   finding content; workers cannot supply source identity. Recovery preserves
   these references and the original findings. The verdict is derived: no finding at or
   above `blocking` is a pass. Blocking findings return to the implementer as
   a repair execution unless admitted operator handling asks for acceptance.
   `max_rounds` counts review rounds; at the limit the
   attempt raises `stopped:limit`, which the operator may approve over (see Attention). A panel of `none` is unreviewed by policy and
   as clean as a pass. A publication counts from the moment it commits, but
   the next seat, the verdict and whatever follows it wait until the
   publishing execution has ended and its box is gone.
7. **Approve.** The operator sees ticket, diff, checks, review and
   protected paths. Approval binds the exact candidate and its
   checks (G2). Under `approve = "auto"` a clean, unprotected candidate is
   enqueued in the transaction that verified it, with the setting recorded as
   the actor. Reject records notes and dispatches a repair execution.
   Abandon ends the attempt.
8. **Land.** For each frozen target, the queue lands the oldest approved candidate whose approval
   still holds (see Projects), qualifying selected landing gates on the merged
   ref (see Git, G3). A landing binds its exact
   target and merged head; nothing verified on an older target authorises a
   landing on a newer one. Once pinned, the prepared input survives unchanged
   pre-intent retries and recovery, with its eligible completed passing children
   and requalified immutable passing references.
   An eligible selected failure with operator handling waits for an exact
   accept or repair answer, keeping approval active. Explicit repair retires
   approval and returns the exact failed evidence to the next implementer
   execution as bound feedback. It consumes no accepted landing input.
   Other red landings
   return to the candidate: the attempt
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
`review`, `gate`, `landing`, `cleanup`, `command`, `report`. There is no other lifecycle: no
owners, leases, receipts or per-feature recovery. One lifecycle is not one
recovery predicate: a box-backed execution is proved by asking pinfold for
its box, a landing by canonical's refs once its host children are over, a
cleanup by the directory's absence, a host execution by its lock (below).

Trusted operator MCP `execution.start` creates a standalone boxed command
or read-only agent report. It requires the registered project, target branch,
full input commit, one resolved node and `timeout_seconds` (1 to 86400).
The node may select a committed profile or explicitly name a configured
boxed gate command or ordinary agent. It has no inputs, required suppliers,
routes or operator failure handling; its sole output is `result` (verdict
for commands, report for agents), and its sole declared outcome is `complete`.
Commands permit read-only or write-allowed source access, no host environment
imports and no network. Reports require read-only source access.
Standalone agent reports require empty `skills` and `references` lists.
Nonempty selection refuses before execution creation or launch as
`standalone_selected_inputs`, naming `node.runner.skills` or
`node.runner.references` and the selected paths, even if those files do not
exist. Selected input staging is supported only for attempt agents.
Packaged runners, candidate outputs and workflow authority are refused as
`standalone_authority`; host commands or imported environment names are
`standalone_box`. This operation creates no ticket or attempt.

The execution's immutable `scope` records project, target, input commit/tree,
resolved node, command or agent settings, definition/staging/image commits
and blobs, harness and Yard runner identity, egress policy and original
deadline. Source comes from a fresh private clone of the exact committed
input. Trusted agent environment and image come from the frozen target
definitions. An optional existing attempt must have the same target
(`scope_mismatch` otherwise). It is association only: neither ticket edits
nor attempt completion change this execution's scope. Its plan revision,
candidate proof and approval are absent. Standalone executions, including
attached ones, cannot provide checks, attempt approval, candidate suppliers
or automatic landing authority.

These executions use the same running/ended rows, shared capacity,
inherited host lock, checked teardown, usage and restart reconciliation.
The admitted lifetime includes waiting and startup and is never reset.
Stop and deadline cancellation retain the inherited execution lock through
box teardown, including any `pinfold down` child, before checked release.
`execution.show` returns scope, results and usage from that row.
`execution.stop` cancels its controller and checks ownership before releasing
capacity; an unrecovered reserved owner refuses as `ownership` with execution
identity. Restart checks the original owner, then ends the row interrupted;
it launches no replacement. A new run requires explicit operator creation.
Successful registered reports publish once with their scope. End records a
typed result separately; interruption preserves raw publication without
promoting it to a completed report. Command nonzero exits record failed
verdicts and their actual exit code. Results and final measured usage remain
on the execution; missing usage remains unknown. Private checkout, transcript
and harness files remain in the project-local effect directory. Historical
scope remains unknown. Migration preserves old IDs, associations, proofs,
checks, approvals, results, usage and ownership records.

Direct operator editing and testing outside Yard needs no execution record.
Re-entry of committed branch history is named `sync`; dirty checkout import
and ticket-free automated source landing are not supported.

**Host children are accounted for.** Before an execution spawns a host
child (Git, pinfold or a host gate), it opens an execution lock file under the
attempt's or the landing's directory, or the project-local effect directory
for cleanup and standalone executions, and takes `flock` on it, and every
such child inherits that descriptor. The daemon's copy dies with the
daemon; a child's lives as long as the child. On restart the daemon takes
each reserved host execution's lock without waiting before reading any
effect: taken proves no old child can still act. Held after the recorded
host gate group is killed, or held by a git child, which is never killed
and ends on its own, leaves the execution unresolved: a landing intent
stays, that target's queue is refused and `red` is raised. The recorded handle is for
killing; the lock is the proof, so a child spawned but not yet recorded is
not missed. A host gate's command starts only once its handle is recorded,
so a restart kills every group that ran a command. A host gate still alive
after a restart is killed by its verified group and the execution is
`interrupted` like a boxed one; a landing re-queues.

A pre-intent landing continuation is a new execution. Its `resumed` chain
names the original landing and the parents of retained child evidence. The
original row alone records `prepared`: target ref, exact old target and merged
head, pinned by `refs/yard/landings/<original execution>`. Preparation opens no
canonical mutation intent. Migration leaves old preparation unknown.
Answers and continuation use one predecessor ownership proof. Every predecessor
landing and child must have ended with no reservation, no unresolved mutation
intent, no live box and a free inherited host lock under the current ownership
contract. Failed box inventory leaves ownership unknown and blocks. Completed
passing or explicitly accepted failed children also
require eligible results, checked teardown and exact candidate, proof, ticket,
admitted plan and approval identity. Their rows and parents stay unchanged.
Reused children require checked teardown and the same exact identity. Their
immutable references must still qualify against the actual prepared input.
Infrastructure-error and interrupted children rerun; neither becomes passing
evidence. The final landing's inputs name the selected child execution IDs.
Changed target or invalid prepared ref or selected evidence retires the
continuation. Fresh preparation and verification follow. No prepared identity
is overwritten. Live policy reads, shared capacity and target serialization
also apply when retaining a child. Failed acceptance binds only its original
preparation; fresh preparation inherits no answer. Failed selected children
have ordinary gate check rows for exact operator evidence. Candidate check
readings exclude landing children.

A worker execution records, on its row: agent, harness and its version
(pinfold's pin, read from `pinfold artifacts` once at daemon start, which
fails if a harness has no pin), provider,
model, effort, the execution it resumed and its session id, start
and end, exit cause, tokens in and out, reported cost, and the reason it was
started (`first`, `repair`, `edit`, `retry`, `dirty`, `restart`).
Usage is optional measured data with per-field completeness and provenance.
The nullable `tokens_in`, `tokens_out` and `cost` columns hold the values;
`usage_fields` holds their metadata. Execution JSON, including trusted MCP
`yard_attempt_show`, exposes `usage` with nullable `input`, `output`, `cost`
and `fields` for each, alongside harness, harness version, model and resumed
execution identity. No report means unknown, not zero. Explicit zero is
a numeric observation. Cost is reported USD, never a bill inferred from
token rates. Historical values and nulls survive migration unchanged with
`legacy` provenance and `unknown` completeness; old zeros are not measured
bills. New fields without a report are `unreported` and `unknown`.

Codex input and output counters are totals. Cached input and reasoning output
are subsets and are never added again. Before launch, the adapter reads the
named session's counters and its append position from bounded regular-file
reads that follow no worker symlink. Fresh executions start at zero. New
session counter observations and terminal reports replace the execution's
values with the difference from that launch baseline, never predecessor rows
or a repeated cumulative spend. Missing or unreadable resumed baselines leave
the affected fields null, `unreported` and `unknown`. Observed counters and
failed turns are `harness` and `partial`; reported completed-turn fields are
`harness` and `complete`. The daemon saves new observations before settlement;
interruption retains partial records. Unreported monetary cost stays null,
`unreported` and `unknown`, including on resume. Reported service tier is
retained as optional `usage.fields.service_tier`; no tier or bill is inferred.
Pi counts assistant `message_end` usage only from the new execution's stream,
including on resume.
Input includes reported input, cache reads and cache writes. Repeated
`turn_end` messages and session history are not counted. Each observation is
saved by the daemon before settlement, with `harness` provenance and `partial`
completeness. Interruption or failure retains those observed increments.
Successful `agent_settled` makes a field `complete` only if every assistant
message reported its coverage; missing components leave observed sums partial.
An absent field stays null, `unreported` and `unknown`, including cost.
Explicit zero stays numeric. Claude API and login executions count streamed
assistant usage by message id, replacing repeated content-block observations.
Input includes reported input, cache reads and cache writes. Streamed tokens
are saved as `harness` and `partial`; interruption retains them and leaves
unreported cost null. An explicit-zero assistant output remains a partial
zero observation; provider counters not emitted by the harness are not inferred.
Result token totals replace streamed observations and cover only the current
invocation, including on resume. Missing result token fields retain streamed
partial observations. Claude result cost and `modelUsage` cost are session
totals. Fresh executions may record a reported cost as a local harness estimate,
never a subscription bill or a token-rate calculation by Yard. Missing fresh
cost stays null; explicit fresh zero stays numeric. Resumed executions leave
cost null, `unreported` and `unknown`: no invocation-start cost baseline is
observed; previous rows or an interrupted run are not used to infer one.
No session total or guessed difference is recorded as local cost. Successful
results make reported fields `complete` only with full component coverage;
failed results retain `partial` completeness.
The adapter contract is increments for the new execution only, never a
repeated session total. `partial` means an observed increment with missing
coverage, not a complete total; `unknown` makes no coverage claim. These raw
records have no aggregate, mandatory cap or prominent board display.
Packaged Durable executions read their own boxed usage baseline before
starting pending work. Token observations and terminal usage subtract that
baseline; cost remains unreported. Changed observations publish
`execution.usage` in history. A crash retains partial observed increments.
A gate execution records gate name, verdict, exit code, duration and the
box's OOM count. Every operator decision
records the exact target it acted on and the text it carried. These rows are
the statistics; nothing is derived from the audit stream.

**Intent before effect.** Before a box comes up or an execution's host child starts, the
execution row exists and holds a persisted capacity reservation. The box name is the execution id, and a crash between
intent and handle is reconciled by asking pinfold; a host child is found by
its lock (G5).

**Publication is durable output, not outcome.** An audit report, a review's findings, a
proposal and a progress note are rows the moment the tool call
commits, whatever the execution does afterwards. A review publication is a
decision: one transaction records its findings and the seat's check, with
its audit event. Publication alone supplies no eligible node result.

Read-only implementers and source-only cleanup receive the audit grant.
`yard_publish_report` accepts
exactly `{outcome, report: {summary, findings}}`. Summary and each finding are
nonblank strings of at most 8000 bytes; findings has at most 200 entries.
The serialized arguments are at most 262144 bytes. Outcome must be declared
by the admitted node; supported read-only nodes declare only `complete`.
The daemon records one publication on the running execution, with its node,
plan revision, ticket revision, typed inputs and starting base, head and proof.
The worker supplies none of those identities. Unknown fields or malformed
content return `invalid_report`; an undeclared outcome returns
`undeclared_outcome`. A second publication returns `duplicate_report`.
Ended or superseded executions cannot publish. A pending ticket edit exempts
the superseded audit from `missing_report`; it produces no report result and
the implementer runs again on the new revision. Trusted operators and their
selected read-only auditors inspect publications through `attempt show`.
Reports use the existing execution lifecycle. The trusted Durable controller
is packaged under `runners/durable-audit` and explicitly selected by the
admitted packaged runner. One live Pi/controller box consumes one existing
execution slot.

The operator's image supplies Node 26.11.1 and the exact supplied npm lock
at `/opt/yard/durable-audit`, outside `/workspace`. The three direct
`@earendil-works` packages `pi-durable`, `pi-ai` and `chord` are 1.2.0;
transitive `pi-telemetry` is overridden to 1.2.0. Durable supplies the
conversation and SQLite checkpoints, Pi AI the model client and schemas,
Chord its context. Telemetry is a pinned Pi AI dependency, with no configured
exporter. The image build checks the official Node archive checksum and
locked npm integrities, disables package scripts, then seals installed files.
The reviewed Debian glibc base supplies the C/C++ runtime. The isolated
image build installs `libatomic1` from its authenticated OS repositories
before invoking Node; it supplies Node's required `libatomic.so.1`.
An offline build requires a reviewed base containing that runtime closure.
The fixed launcher verifies those files and the Node binary; the controller
checks the exact runtime, lock and required package exports before any model
request. Durable's `Harness` is an object with callable `open`; the other
required factory exports are functions. Missing or incompatible assets name
`runner_unavailable` and the asset. Installation and installed `.yard` commit
and sync belong to the operator. No package installation, discovery or
download occurs at runtime.

Controller, model client and read tools run inside one box supplied and
owned by Yard. The controller owns no box and executes no host code. It
uses the existing OpenRouter or Opencode injecting route and a guest
placeholder, never an upstream credential. Real Pi initializes the execution's
one MCP session and keeps its id in memory. For the selected runner its native
input hook launches the fixed image-owned controller in the same box before
model dispatch. Selection makes no outer Pi model request. Anonymous pipes
carry one bounded report to Pi's native
`yard_publish_report` client. The child inherits routing variables only,
never the bearer or session. The native bridge writes session and terminal
frames directly to stdout before its input hook returns. Child terminal status
is read separately from report outcome. A bare successful exit is `missing_report`; no eligible report
exists without native publication and checked teardown. Package completion
alone supplies no result eligibility.

Yard writes admitted launch data at `/yard/input/audit.json`, never from a
candidate entrypoint. The packaged controller uses the execution's frozen
model and optional effort. Declared effort sets the inner agent's thinking
level and provider reasoning capability: OpenRouter sends `reasoning.effort`;
opencode-go sends OpenAI-compatible `reasoning_effort`. Unsupported effort
values refuse; omitted effort enables no reasoning capability. Explicit SDK
maps preserve the accepted xhigh and max levels without clamping.
An omitted effort adds no launch field, preserving the older sealed package
protocol. New singleton audit admissions record `package_launch = "effort"` separately from
the sealed package identity; revisions preserve that launch contract.
That contract and cleanup's `source-write-v1` authorize supported effort even when the receiving
daemon's compiled package differs from the frozen package.
Without that record, only an exact match to the daemon's compiled package
establishes effort support. Other preserved packages receive the original
six-field launch even when their historical settings record effort. That
setting remains recorded but does not control the older inner agent.
Adoption uses the same capability reading as startup. Without effort support,
a different non-null effort refuses as `unsupported_effort`, naming `effort`
and the requested expected and recorded current values, before any revision
or question/owner change. An unchanged historical setting or explicit omission
is allowed. Effort-capable packages accept supported effort changes.
Admission records the selected canonical regular Git
blobs and package identity (fixed launcher, Node version, launcher/controller
and lock hashes). The native Pi bridge verifies that identity before launching
package code, including an older sealed package; the launcher verifies its
runtime closure before any model request.
The recorded package identity stays authoritative after admission, including
across daemon upgrades. Readback validates original canonical selected-file
identities independently of the current binary's package bytes. An image
incompatible with the recorded package refuses as `runner_unavailable`, naming
the asset, before model requests; it does not invalidate the frozen admission.
This identity is separate from pinfold's real Pi artifact pin. Selected Markdown and
declared references are staged from those blobs under `/yard/input/trusted`,
read-only and separate from source, proof and writable state. Later canonical
or candidate changes cannot replace them. Broad base guidance, skills and
extensions are excluded from this runner's agent environment.
At most 512 regular files are declared under `/workspace`, `/yard/proof` and
`/yard/input/trusted`, including the selected inputs. Links and traversal
are refused. A traversal component in a declaration names `invalid_input`
and `declared file` before model requests. Undeclared reads name
`undeclared_file`. Read tools open only
those files. Yard supplies frozen read-only audit, proof and trusted mounts;
selected cleanup alone receives its writable source clone. This package
supplies no mount lifecycle. Only the trusted
selected Markdown becomes instructions. Candidate code and extensions are
never loaded. There is no shell, arbitrary host-read or delegation tool. `write_declared`
and `commit_source` refuse as `source_write_refused` without explicit source-write
launch authority. Only selected write-allowed cleanup receives that authority,
under the recorded `package_launch = "source-write-v1"` contract. Its launch
sets `source_write = true`; historical audit launches retain their original schema.
`write_declared` replaces one declared existing regular source file with at most
32768 UTF-8 bytes. Links, multiply linked files, traversal, `.git` and `.yard`
components are refused. `commit_source` stages and commits only declared source
paths with a nonempty message of at most 1024 bytes. Git runs inside the same
box with explicit cwd, scrubbed environment, disabled hooks, filters, fsmonitor and
signing, at most 32768 bytes per output stream and 120 seconds per call. Cleanup must leave
a clean committed tree before its report can advance to repair. The daemon's
final packaged cleanliness inspection also uses a scrubbed environment and
disables repository-configured hooks, filters and fsmonitor. It refuses an
index or HEAD containing Gitlinks before status can enter submodules and
load their independent configuration. The daemon
validates actual committed Git and proof output through the existing handoff;
source-write authority grants no candidate publication or landing authority.

Every read-only audit execution receives a fresh detached source checkout
at its recorded head (or admission base) and a bounded, link-refusing copy
of its available proof, both mounted read-only. Absent proof is an empty
read-only directory. Its writable harness, scratch and checkpoint state
and read-only launch input belong only to that execution and its admitted
node. The execution row retains that identity. No audit resumes a Pi session
or mounts predecessor Pi or scratch state. Host staging uses fresh execution
paths, and cleanup never follows guest links. Execution reservation,
checked teardown and whole-attempt cleanup govern these paths.
New packaged admissions record `package_checkpoint = "exclusive-v1"`.
After an interrupted execution, supported `start` may continue Durable work
in a new execution. The predecessor must be ended, unreserved, have checked
teardown and have no report. Its box and inherited host owner must be proved
gone before checkpoint reservation or mounting. Unknown ownership keeps its
reservation and attention. Checked recovery resolves only that owner's item;
interrupted work still needs an explicit start or an edit.
`prepared.durable` records checkpoint origin, predecessor and frozen node,
typed inputs, candidate/proof, ticket/plan revisions, consumed code/package,
model, selected skill/tool sources and containment/work policy identities.
`resumed` names the predecessor execution. Only matching identities reuse
that checkpoint; changed pending-work inputs start fresh. A competing start
refuses as `checkpoint_owned`, naming checkpoint and execution identities.
The one execution reservation excludes another box mounting the checkpoint.
Only the replacement boxed controller opens the retained
`/yard/state/audit-checkpoints` mount. Fresh Pi, scratch and input state stay
separate. The daemon reads no checkpoint bytes and runs no checkpoint code.
The boxed controller checks its bounded checkpoint identity and storage;
corruption or incompatibility names `checkpoint_invalid`. A supported retry
after that error starts fresh. There is no host writer or live state copy.
Durable reacquires the same submission. Interrupted tools are unsafe and are
not replayed automatically, including `write_declared` and `commit_source`.
Native MCP publication and canonical input identities stay authoritative.
Stop signals the existing controller to abort its work, then performs checked
box teardown. Cancelling a waiter alone does not abort work. Failure to join
abort still requires checked teardown before ownership can be released.
Older admissions without the checkpoint contract keep fresh-only behavior.
SQLite checkpoints remain untrusted boxed data, never workflow evidence.

The controller admits at most 12 model HTTP requests and 64 executed tool
calls. Exceeding a bound names `audit_limit`. Each HTTP response is at most
1 MiB with a 60-second request bound.
Reads return at most 32768 file bytes with a truncation flag; the selected
Markdown must fit. Tool results serialize only that bounded read or a named
publication/refusal. Launch JSON is at most 128 KiB and its prompt at most
32768 bytes. Report bounds remain the publication contract above. Logs emit
only named completion/refusal and numeric usage frames; ambient client environment except
guest proxy routing is removed before package imports.

A live legacy attempt without admission evidence cannot continue as a
read-only workflow. `attempt start` refuses with `unbound_report` and the
attempt identity before creating an execution or resolving its attention.
The scheduler also leaves that attempt stopped, including after an edit.
Classification uses its target's current workflow config, as for other
legacy work. Abandon the attempt, then start a fresh admitted attempt.
Existing G5 ownership requirements still apply; this refusal releases no
retained owner. Historical admission, node, result and teardown evidence
stays unknown. Ended planner proposals keep their existing acceptance path.

Ordinary implementer, gate and seat executions bind the admitted plan revision,
node name and typed exact inputs before any effect. The implementer has no
graph inputs; its row holds the starting base, head, proof and ticket revision.
Candidate gates and seats name the producing implementer execution and its
`result`, with exact base, head and proof. Later seats bind the preceding
eligible panel verdict. Repair feedback names the eligible result that routed
it and records the evidence before the implementer starts. Landing gates name their parent
landing as the supplier of external `merged`, with the target base, merged
head and approved proof. Merge and cleanup executions are not graph nodes.
Continuation and repair remain start reasons, not gates or checks.
If parent ownership release refuses after landing-child admission but before
its controller starts, that exact child ends interrupted with no judgment,
route or checked teardown. Its binding stays intact. The parent retains its
reservation and ownership attention until checked proof; no child is launched
or duplicated by this refusal (G5, G8).

Execution end records a result separately from publication. Its typed outputs
are the admitted candidate, a completed read-only report, or a gate/seat
verdict, or a selector's existing candidate reference. A produced output declares `complete`, except a candidate gate's
failed judgment declares `failed`, and a final seat with any blocking panel
check declares `blocked`. Each seat verdict carries the panel's blocking
check identities alongside its own publication and verdict.
Infrastructure outcome and exit evidence remain separate from the
declared judgment. Missing publication or infrastructure error alone has no
judgment output or outcome route; existing truthful attention applies.
Successful terminal text or exit without an audit publication is an
infrastructure failure, never an inferred report. A held publication stays
durable but ineligible. An audit result requires successful harness
registration and settlement after box teardown; interruption preserves the
publication without promoting it to a result.
A result is eligible only after the execution ends, its
reservation is released and existing G5 ownership proof records checked
teardown. Review also requires the durable harness registration proof.
An interrupted published review keeps its publication and binds its verdict
on the same execution after ownership reconciliation. It needs no new seat.
When that registered review has an eligible interrupted result after checked
release, its end transaction resolves only its own `red:ownership` item as
`reconciled`. Scheduling continues without an operator retry. Unregistered
or unpublished reviews and unrelated attention retain their existing exits.
`attempt show` exposes bindings, results, checked teardown and eligibility.
Historical bindings, results and teardown stay unknown; a new node consuming
a historical candidate records its exact value with an unknown producer.
Node-bound gate and review checks require an eligible result after checked
teardown. Historical checks require ended, unreserved executions. Their node,
result and teardown evidence stays unknown. Both retain exact identity
requirements; reviews also require durable harness registration.
Older complete-only ordinary admissions translate their supported decision
routes in memory. Their recorded admission and prior results stay unchanged;
unknown historical producers remain unknown. The same scheduler handles them.

**Capacity.** `max_lanes` counts attempts from admission until approval,
abandonment or a candidate returning from the queue for repair, which must
reacquire a lane before its worker starts. Ordinary panels run one seat
at a time; selected graph readers and independent writers may share one
attempt's lane concurrently. A completed writer graph releases its lane at
`stopped:candidates` when it has no selector; declared continuation reacquires one.
Selected graphs keep the lane through native candidate verification and approval.
Landings serialize per frozen target;
unrelated targets can advance within the shared project execution cap.
Every implementation, review, gate, landing Git phase, cleanup and standalone execution uses the
same project `max_executions`. A live box/controller counts once; each
concurrently acting bounded host execution counts once. Sequential host
children inside one controller share its slot. Waiting executions own no
box. Landing releases its slot while awaiting child gates; each child
reserves its own slot, so a limit of one completes sequential gates and
the final Git effect without counting the parent twice.
Effects reserve capacity before taking target locks. Final landing keeps its
target lock through intent, publication and the effect's ownership proof.
The queue screens existing rows only. It creates the landing before any
Git-backed approval or policy check. Initial admission rereads current
approval and policy inside preparation's reservation and inherited host lock;
a lapse withdraws the landing and approval truthfully.
The initial landing target-config read shares preparation's reservation
and inherited host lock, including branch validation and Git reads on a
cache hit.
Before each child gate, the landing reserves its own slot for the current
approval and policy read, including protected-path Git. It takes the canonical
lock inside that effect and releases the slot before the child acts.
Returning a failed landing gate reserves the landing row again before
writing the repair target bundle. A conflict return uses the reservation
already held by Git preparation; it does not reserve a nested slot.
Reservations live on execution rows. An ended row retains its slot until
pinfold proves its box absent and its inherited host lock is free. Unknown
ownership retains capacity and raises `red:ownership`; startup and later
scheduler probes release only after that proof. A recovered registered
published review resolves its obsolete ownership item only when its
interrupted result is eligible; no legacy quiescence is inferred.
A supported packaged implementation ended as `unchanged` after stop resolves only
its exact `red:ownership` item as `reconciled` in the checked-release
transaction. Its current inherited lock must exist and be free, and its box
must be absent. Packaged qualification uses that execution's recorded revision,
never the latest admission. Missing revision or admission supplies no proof.
The stopped decision, raw result and result ineligibility remain unchanged,
including after source adoption. Explicit supported start after checked release
consumes the adopted committed definitions.
Ordinary stopped preparations retain their existing ownership operator exits.
A recorded release on that execution takes precedence over an older
concurrent failed probe; that probe raises no new ownership item.
Cleanup's current execution lock survives removal of its attempt directory.
A supported cleanup already ended as `removed` resolves only its exact
`red:ownership` item as `reconciled`, in the checked-release transaction.
This requires current box absence and an existing free host lock. The lock
directory is retired after that transaction. Unknown ownership retains both
reservation and attention. Failed or incomplete cleanup, unsupported lock
contracts, missing current cleanup locks and unrelated questions keep their
existing refusals and operator exits. Cleanup is not replayed by this release.
A supported implementation already ended as `candidate` resolves only its
exact `red:ownership` item as `reconciled` in that same checked-release
transaction, after its existing result becomes eligible. Its current
inherited lock must exist and be free, and its box must be absent. The same
candidate, execution and input bindings continue into ordinary review;
unknown ownership, a running controller, unsupported legacy contracts and
unrelated questions retain their refusals. The implementation is not replayed.
A supported review or gate already ended as `pass` or `fail`, or a registered
review ended as `selected` with a typed candidate or finding selection matching
its publication, with completed infrastructure resolves only its exact
`red:ownership` item in the checked-release transaction, after its existing
result becomes eligible.
Current box absence and an existing free inherited lock are required.
Registered review publication and gate verdict remain the existing result's
sources. The same execution, inputs, publication and typed selection or
truthful judgment route continue without controller replay or operator retry.
Running or ineligible results, infrastructure failures, missing publication,
missing current judgment locks, legacy contracts and unrelated questions
retain their refusals.
Reducing the current limit
blocks new effects while existing usage drains. When an attempt lane frees, the scheduler
gives it to a live attempt waiting to reacquire one before admitting a new
ticket.

**Legacy ownership.** Each new row records that its binary established the
execution lock contract before effects. Migration marks older rows without
that contract and reserves running rows, ended open intents, failed,
refused or abandoned effects, red/conflict landing returns, and gate/review
rows whose teardown may have been unchecked. Absence of a new execution
lock proves nothing for them.
A legacy landing with an open intent may use its existing landing lock.
A legacy host gate with a recorded host handle may use its existing
attempt/landing lock. These locks must exist and be free, and pinfold must
prove the execution's box absent. A terminal boxed gate/review `pass` or
`fail` with a recorded box handle finished its pre-box Git; box absence
proves its remaining controller cannot act. Other reserved legacy rows
retain capacity and `red:ownership`. Start, edit and abandon do not attest
host quiescence or clear these reservations. The operator stops an
ambiguous upgrade and resolves old host ownership outside Yard; there is
no CLI override or guessed release. The final upgrade rehearsal must
account for this ambiguity. A quiescent old daemon with no ambiguous rows
can migrate its completed history and candidate approvals.

**Cancellation** is taking the box down. Yard never signals a process it did
not start on the host, and never signals a process inside a box. Host work
in a blocking thread cannot be aborted; it is bounded by its own timeout and
its child process is what gets killed.

## Attention

One table, five kinds. Each kind declares its payload and its exits; every
surface reads that declaration.

| Kind | Raised when | Exits |
|---|---|---|
| `approval` | a candidate is verified and `approve` is `manual`, or it touches a protected path | approve, reject, abandon |
| `candidate` | an eligible named candidate or merged landing failure selects operator handling, or requested landing omissions have a pinned preparation | accept, repair, edit, abandon; omissions have no repair |
| `proposal` | a worker proposes | accept, reject |
| `stopped` | an execution ended without a candidate advancing: `unchanged`, `dirty`, `failed`, `interrupted`, `timeout`, `limit` | start, edit, abandon; on `timeout` only edit, abandon; on `limit` approve, edit, abandon |
| `red` | a gate error, a review error, a second landing red, a landing that could not run, a landing intent canonical cannot decide, unknown external ownership (`ownership`), an execution that could not record its own end (`error`) | start, edit, abandon; on a landing's item only start |

`start` always means "try again from here". After an implementer stop it
starts the next implementer execution. On a gate or review error it reruns
that check on the same candidate. On a landing that could not run it
re-queues the candidate under its existing approval. On an undecided landing
intent it reads canonical against the restart table again, typically after
the operator has repaired canonical by hand; while canonical still decides
nothing the item stays. On `timeout` an edit renews the total-work clock; on
`limit` it allows one more review round; either way the implementer runs
next. On `limit`, `approve` answers the item while the attempt is at its round
limit. It binds the frozen target branch, head, proof, ticket revision,
admitted plan revision and both digests like an `approval` item. Template
sync keeps the admitted gate and review identities. It names the gate checks and the blocking
review checks it overrides, and records that it overrode a review. The
landing and everything after are unchanged, and an edit withdraws it.

A candidate awaiting its queue turn raises nothing. A landing whose target
moved raises nothing. An item is resolved in the transaction of the command that
answers it.

Trusted `yard_attempt_accept` and `yard_attempt_repair` (daemon
`attempt.accept` and `attempt.repair`, CLI `yard attempt accept|repair`)
name ticket, attention, check, plan revision, target, head, proof when present,
and nonempty text.
For `candidate:landing`, they also name original landing, child, selected
node, candidate base, ticket revision, exact old target and pinned merged head.
The attention identity binds the full candidate and selected result evidence.
Acceptance grants no early landing approval. Final approval and its audit name
the accepted evidence; the queue revalidates the exact selected check and
accepted result identities at each approval reading.
Merged acceptance keeps the existing approval and revalidates its prepared
input and exact failed evidence before every consumption and final landing.
Merged repair uses the same identity checks and decision store. It takes the
ordinary return path with the complete failed evidence in the new execution's
feedback. The original failed check and result stay unchanged. The repaired
candidate needs fresh selected verification and approval before landing.

Trusted `yard_attempt_approve` (daemon `attempt.approve`, CLI `yard attempt
approve`) optionally names `accept_skipped` (`--accept-skipped` per node),
approval attention, base, ticket `revision`, admitted `plan_revision` and nonempty text, alongside
target, head and proof when present. Only candidate omissions from the admitted
selection are eligible. A stale candidate, ticket, plan or named selection
returns `stale` with `identity`, `expected` and `current`, and writes no row.
The authoritative approval and its audit retain the explicit decision through
landing and restart. The queue revalidates its skipped-selection identity.

Approval may also name `observe_landing_skipped` (`--observe-landing-skipped`
per node) to request a prepared omission question. The request accepts
nothing. Its later `attempt.accept` names `accept_skipped` and the merged
bindings; it has no check, child or result identity. Acceptance and nonempty
text use the existing attention resolution and audit. The complete skipped
selection, original landing and pinned input are revalidated at answer and
consumption. Every retained selected gate remains required.

## Store

SQLite, WAL, foreign keys, one writer: the daemon holds an exclusive
`flock` on `store.lock` for its life. Every decision is one short
transaction that also appends its audit event (G7): admissions, execution
starts and ends, checks, approvals, landings, operator commands.
Observational updates (handles, progress, usage) are column writes with no
event. No transaction is held across external work. Nothing replays the
stream.

The events, each naming its ticket, attempt, execution or attention target:
`sync.imported`, `sync.consumed`, `sync.default`; `ticket.new`, `ticket.edited`,
`ticket.parked`, `ticket.unparked`, `ticket.linked`, `ticket.done`,
`ticket.abandoned`; `attempt.admitted`, `attempt.candidate`,
`attempt.stopped`, `attempt.abandoned`, `attempt.ended`; `candidate.selected`; `plan.revised`;
`execution.started`, `execution.ended`, `execution.waiting`, `execution.stopped`,
`execution.reserved`, `execution.released`; `check.recorded`; `approval.given`,
`approval.ended`; `landing.prepared`, `landing.continued`, `landing.intent`, `landing.recorded`; `attention.raised`,
`attention.resolved`; `report.published`; `execution.usage`; `tool.refused`.
`report.published` names its execution and any associated ticket and attempt.
`execution.usage` records observed read-only agent usage on that execution.

Tables: `ticket`, `dependency`, `attempt`, `plan_revision`, `execution`, `check`, `finding`,
`approval`, `attention`, `audit`. Findings, proposals and decision text are
rows and survive cleanup. Forward
migrations only; a store from a newer schema is refused. Reconciliation
backfills an old live attempt's target from the former single-target HEAD
and adds that known target to its open approval and stopped-limit candidate
identities when absent. Other identity fields stay unchanged. Unknown ended
history keeps no target. Added execution workspace and settings columns
remain NULL for historical rows; migration invents no producer or session
compatibility evidence.
Attempt and approval `choice` columns remain NULL for history without a picker.
Migration never infers picker provenance from a candidate head.

Files live outside the clone while an attempt is live, under
`.yard/local/attempts/<id>/`: transcripts, gate logs, harness state, the
worker's live `proof/` directory and its `proof-snapshots/<digest>` copies.
Writer graphs retain each writer's `writers/<workspace-key>/clone`, `state`, `input`
and live `proof`. Their bounded proof snapshots share the attempt's
content-addressed snapshot directory, with separate execution staging.
Selectors and integration writers stage candidate input clones and bundles
under `candidate-inputs/<execution>/`; these stay separate from writable trees
and are removed with the attempt.
Read-only audits use `audits/<execution>/` for checkout, proof copies,
state and input; none is reused by another execution.
Cleanup removes the whole attempt directory, including every writer workspace,
after every own execution releases ownership. Other attempts remain.
A surface that needs a log
after cleanup does not get one; it gets the rows.

## Git

Every git call is a bounded child with an explicit cwd, `GIT_CONFIG_GLOBAL`
and `GIT_CONFIG_SYSTEM` pointed at `/dev/null`, `LC_ALL=C`, a fixed
identity, and `-c` overrides that no repository configuration can undo:
`core.hooksPath=/dev/null`, `core.fsmonitor=false`, `commit.gpgsign=false`,
`tag.gpgsign=false`, `protocol.allow=never`, `protocol.file.allow=always`
(local paths the daemon validated are the only transport),
`protocol.ext.allow=never`. Git and pinfold are found on the daemon's `PATH`. Canonical is a bare
repository under `.yard/local/canonical.git`. An ordinary writer uses a
`--no-hardlinks` clone of canonical on branch `yard/<ticket>/<attempt>`.
Independent graph writers each clone once on
`yard/<ticket>/<attempt>/<workspace-key>` and reuse that clone for sequential executions.
Clones share no objects, hardlinks or indexes with canonical or each other.
Writer graph publications are fetched into
`refs/yard/candidates/<attempt>/<execution>`. The daemon runs no git command with its
cwd in a clone. It takes a candidate's objects by a fetch run
in canonical from the clone's path, which git serves under its rules for
untrusted repositories, takes nothing in the clone's configuration as
authority, and refuses a malformed clone by name. Workers commit;
they cannot push.

**A repair sees its target.** A repair after a red or conflicting landing
names the frozen branch and target commit it failed on. The daemon writes a bundle of that branch
into the attempt's files, mounted read-only in the implementer's box, and
the worker fetches from it; canonical itself is never mounted. A
candidate's base is the daemon's: the head at admission, then the latest
target handed to the attempt once that target is an ancestor of the head.
An independent writer retains its own recorded base until that ancestry is
proven. The selected candidate's base remains authoritative for downstream
judgments.
The `.yard` refusal and protected paths read `base..head`, so a merge of a
target that carries an operator's `.yard` change passes, and a head that
undoes one is refused.

`yard sync [BRANCH]` is the only operator ingress and egress. Omission selects
the project default. Importing fast-forwards only the selected canonical
branch from committed checkout history after validating its `.yard/config.toml`
and trusted Dockerfile. An absent canonical branch is created only from an
existing checkout branch with those inputs. Ticket admission creates no branch.
Consuming uses already imported history, without revalidating its declaration
of a former default. It fast-forwards only a clean checkout of that same branch to canonical;
a different checked-out branch or dirty checkout is refused by identity.
Divergence is refused naming both heads. No unrelated branch ref moves.

Named import leaves the default unchanged. `yard sync BRANCH --default`
explicitly selects an imported existing branch whose committed `target.ref`
names that branch. If checkout history is ahead, selection imports it first;
divergent history is refused. If canonical is ahead or the checkout branch
is absent, selection uses canonical's imported head and does not consume,
create or switch a checkout branch. Selection alone requires no clean or
matching checkout. Only consume has those preconditions. Validation precedes
mutations. An unresolved landing intent refuses the switch, naming its identity.
Live attempts keep their frozen branches; default changes affect future
admissions only.
The refusal reads live work before waiting for canonical and again under its
lock, so a held landing child cannot delay its named intent refusal.
Default selection, admission, workflow edits and ticket-proposal acceptance
serialize with project default configuration publication. Sync re-reads
the default under that lock after taking its target lock and before import
invalidation. Unnamed sync resolves its branch under both locks; a changed
default releases both locks and repeats selection with the current target.
An uncertain import or default-selection error does not
restore the input. Named sync also refuses a retained landing mutation owner
after taking the target lock. Body, title and priority edits, proposal
edit/link acceptance and rejection do not wait on canonical. Target edits
also do not wait on canonical; admission rechecks the revision and branch.
The existing queue is partitioned by frozen target, in approval order within
each target. A running or reserved landing or child, or an unresolved intent,
blocks only that target. A red-blocked approval is skipped without blocking
later eligible approvals on that target. Every landing effect uses the shared
project execution cap; there is no separate queue limit or lifecycle. Landings, named sync and
intent recovery lock their target. Each landing policy re-read and the
transaction it admits hold that same lock, so no sync falls between them.
Canonical branch updates are compare-and-swap. Symbolic HEAD changes and
default configuration publication use the project admission lock; committing
a landing intent also serializes with default selection.

**Landing.** Merge the candidate onto its frozen target branch head (a merge commit,
or a fast-forward when it already sits there), persist its prepared identity
before pinning, and verify. A pre-intent continuation uses that pinned input
after checked predecessor teardown. Retain eligible completed passing children,
qualifying passing references and exactly accepted failures for the unchanged
selected input. Run remaining verification.
Requested landing omissions pause on exact prepared attention before verification or intent. Explicit acceptance survives restart through the existing attention reading. Every selected gate runs or qualifies an immutable passing reference; fresh preparation inherits no decision. Recheck the prepared ref, omission decision and selected children before intent. Then, in one
store transaction that is serialised with every command that could
invalidate it (ticket edit, abandon, reject), re-read the approval
against current rows and policy and record a landing intent: target ref,
expected old head, verified merged head, the candidate and approval identities.
From that commit until the intent is
resolved, those commands are refused for those tickets. Then `update-ref`
with the expected old value; then `merge-base --is-ancestor` for the
candidate head; then record the landing, close the ticket and resolve the
intent. Yard never force-updates a ref and never reads command success as
a landing.

On restart, once the landing's lock proves its git children are over, an
unresolved landing intent is decided by its persisted target ref alone.
Legacy intents acquire that ref from their frozen attempt target during migration;
older live attempts are first backfilled from the previously fixed default.
Named sync refuses an unresolved intent on its target by execution identity;
unrelated targets remain available.

| Canonical's target | Meaning | Action |
|---|---|---|
| equals the expected old head | nothing landed | retire the intent; the candidate re-queues |
| equals the verified merged head | landed, unrecorded | record the landing once; no merge runs |
| contains the merged head | landed and moved on | record the landing once from the ancestry |
| none of these, or unreadable | unknown | keep every row, refuse that target's queue, raise `red` on the landing |

## Boxes

Yard builds the project image with `pinfold image build` before an
execution (see Threat model) and records the image id the box reports. A
box spec names mounts, env, the harness, egress and memory.

| Box | `/workspace` | Harness state | Egress | Env |
|---|---|---|---|---|
| implementer, ordinary sequential cleanup and repair | its declared writer clone, writable; `/yard/proof` writable | writable | model route, MCP route, `isolation.egress` | the harness's env, git identity, `YARD_MCP_BEARER` (from) |
| packaged sequential cleanup | the exclusive handoff clone, writable; its admitted proof snapshot at `/yard/proof`, read-only; selected instructions and launch input at `/yard/input`, read-only | fresh Pi, scratch and input state per execution; only matching Durable checkpoints continue after checked predecessor release | model route, MCP route, `isolation.egress` | native Pi env and execution bearer; controller inherits routing only |
| integration writer | its own retained writer clone, writable; `/yard/proof` writable; named candidate source/proof snapshots and bundles read-only under `/yard/candidates` | its own, writable | model route, MCP route, `isolation.egress` | implementer env and execution bearer |
| read-only audit | a fresh detached checkout of its recorded head or admission base, read-only; `/yard/proof` is its frozen available proof or empty directory, read-only; selected packaged instructions and launch input at `/yard/input`, read-only | exclusive to its execution and admitted node, writable; matching Durable checkpoints continue only after checked predecessor release | model route, MCP route, `isolation.egress` | the harness's env, git identity, `YARD_MCP_BEARER` (from) |
| standalone report | fresh exact-input checkout, read-only; no candidate proof mount | its own, writable; trusted environment from frozen target | model route, MCP route, frozen `isolation.egress` | harness env, git identity, execution bearer |
| standalone command | fresh exact-input checkout; resolved read-only or write-allowed access; no proof mount | none | none | `HOME`, `PATH`, `YARD_BASE` equal to input |
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
variables its `env` names, bounded output (its stdout and stderr on one pipe, in the order it wrote them, the last 1 MiB kept, so a red's detail is the end of the run) and its `timeout_minutes`. It exists for gates that
need what a box cannot give, such as a container runtime. `YARD_PROOF` is
the host path of the proof snapshot this execution judges: the candidate's
at the candidate stage, the approved candidate's at landing.

Harness state (a harness's session store and launch files) for a write-allowed
implementer is one directory per attempt under its files, mounted at one
fixed path. A read-only audit, packaged cleanup or review seat gets fresh harness session,
launch and scratch state per execution. A compatible Durable continuation
exclusively mounts its retained checkpoint after checked predecessor release.
Its row binds the admitted node and checkpoint. Whole-attempt cleanup removes
the state after checked release.
A fresh session is one that resumes no execution; a harness version change ends session
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

Ordinary workers get the project's own agent environment from the base, staged
at the harness's user level, and nothing from `/workspace` loads: the base is
the seat's candidate base, or the implementer's attempt base when the
execution starts, so the candidate never steers its own review. The base is
the operator's, so it loads whole, hooks included, in seats as in the
implementer; a base file that itself names a `/workspace` path, a hook
script or an instructions file, reads the candidate's copy. Yard reads the named paths from canonical's objects on the
host, never from a worker's tree: root `AGENTS.md` and `CLAUDE.md` (64 KiB
each), project skills (`.agents/skills/`, `.claude/skills/`,
`.pi/skills/`), Pi extensions (`.pi/extensions/`), `.claude/settings.json`,
and Codex's `.codex/config.toml`, `.codex/hooks.json` and `.codex/skills/`.
Only regular files are taken, copied as their bytes; a link or submodule
is left out, never followed. A file over 1 MiB, a set over 512 files or
8 MiB, and guidance that is not UTF-8 or together passes the 128 KiB prompt
bound each refuse the execution naming the path. Each harness gets the paths
it would load from a project, into its own state, and Yard's own keys ride
the harness's highest-precedence layer, its argv.

The selected packaged audit or cleanup uses an empty broad agent environment. Its native
Pi MCP extension is Yard-owned; only admitted selected canonical instructions
and references are staged separately at `/yard/input/trusted`.

Pi's launch argv starts or resumes a session, the normalisation of its
frames yields `started`, observational usage, `finished` and `failed`.
Its launch files include Yard's staged MCP client extension and `agent/models.json`, which
moves the provider onto its route. Discovery of extensions, skills, context
files, prompt templates and themes is off (`--no-extensions`, `--no-skills`,
`--no-context-files`). The base's extensions and skills are staged in the
agent dir and passed by path: `-e` for each extension Pi would discover (a
`.ts` or `.js` file, or a directory's `index.ts` or `index.js`), `--skill`
for each skill root. The base's guidance rides the system prompt. Pi's own
agent settings stay Yard's; project settings are not staged.

Claude runs through a login or direct Anthropic injecting route. For the
API route its environment holds a placeholder `ANTHROPIC_API_KEY` and
`ANTHROPIC_BASE_URL = http://anthropic.yard`. The route injects the real
`x-api-key` on the host; Claude supplies `anthropic-version: 2023-06-01`
and its protocol headers. `YARD_ORIGIN_ANTHROPIC` overrides the host origin.
Both modes use the same execution grant and read-only candidate rules.
Its launch files hold a `--mcp-config` naming only
the `yard` HTTP server, whose bearer comes from the env, and `--settings`
with its automatic-compaction setting. `--strict-mcp-config` keeps Yard's
server the only one; `--setting-sources user` keeps every workspace setting,
hook and plugin from loading, and Claude's user level, `.claude/` in its
home, holds the base's settings and skills: those of `.claude/skills/`,
then those of `.agents/skills/` under top-level names not already taken. An
earlier top-level entry hides the whole later subtree. `--setting-sources user`
also stops Claude loading project memory, so the base's `AGENTS.md` and
`CLAUDE.md` are appended to Claude's system prompt. The `system`/`init`
frame names the session id and the tools the server registered, and that is
the registration proof. The `result` frame is the outcome. The model and
effort ride the argv, `--resume` continues a session, and the upstream
credential stays a placeholder in the box.

Codex runs as a login too, and the attempt's `CODEX_HOME` is its user level, holding the base's
`.codex/config.toml`, `hooks.json`, skills and guidance (`AGENTS.md` then
`CLAUDE.md`), with `.agents/skills/` in its home. Yard's keys ride `-c`: the
`mcp_servers` table holding only the required `yard` HTTP server, whose
bearer comes from the env; the compaction threshold; `approval_policy =
"never"` beside the sandbox; `project_doc_max_bytes=0`, so the workspace's
`AGENTS.md` files do not load; `/workspace` pinned untrusted, so a staged
config cannot activate the workspace's `.codex/config.toml` (an inline
`projects` table, since a dotted key keeps its quotes); and the `pinfold`
model provider on the route. Codex merges a `-c mcp_servers` table into a
staged one, and a selected profile outranks `-c`, so the `mcp_servers` table
and `profile` are dropped from the staged config, the one file not copied
verbatim. Every launch passes `--dangerously-bypass-hook-trust`, since a
fresh `CODEX_HOME` holds no hook trust. Codex discovers skills in the
workspace whatever its settings say, so the argv disables each `SKILL.md`
under `/workspace/.agents/skills/` and `/workspace/.codex/skills/`
(`skills.config`), read from the seat's checkout or, for the implementer,
its clone when the execution starts; a link at any component of those
roots, a name that is not UTF-8, a tree past 32 directories or 512 skills,
or a list past the 128 KiB argument bound refuses the Codex execution.
Its `exec --json` stream starts a
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
authenticated by a per-execution bearer. The bearer's first `initialize` opens
its one session and records the connection before replying with a random
`Mcp-Session-Id` header, kept only in the harness's memory. Every later request must carry it; a second
`initialize` or a request without it is refused with 404 and names nothing of
the grant. Tools are granted by declared output independently of source
access. Audit is the report capability of a read-only implementer or
source-only cleanup, not a new execution kind:

| Tool | Kinds |
|---|---|
| `yard_context` — ticket context or standalone frozen scope | implementation, review, selector, audit, standalone report |
| `yard_progress` — one bounded note | implementation, review, selector, audit, standalone report |
| `yard_propose` — a ticket, an edit, a link | implementation, audit |
| `yard_publish_review` — findings; once | review |
| `yard_publish_selection` — one declared existing candidate input or exact finding groups, by admitted output; once | selector |
| `yard_publish_report` — typed report and declared outcome; once | audit, standalone report |

A standalone report grants only context, progress and report publication.
Worker requests for operator execution, attempt and sync operations are refused
as `operator_only` with their exact execution and recorded. Tool errors retain
their code, message and identity data in `structuredContent.error`.
Operator stdio performs the allowed operations in the same project.

Every payload is validated at the boundary (G6). A tool call outside the
grant is refused and recorded.

The selected packaged controller offers `read_declared`, `write_declared`,
`commit_source` and the anonymous native `yard_publish_report` bridge to its model.
The write and commit tools refuse without explicit cleanup source-write authority.
Pi retains the existing
audit grant and one MCP session; the child receives no bearer, session or
additional worker grant. No model-elected child-launch tool is offered.

## Daemon and CLI

`yard mcp` serves trusted operator MCP over stdin and stdout, one JSON-RPC
message per line. It is a client of the existing unix socket, with the same
build boundary as the CLI. It never opens a store or starts a daemon. Run it
only on the trusted host. Worker HTTP MCP capabilities are separate;
an execution bearer grants none of the operator tools.

Operator tool schemas keep node identities as strings for attempt acceptance
and repair. Standalone execution start takes a node definition object.

`initialize`, `notifications/initialized`, `ping`, `tools/list` and
`tools/call` are served. Operator tools are `yard_` plus the daemon method
with dots replaced by underscores. They cover `daemon.status`,
`project.list`, `project.forget`, `init`, `doctor`, `sync`, `status`,
`events`, `attention`, all ticket, attempt and proposal operations listed
below, `attempt.stats`, and `execution.start`, `execution.show`, `execution.stop`. Arguments name absolute registered `project`
paths; `init` and `project.forget` instead take an absolute `path`.
Required identities and argument types are declared by each tool's schema.
Unknown fields and wrong types are `invalid` before forwarding; daemon
validation remains authoritative. Results carry the same compact JSON in
text and `structuredContent`, as `{result}` or `{error: {code, message,
data}}`; errors set `isError`. Stale expected/current identities are intact.
Status and ticket lists omit bodies; `ticket.show` reads a body explicitly.
Transcripts and recorded statistics appear only through `attempt.tail`
and `attempt.stats`; missing recorded statistics remain null.

`events` returns one page of at most 500 audit events after `since`, waiting
at most 25 seconds if empty. `attention` is one bounded poll of the same
wake primitive as `status --watch`, with its `seq`, `attention` and `idle`.
Its 25-second bound covers the whole poll, including config and readiness
Git reads. Expiry returns the caller's `since` as `seq` (zero when absent),
no attention and `idle: false`. Incomplete inputs never establish an idle
board or advance the cursor past unseen attention.
Clients may poll again. `notifications/cancelled` with `requestId` releases
only the matching events or attention request and answers JSON-RPC error
`-32800`. Stdin EOF releases waits and the adapter exits normally after
accepted requests finish. Input errors, oversized frames and stdout errors
stop input, release waits and drain accepted mutation forwards before a
normal error exit. Completed requests are answered while stdout remains
writable.
An oversized frame first answers JSON-RPC `-32600` with a null id when
stdout is writable. Neither cancellation nor handled transport failure
stops an execution. Stdio accepts at most 64 concurrent requests and bounds
each input line to 1 MiB.

One daemon per machine, run by the user's service manager, serving every
registered project from one process and one scheduler tick. One unix socket
at `$XDG_STATE_HOME/yard/yard.sock`, which must stay under macOS's 104-byte
socket path limit. The daemon passes the host's `XDG_RUNTIME_DIR` and
`DBUS_SESSION_BUS_ADDRESS` to every pinfold child. On start the daemon reconciles
every project, the landing restart table and the box prune included, and
serves no command until that has committed. `yard daemon run` then prints
one JSON line, `{"event": "serving", "socket", "pid", "boundary"}`, to
stdout. `yard daemon install` writes the
service: a systemd user unit on Linux, a launchd agent on macOS, both
running `yard daemon run` from the installed binary's absolute path with
the installing shell's `PATH` and no other variable. It then restarts the
service and returns once the daemon answers, with its `pid` and `boundary`,
or fails as `restart` does. On
Linux it enables lingering so the daemon outlives the login session, and
says so where it cannot. `yard daemon uninstall` stops and removes it;
`yard daemon restart` goes through the service manager and returns once the
new daemon answers, with its `pid` and `boundary`.

The daemon owns the project registry, a list of absolute project paths in
`$XDG_STATE_HOME/yard/projects`. `yard init` registers a project and `yard
project forget` removes one and retires the project's image name
(`pinfold image rm`; a box's image stays until the box goes, and running
`forget` again retries); a registered path that is gone is skipped and
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
yard init | doctor | sync [BRANCH] [--default] | status [--watch [--since SEQ] | --history [--since SEQ]] [--json]
yard daemon run | install | uninstall | status | restart
yard project list | forget
yard ticket new | show | edit | park | unpark | depend | list | done | abandon
yard attempt start | stop | approve | reject | abandon | show | diff | tail
yard proposal accept | reject
yard mcp
yard version
```

`status --watch` returns on either of two conditions, prints the open items
and the seq it answered at, and exits; it is the wake primitive. One: an
attention item is open. Two: the board is idle, with no live attempt, running
or reserved execution, no open item and no ready ticket the scheduler would admit. `--watch --since SEQ`
returns once an item is open whose `attention.raised` event is after SEQ, or
the board is idle and an `attempt.ended` or `execution.ended` event is after SEQ, so a watch armed
on an idle board waits for more work to arrive and drain. It prints every open
item (none on an idle return); `--json` prints
`{"seq", "attention"}`, and passing that seq to the next call loses no item.
`status --history` follows
the audit stream from a sequence and prints one line per event; MCP `events`
reads bounded pages of the same stream. `attempt tail` prints the live transcript file as the
harness wrote it. Approve and reject require `--target BRANCH` as well as `--head SHA`.
A different target is stale and writes nothing. Status names the frozen
target of each live attempt and queued candidate. Every
command runs without a terminal and answers `--json`. `doctor` reports what
pinfold says of itself, the service, the project image, which connections
have a credential, and each configured login's pin and, where Yard holds its
token, whether it is set.

## Guarantees

These are the invariants: binding, cited elsewhere in this spec as G1 to
G15. Each is shown by one or more end-to-end scenarios; testing policy is in
`AGENTS.md`.

The guarantees are product requirements. Shown by describes representative
automated checks, not exhaustive coverage or a promise to test every option,
combination, lifecycle or historical version.

| # | Guarantee | Shown by |
|---|---|---|
| 1 | Only the queue lands | A real worker fails to push to canonical, reach its host path/socket, or use sessionless RPC. Explicit operator approval then lands that same candidate through the queue. |
| 2 | Judgments bind candidate, choice, ticket and admitted plan identity; integration publishes its own candidate bound to named inputs; Durable continuation binds frozen checkpoint inputs; prepared continuation binds exact merged input; operator policy stays live; bounded handoff publications require fresh judgments | A declared closed gate reuses unchanged evidence but changed tree/proof or unknown inputs rerun. New head, ticket revision and proof invalidate judgments. Protected-path policy changes withdraw automatic approval; manual approval remains explicit. Stale candidate acceptance names the changed identity and writes nothing. Five distinct implementers receive their own frozen reviews; exact picker and replacement bindings select the only checked and landed candidate. Named-input integration publishes C containing both fixture changes; stale A approval writes nothing and only C is approved and landed. Ordinary writable agents retain selected admitted blobs across source edits; staging adoption binds new input identities and starts a fresh session. Same-tree repair publications bind distinct producer and judgment identities before native landing. Finding selections reject foreign and stale exact source identities, retain all duplicate contributors and historical inputs, and require fresh inputs after revision or candidate change. |
| 3 | Landing is compare-and-swap with selected verification and durable exact prepared merged-ref evidence | A held native ref update stays undecided until its outcome is known. Canonical movement before compare-and-swap retires the landing without overwriting newer history. |
| 4 | Capacity holds | Concurrent starts preserve one live attempt and the lane bound. Two held mixed reader/worker effects have exactly two native project boxes; the join waits for exact results. One-slot landing children exclude a worker and still land. An inherited native owner retains capacity across restart. Identical execution IDs in different projects keep separate grants. |
| 5 | Intent precedes effect, and a restart loses only the turn; Durable continuation is a new exclusive execution after proved predecessor removal; individually useful work needs no ticket | A real held writable turn survives daemon death as interrupted work and resumes its retained tree. Startup removes a surviving host gate process group before requeue. A legacy owner without trustworthy lock proof remains reserved across upgrade. Matching Durable unsafe intent resumes only after old-box removal; corrupt checkpoint identity refuses before a fresh retry. A completed registered finding picker retains its immutable selection, reservation and exact ownership item across restart while a real runtime descendant holds its inherited effect lock after box removal; checked release reconciles only that item without replay. |
| 6 | Inputs are validated at the boundary; new supported workflow admissions retain immutable resolved input evidence, and unrecorded history stays unknown; standalone scope is explicit | Malformed worker MCP input writes no proposal; the valid same-turn control succeeds. Unsupported selected admission creates no work, while a supported held execution keeps its frozen inputs across template sync. A real historical store preserves rows, audit and foreign keys and supports current decisions after upgrade. |
| 7 | A ticket lands end to end and leaves only rows; usage is optional measured data with provenance; Codex token totals retain their meaning and unreported monetary cost is unknown | A real ticket goes through commit, gate, review, approval and native landing, then leaves rows while unrelated live work remains. Actual optional Pi frames retain unknown, interrupted partial and resumed explicit-zero measurements across reopen. Operator stdio preserves worker authorization and stale mutation identity while canceled waits and stdin EOF leave work running. Standalone command/report execution has no ticket or landing authority. |
| 8 | Reviews, audits and read-only candidate and finding selections are explicit bounded publications; an empty selected panel is unreviewed; selected eligible merged failures require exact operator decisions | Silent or unregistered review publication cannot authorize approval; a proven same-project retry can. Report validation and held publication preserve exact frozen identity and eligibility, and successful text alone creates no report. Five-model selection binds own reviews and one replacement to actual landing. A correctly resealed changed packaged controller refuses before model/code effects; its unmodified control publishes through real Pi. The exact multi-model join picker refuses malformed groups and repeated contributors before a valid subset and two-source duplicate publication; checked ownership release makes one typed finding selection inspectable, preserves raw sources and grants no verdict, choice or landing authority. |
| 9 | Each implementer execution starts from its frozen target and the right candidate; independent writers retain isolated reusable workspaces, including integration from named candidates; the bounded implementer, cleanup and repair path retains one exclusive clone, including selected Durable cleanup | A held worker receives operator edits and adopted definitions in its next execution. Stop/start delivers the edit while retaining configured model identity. Five independent writer workspaces preserve their own bytes, sessions and frozen read-only review inputs through continuation and restart. Held integration retains partial work across restart; source writes and canonical access fail while its own write succeeds. Conflicts require repair before selection. Boxed Git inodes, fixture bytes and ancestry survive sequential selected roles; a held cleanup refuses a competing workspace retry, and checked release precedes handoff. Durable cleanup resumes a held unsafe source write in a new exclusive execution after checked release, without replay; its boxed commit passes the same clone to ordinary repair and native landing. |
| 10 | The queue lands one at a time per target and re-judges what does not merge | Two approvals land in order; a red merge returns for repair. A feature ticket uses its frozen target source, definitions and image, repairs a real conflict, and lands only on that branch. |
| 11 | Only the operator's sync changes `.yard` and canonical from outside | A worker configuration commit is refused before gates; operator sync makes the same configuration available to future admission. Named-branch import, divergence refusal and consume preserve main and another imported branch, with dirty/other-checkout refusals and allowed controls. |
| 12 | Boxes hold nothing secret | Real Pi, Claude login/direct API and Codex routes keep upstream credentials on the host. Candidate extensions, hooks, config and guidance cannot alter their own review; trusted base inputs remain usable. A real packaged controller has no upstream key in environment, mounts, transcript or stored content. |
| 13 | Every gate, seat and read-only audit runs where its row says; selected ordinary inputs, packaged cleanup and standalone source access are contained; source access and publication grants are independent | Boxed gates and reviews read the approved proof snapshot through read-only mounts and restricted routes. Host gates receive only named environment and the correct candidate/merged base. A superseded approval starts no host landing gate. An ordinary writable worker reads admitted skill/reference bytes after canonical and candidate edits, refuses trusted-input writes while source writes succeed, and excludes unselected guidance. Separately selected cleanup and repair settings start fresh sessions in one clone; writable cleanup publishes only a report and its operator landing call refuses with the exact execution. The same project refuses the read-only Durable audit's write and permits selected cleanup's write and commit; cleanup publishes only a report, repair alone publishes the final candidate, and landing waits for operator approval. Packaged inspection refuses Gitlinks without running a nested filter that ordinary boxed Git executes in the same project. Distinct declared reviewer models and the post-join finding picker bind exact immutable review inputs and read-only candidate/proof mounts. |
| 14 | Worker git is untrusted, and Yard stays local | A worker plants Git hooks, fsmonitor, filter, remote and a harness-state link and corrupts an object. Host markers stay absent, canonical fsck stays clean, refusal names the corrupt identity, and a legitimate recovery succeeds. |
| 15 | Plans and proposals resolve | A read-only planner proposes bounded children and a body edit; stale edits preserve identity and rows. Accepted dependencies block the parent, and a writable dependent retains its ordinary workspace authority. Closing a live waiting attempt refuses. Actual legacy proposals and limit overrides remain usable after migration. |

## Code

Rust. One binary, `yard`, client and daemon. Linux builds are static musl.

| Target | Role |
|---|---|
| `aarch64-apple-darwin` | Mac |
| `aarch64-unknown-linux-musl`, `x86_64-unknown-linux-musl` | Linux hosts |

Rust dependencies: `tokio`, `hyper` with `hyper-util` and `http-body-util`,
`serde`, `serde_json`, `toml`, `rusqlite` (bundled), `nix`, `sha2`,
`getrandom` (MCP bearers), `clap`. Git and pinfold are child processes.
The trusted boxed Durable package uses Node 26.11.1 and the locked npm
dependencies described in Publication; neither is a host runtime dependency.
Native registration and session use pinfold's real Pi artifact. Packaged
asset hashes bind the image-owned launcher/controller/lock separately and
introduce no pinfold artifact or host controller.
CI fetches its public build inputs before the host suite. Host `curl` fetches
archives, `jq` reads the exact lock, and `sha256sum` and OpenSSL verify Node
SHA256 and npm SHA512 respectively. These build tools execute no package code.

```
crates/yard/src/
  store/    mod.rs (open, migrate, audit) schema.rs tickets.rs attempts.rs executions.rs checks.rs
  jobs/     mod.rs (load, step, advance) admit.rs supervise.rs review.rs queue.rs reconcile.rs cleanup.rs proof.rs
  git.rs box.rs harness.rs agent_env.rs pi.rs pi-mcp-extension.ts claude.rs codex.rs mcp.rs
  daemon.rs api.rs cli.rs config.rs main.rs
crates/e2e/  src/lib.rs (harness, helpers) model.rs (fake model fixture), tests/guarantees/main.rs g*.rs
```

## Release

A direct API reviewer patch is verified before the operator switches build
review profiles. Release preparation includes version metadata and a dated
checklist with whole-tree cleanup, a test-audit sweep, delete/add ratio,
largest source file and this spec's line count. The operator passes this
spec and README before release.

Verification requires fmt, locked all-targets clippy, build, real host e2e
on Linux x86_64 and arm64, and the operator's Mac release checks. The patch
artifact is checked against the scripted direct API fixture on isolated
state; its hash and schema compatibility are recorded. Development binaries
never open the construction backlog or mutate the running service.
Publication, installation, credentials, configuration sync and runtime
switchover are separate operator actions. Construction keeps its frozen
binary until the operator authorizes the verified patch.

## Open questions

- `replay` is out: whole-attempt disposal keeps no diff to seed from. The
  branch stays in canonical; an operator who wants it cherry-picks.
- Batching several candidates into one landing, with bisection, is out
  until queue wait is measured.
- Whether Yard later links pinfold as a library instead of a process.
- The Pi MCP client extension stays TypeScript inside the Rust binary as an
  embedded file; whether pinfold should carry it instead.
