---
name: test-audit
description: Judge switchyard's end-to-end tests. Gate a change that adds or changes a test, and sweep the suite for assertion blocks that fail the bar in AGENTS.md, deleting by default. Load this when a change touches crates/e2e, or when asked to audit, prune or review tests.
---

# /test-audit

AGENTS.md's Tests section is the policy, and the guarantees table in
ARCHITECTURE.md is the list. The unit is the assertion block: each one
shows a clause of a guarantee row's "Shown by".

## Gate: a change touches a test

The diff and its message must answer all four questions; send the change
back if one is missing.

1. Which guarantee row and which "Shown by" clause does it show? If no
   clause describes the scenario, add one in the same commit.
2. Which change to the binary makes it fail? The test's comment names it.
3. Why don't the row's existing blocks already catch that failure, and is
   this the row's hardest case?
4. Where does each expected value come from outside yard?

A crash scenario also says where the daemon is held, and how the test
proves the state from outside before restarting. No tag in `sweep.md` may
apply to the new blocks.

## Sweep

Launch the agents in one message with the Agent tool
(`subagent_type: "general-purpose"`, `model: "opus"`), each told to edit
nothing. Give each two or three files from
`crates/e2e/tests/guarantees/g*.rs`, never splitting a file. Prompt: the
file paths and "Follow `.agents/skills/test-audit/sweep.md`."

## Judge and land

1. Merge the lists and dedup twins across groups.
2. Settle any doubted sabotage: apply it, run that test
   (`cargo test -p e2e --locked g03_`), and revert it.
3. Send these to the operator as spec changes, and wait for the answer:
   - a deletion that leaves a "Shown by" clause with no block;
   - every unshown clause.

   The answer is either to cut the clause or to write a block for it.
4. Everything else: delete the block and the helpers it orphans, and edit
   the row's "Shown by" to match. One commit per test file.

A kept block that fails on the branch is a product bug. Reproduce it
through the CLI and ticket it; never delete it.

Before each push, run `cargo fmt --check`, `cargo clippy --all-targets
--locked -- -D warnings`, and the suite on the host with `cargo test -p
e2e --locked`.

Report in the code-cleanup file when this runs inside that pass. Otherwise
write `docs/archive/YYYY-MM-DD-test-audit.md` with:

- each disposition and its row
- the operator's list with the answers
- the test lines removed beside the binary lines removed
