---
name: yard-file
description: File work into a Yard project. Admit or reject proposals by the project's rule, size tickets to one attempt, write bodies a worker and a reviewer can judge against, and send Yard's own defects upstream. Load before creating or editing a ticket, deciding a proposal, or turning a report into work.
---

# yard-file

A ready ticket starts on the scheduler's next tick. A ticket is ready when
it is open, not parked, and every ticket in its `--depends-on` is done. So
when you mean to read a ticket before anything spends on it, file it
`--parked`, then `yard ticket unpark` it. Parking it after filing is a race
you lose.

## Admission

Admit work for a failure that happened or a use somebody has. Do not admit
it for being a good idea, for symmetry, for extensibility, or for hardening
beyond the threat model. The project's own rule, usually in `AGENTS.md`,
governs.

When you reject, record the condition that would re-admit the work:
`yard proposal reject ID --text "re-file if X happens"`.

Proposals:
- **Accepting creates the ticket as written, and it starts if ready.** If
  it should wait, reject it and file it yourself `--parked`.
- **An edit proposal pauses its attempt.** Decide it now.
- **Decide other proposals once their attempt has settled.** The next
  repair often covers them.

## Size

- **One ticket is one change you can read in one sitting.** Split larger
  work into tickets linked with `--depends-on`.
- **`--workflow plan` makes a read-only planner propose children and a
  body.** It does not make oversized work legitimate.
- **A frightening size means the scope is wrong.** Cut or defer behaviour
  before splitting.

## Body

The worker gets the body and the repository. The reviewer judges against the
same body. Write:
- the mechanism, traced to file and function;
- who the change serves;
- the behaviour afterwards, as a list of what is observable;
- what proves it: the test to extend, or "no test" and why;
- what is out of scope.

Include the evidence you traced it from. Do not write the implementation.

A candidate that touches `.yard/` is refused. When the work needs a package,
a gate or a config change, the ticket says so, and the operator makes that
change with a commit and `yard sync`.

## Yard's own defects

Yard doing what its spec or help says it does not do is a defect in Yard,
not work for this project. Report it on Yard's repository: one observation
per issue, with the command and its `--json` output, keys and private paths
redacted. Work around it here meanwhile.
