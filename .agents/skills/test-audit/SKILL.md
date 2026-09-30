---
name: test-audit
description: Judge switchyard's end-to-end tests. Gate a change that adds or changes a test, and sweep the suite for assertion blocks that fail the bar in AGENTS.md, deleting by default. Load this when a change touches crates/e2e, or when asked to audit, prune or review tests.
---

# /test-audit

AGENTS.md's Tests section is the policy, and the guarantees table in
ARCHITECTURE.md is the list. The unit is the assertion block: each one
shows a clause of a guarantee row's "Shown by".

Keep the smallest suite that proves those clauses. Exhaustive coverage of
commands, options, combinations and hypothetical contingencies is not a
goal. A new scenario must expose a distinct way a guarantee could fail.

## Gate: a change touches a test

For new or changed assertion blocks, the diff and its message must answer
all four questions; send the change back if one is missing.

1. Which guarantee row and which "Shown by" clause does it show? If no
   clause describes the scenario, add one in the same commit.
2. Which change to the binary makes it fail? The test's comment names it.
3. Why don't the row's existing blocks already catch that failure, and is
   this the hardest case for that mechanism?
4. Where does each expected value come from outside yard?

A crash scenario also says where the daemon is held, and how the test
proves the state from outside before restarting. No tag in `sweep.md` may
apply to the new blocks.

## Sweep

Use the available delegation tools within their concurrency limit, or
read sequentially. Reviewers edit nothing. Give each two or three files from
`crates/e2e/tests/guarantees/g*.rs`, never splitting a file. Prompt: the
file paths and "Follow `.agents/skills/test-audit/sweep.md`."

## Judge and land

1. Merge the lists. Read all guarantee files for twins across groups,
   including blocks no reviewer flagged. Compare mechanism and observation
   and name the survivor.
2. Settle any doubted sabotage: apply it, run that test
   (`cargo test -p e2e --locked g03_`), and revert it.
3. Before removing or weakening a guarantee or a "Shown by" clause,
   send the proposed change to the operator and wait for the answer. A
   deletion that leaves a clause without coverage counts as weakening it,
   even if the spec text is unchanged.

   For an unshown existing clause, add focused coverage within the
   authorized audit. Apply the Gate above; no operator approval is needed
   to add coverage while preserving the existing promise.
4. Delete invalid or duplicate blocks and the helpers they orphan. Keep
   the row's "Shown by" accurate without weakening it. One commit per
   test file.

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
