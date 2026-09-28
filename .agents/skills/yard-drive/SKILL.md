---
name: yard-drive
description: Answer a Yard board. Hold the wake loop across every decision, answer attention with the exits it names, read a candidate before approving it, handle findings below the blocking level, and retire a ticket in the order that works. Load this when asked to operate, drive, watch or babysit a Yard project, or before answering a `yard status` attention item.
---

# yard-drive

Yard runs the work; you take the decisions between its steps. `yard-file`
covers filing and proposals; `yard-operator` lists the commands.

## 1. The loop is wake-driven; re-arm after every decision

`yard status --json` gives `seq`. `yard status --watch --since SEQ --json`
prints every later event as one JSON line and keeps running. Act on the
events that need you (`attention.raised`, `landing.recorded`,
`attempt.ended`), and read `status --json` for what is open.

After every decision (approve, reject, nudge, start, accept, abandon), your
next command is the watch. The decision is usually what produces the next
event. In v2 an operator who stopped watching after a decision left a
candidate waiting for approval until someone else noticed. It happened
twice.

Nothing running and nothing ready does not end your shift. It means the
next event is something you do. If there is nothing to do, say so and hand
over the `seq`. The open items in `status --json` are decisions you still
owe. Their events have already fired, so a watch from a later `seq` never
replays them.

## 2. Answer attention with the exits it names

Each item in `status --json` carries `kind`, `reason` and `exits`. Answer
with one of those exits. Do not substitute a neighbouring mechanism you
reasoned your way to. In v2, reaching for a reject where a nudge was the
exit cost two gate runs.

- `start` means "try again from here". It starts the next implementer
  execution, reruns the failed check on the same candidate, or re-queues a
  landing. It never makes a fresh attempt.
- `nudge --text` queues guidance for the next implementer execution and
  never interrupts. On `timeout` it renews the total-work clock. On `limit`
  it buys one more review round.
- A stop you cannot see a way out of is a stop you have not finished
  reading. Read `reason`, `yard attempt show`, and the exit's `--help`. The
  worst v2 outcome was an operator who decided a timed-out worker had no
  recovery and abandoned a healthy attempt. The nudge exit was printed
  under the item the whole time. If you still see no way out, leave the item
  open and ask. An open item costs nothing.
- An item that is gone was answered. Find out what answered it before you
  act again.

Everything Yard knows is reached through a command. Never edit
`.yard/local`, never remove a clone or a box by hand, and never kill a
process to make a state go away. When liveness or an outcome is unknown, do
not duplicate work, land, delete or kill. Read again, or ask.

## 3. Approval is a read

Before `yard attempt approve Y-n --head SHA`:

1. Run `yard attempt show Y-n`. Read `base..head`, each gate's check, the
   review's findings, and any protected paths the candidate touches.
2. Run `yard attempt diff Y-n`. Read it against the ticket body: does it do
   what the ticket asks, and nothing the ticket does not ask?
3. Approve with the full head you read. Approval binds that exact
   candidate; a new commit leaves it counting for nothing.

These reads come from the host CLI. Nothing the worker says is a reading of
the candidate, not its progress notes and not its commit messages. That is
the party being judged describing its own work. A passing review is one
reader against a threshold. A gate proves what its command asserts. Neither
approves anything: you do, against the ticket.

`protected` on an approval item means the candidate touches the guidance
every worker reads (`AGENTS.md`, `.agents/` and the like). Read those hunks
as a change to every future worker's instructions.

Approving is not landing. The landing is the `landing.recorded` event.
After it, `yard sync` brings the landing into your checkout, and
`git log -1` shows that it arrived. A landing that goes red becomes a repair
on its own. A second consecutive red raises `red`.

If the read does not convince you, run
`yard attempt reject Y-n --head SHA --text "what to change"`. The notes
become the repair's prompt. A reject is cheap next to a revert.

## 4. Findings below `blocking`

A pass can carry findings below the project's `blocking` level: the reviewer
saw them and chose not to block. There are three answers.

- **Land.** This is the usual one. Pre-existing debt, style, or
  extensibility nobody asked for does not hold up a candidate that does what
  its ticket asks.
- **Reject with notes naming the finding.** Only for integrity findings:
  evidence, a guard, an identity binding, a fail-safe path, or data that
  cannot be rebuilt.
- **File a ticket** for a remaining finding that clears the project's
  admission rule (see `yard-file`). Findings are rows and survive cleanup,
  but nobody reads them again unless they are filed.

## 5. Nudge, edit or retire

- A nudge steers the implementer. The reviewer never sees it; the reviewer
  judges against the ticket body. When what should change is what the ticket
  asks, run `yard ticket edit Y-n --revision R`. The edit supersedes every
  check taken on the old revision, and the next round is judged against the
  new body.
- An edit reaches work in flight, but it does not unbuild that work. When
  the premise has moved so far that the built work is wrong rather than
  unfinished, retire the attempt and file a fresh ticket naming the old one.
  In v2 one attempt spent half its budget escaping an amendment that a fresh
  ticket would have avoided.
- While a landing records its intent, Yard refuses edits, rejects and
  abandons on that ticket. Let the landing finish.

To retire a ticket, go in this order:

1. `yard ticket park Y-n` takes it out of automatic admission.
2. `yard attempt abandon Y-n --reason R` ends the attempt and takes its
   boxes down. The rows remain, and the branch stays in canonical.
3. `yard ticket abandon Y-n --reason R`, or `yard ticket done Y-n --reason R`
   if the work happened elsewhere.

An abandon without a park returns the ticket to ready, and the scheduler
admits a fresh attempt at once. So decide before you abandon: to discard
only the attempt, leave the ticket unparked and a fresh attempt starts; to
discard the ticket, park it first.

## 6. Configuration and upgrades

A `.yard/` change reaches the next execution once `yard sync` imports it; no
restart is needed. A new `yard` binary needs `yard daemon restart`, which
interrupts every running execution. Each one comes back as `stopped` with
the reason `interrupted`, and its `start` exit continues it. So restart
between executions where you can.

## When to stop and ask

Hand over, with the exact line you are looking at, when:

- a refusal names something you cannot fix;
- after reading, no exit on an item fits;
- liveness or an outcome is unknown;
- approving would mean approving a diff you do not understand;
- a decision would change a guarantee, the threat model or the scope.

Then watch again.
