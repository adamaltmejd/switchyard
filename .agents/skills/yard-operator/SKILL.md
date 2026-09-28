---
name: yard-operator
description: Drive Switchyard (yard) for this project: file tickets, answer attention, land work.
---

# Operating yard

Yard turns tickets into landed code. `yard status --json` shows `tickets`,
`attempts`, `attention`, `queue`, `running` and `seq`. Answer attention as
it opens; the scheduler runs everything else. Wait with `yard status --watch
--since SEQ`, the `seq` from the last `status --json`; it is the only
history view.

## The loop

`yard ticket new --title T --body B` files a ticket; the body is the plan.
Add `--priority P`, `--depends-on Y-n`, `--parked`, or `--workflow plan`.
A planning worker changes no code: it proposes child tickets and an edit to
its own body. The scheduler admits ready tickets, so a ticket starts on its
own once it is open, unparked and its dependencies are done. `yard attempt
start Y-n` admits one now, and is the same command a `stopped` or `red`
item's exit names.

## Attention

`status --json` lists each open item with its `kind`, `reason`, `ticket`,
`attempt` and `exits`. Answer with an exit the item names.

- `approval` — first read `yard attempt show Y-n` and `yard attempt diff
  Y-n`, then `yard attempt approve Y-n --head SHA`, `yard attempt reject
  Y-n --head SHA --text T`, or `yard attempt abandon Y-n`. Approval binds
  the exact `--head`; a reject sends its notes back as a repair.
- `proposal` — `yard proposal accept ID` or `yard proposal reject ID`,
  where `ID` is the attention id.
- `stopped` — `yard attempt start Y-n`, `yard attempt nudge Y-n --text T`,
  or abandon. On `timeout` and `limit` only nudge and abandon; a nudge's
  text reaches the next execution.
- `red` — the same three, except a landing's item (no `attempt`) exits only
  `yard attempt start Y-n`.

## Judging and syncing

- `yard attempt tail Y-n` prints the live transcript.
- `yard ticket edit Y-n --revision R --body B` needs the revision it read.
  A stale `--revision` changes nothing; re-read `status` and retry.
- `yard sync` imports the checkout's commits, `.yard/` changes included,
  and consumes landed work back into the checkout. A divergence is refused,
  naming both heads.
- `yard doctor` reports pinfold, the configuration, the image and the
  credentials; run it when a connection or the image looks wrong.
