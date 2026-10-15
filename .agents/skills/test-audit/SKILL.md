---
name: test-audit
description: Judge switchyard's end-to-end tests. Gate a change that adds or changes a test, and sweep the suite for assertion blocks that fail the bar in AGENTS.md, deleting by default. Load this when a change touches crates/e2e, or when asked to audit, prune or review tests.
---

# /test-audit

AGENTS.md's Tests section is the policy, and the guarantees table in
ARCHITECTURE.md is the list. The unit is the assertion block: each one
shows a clause of a guarantee row's "Shown by".

Keep a small risk-based core. Prefer checks for unauthorized effects, stale
identity, corruption and recovery, ownership, isolation, secrets and
representative end-to-end use. A unique mechanism or existing Shown by clause
does not by itself justify its runtime and maintenance cost. Product controls
remain exact; automated coverage is deliberately selective.

## Gate: a change touches a test

For a new or changed assertion block, answer:

1. Which material failure and product guarantee does it protect? Why is the
   impact worth the fixture cost?
2. Which change to the binary makes it fail? Name it in the comment.
3. Can an existing hard case replace this work? Count fixture machines,
   historical builds, boxes and model turns, not only test functions.
4. Where does each expected value come from outside Yard?

A crash needs a genuinely held point and externally proved state before
restart. Keep meaningful allowed controls for retained refusals. Add neither
replacement tests nor a coverage matrix merely because a behavior is untested.

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
3. Ask the operator before changing a product control, promised behavior or
   guarantee. Reducing automated coverage alone does not change the product
   contract and needs no further approval within an authorized audit. Judge
   unique cases by risk and cost too. Record deliberately untested behavior
   and narrow Shown by to the checks actually retained. Do not turn every
   absent observation into new work.
4. Delete invalid, duplicate or lower-value blocks and orphan helpers. Keep
   Shown by accurate and product guarantees unchanged. One commit per test
   file, including its module registration and dated disposition where needed.

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
