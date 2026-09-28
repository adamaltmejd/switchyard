---
name: yard-file
description: File work into a Yard project. Decide a proposal by the project's admission rule, size a ticket to one attempt, write a body that a worker and a reviewer can both judge against, and send Yard's own defects upstream. Load this before creating or editing a ticket, deciding a proposal, or turning an issue, report or finding into work.
---

# yard-file

Filing is the cheapest decision to get right. Scope you never file costs
nothing. Scope that reaches a worker costs a box, the gates, a review, and
every round they spend converging. `yard-drive` covers answering the board.

**The scheduler admits a ready ticket on its next tick.** A ticket is ready
when it is open, unparked and has no unfinished dependencies. File with
`--parked` whenever you mean to read or edit the ticket before anything
spends on it, or when its preconditions do not hold yet. Run
`yard ticket unpark Y-n` when it is ready. Parking it afterwards is a race
you will lose. `--depends-on Y-m` holds a ticket until Y-m is done.

## 1. Admission

Accept what a named failure admits, and reject the rest. A ticket earns its
place through a concrete failure that happened, or a use that somebody has.
Being a good idea is not enough, and neither are symmetry, speculative
extensibility, or hardening beyond the project's threat model. The project's
own admission rule, usually in `AGENTS.md`, governs, and it is stricter than
instinct.

When you reject, the reason is what you leave behind. Run
`yard proposal reject ID --text` with the condition that would re-admit it,
for example "no observed failure; re-file if X happens". "Not now" teaches
nobody.

For proposals:

- `yard proposal accept ID` runs the proposal's command against current
  state. A ticket it creates starts at once if it is ready, so accept only
  what you would file yourself, as written. If it must wait and the worker
  did not propose it parked, reject it and file it yourself with `--parked`.
- An edit proposal on the worker's own ticket pauses that attempt until you
  decide it. Decide it now.
- Decide any other follow-up proposal after its origin attempt settles. The
  next repair round often fixes it.
- Accept each proposal because you decided it, never to clear the board.

## 2. One attempt's worth

- A ticket is one cohesive change that a worker can carry to a candidate you
  can read in one sitting. Work that crosses several boundaries is a
  sequence you have not written down yet. File it as several tickets linked
  with `--depends-on`. The worst v2 ticket took fourteen worker executions
  and nine reviews.
- `--workflow plan` buys a read-only planner that proposes child tickets and
  a body edit. It investigates and decomposes. It does not make oversized
  work legitimate: a plan that comes back as a big backlog is telling you
  the ticket was wrong.
- Every ticket names who it serves and what they observe afterwards. A
  ticket without that is one nobody can tell is finished.
- Problems that have not happened are not requirements. A frightening size
  means the scope is wrong. Delete or defer behaviour first, and split off
  only what is useful on its own.

## 3. The body

The worker gets the body and the repository. The reviewer judges against the
same body. It carries five things.

- **The mechanism**, traced to a file and a function, not just the symptom.
- **The consumer**: who gets the behaviour, and in what situation.
- **The behaviour**, as a list of what is true afterwards, stated as what
  someone observes. A helper the change must reuse goes on this list.
- **What proves it.** Name the test to extend, and where the proof does not
  go, for example "one block in the existing G9 scenario; no new file". Or
  write "no test" and why, following the project's test rules.
- **Out of scope**: the neighbouring work, and the generalisation a worker
  would otherwise infer.

The evidence you traced it from is worth its length. An implementation for
the worker to type is design done in the wrong place.

`.yard/` belongs to the operator, and a candidate that touches it is
refused. A ticket that needs a package, a gate or a configuration change says
so and leaves `.yard/` alone. The operator makes that change with a commit
and `yard sync`.

## 4. Yard's own defects go upstream

A stopped attempt, a blocking finding or a rejected proposal is the loop
working. Yard doing something its spec or help says it does not do is a
defect in Yard, not work for this project. Report it on Yard's repository,
one observation per issue, after searching for a match. Include the command,
the `--json` output or event lines, and the daemon log excerpt, with keys and
private paths redacted. Work around it here in the meantime. In Yard's own
repository the report is a ticket like any other, filed under its admission
rule.
