---
name: yard-operator
description: Drive Switchyard (yard) for this project: file tickets, answer attention, land work.
---

# Operating yard

- `yard status --json` shows tickets, attempts and open attention.
- `yard status --watch --since SEQ --json` follows the audit stream; it is the wake primitive.
- `yard ticket new --title T --body B` files a ticket; its body is its plan.
- Answer attention with the commands each item names: `yard attempt approve Y-n --head SHA`,
  `yard attempt start Y-n`, `yard attempt nudge Y-n --text T`, `yard proposal accept ID`.
- `.yard/` changes only through a commit in this checkout and `yard sync`.
