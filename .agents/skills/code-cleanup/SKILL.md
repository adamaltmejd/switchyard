---
name: code-cleanup
description: Audit switchyard's whole tree for yagni, duplication, fixes at the wrong level and unmeasured cost, then land the deletions one commit per module, or write a ticket for a rework. Hands every finding that touches a control or removes a feature to the operator. Load this when asked for a cleanup, yagni, simplification or consolidation review, and before every release.
---

# /code-cleanup

Start from a clean, green tree. Read the whole tree, not a diff.

## 1. Measure

Take these now and again at the end. The base is the last tag, or the root
commit before the first release.

```sh
base=$(git describe --tags --abbrev=0 2>/dev/null || git rev-list --max-parents=0 HEAD)
git diff --shortstat "$base"..HEAD
wc -l $(git ls-files 'crates/*.rs' 'crates/*.ts') | sort -rn | head -3
wc -l docs/ARCHITECTURE.md
```

## 2. Mechanical checks

Run these yourself; each hit is a finding for step 3.

- **Dependencies.** `cargo tree -p yard --depth 1 -e normal` against the
  list under `## Code` in ARCHITECTURE.md. `cargo tree -p e2e --depth 1
  -e normal` against what the harness needs to observe from outside.
- **Suppressions.** `rg -n 'allow\(' crates`. Each needs a comment saying
  why the lint is wrong there.
- **Child processes.** `rg -n 'Command::new' crates/yard/src`. Each spawn
  needs an explicit cwd, a scrubbed environment, and bounded output and
  time. A git or pinfold spawn outside `git.rs` or `box.rs` is `dup:`; a
  missing control is `spec:`.
- **Deferrals.** `rg -n -i 'TODO|FIXME|XXX|revisit' crates`. A TODO breaks
  AGENTS.md. A revisit trigger that has fired, or that names no trigger, is
  a finding. So is an entry under `## Open questions` that the suite now
  answers.

## 3. Read, in parallel

Launch every agent in one message with the Agent tool
(`subagent_type: "general-purpose"`), each told to edit nothing:

- **Modules**, `model: "opus"`. Group related files from the `## Code`
  tree, never splitting a file: roughly one agent per `jobs/` pair, one for
  `store/`, and so on. Add `crates/e2e/src/`, and ARCHITECTURE.md with
  AGENTS.md as one more module. Prompt: the file paths, the step 2
  findings that touch them, the rejected list from the newest
  `docs/archive/*-code-cleanup.md`, and "Follow
  `.agents/skills/code-cleanup/module.md`."
- **Traces**, `model: "sonnet"`, with the prompt "Follow
  `.agents/skills/code-cleanup/trace.md`."
- **User docs**, `model: "sonnet"`, with the prompt "Follow
  `.agents/skills/code-cleanup/docs.md`."
- **Tests**: `test-audit`'s sweep agents.

## 4. Judge

Merge the lists and dedup findings on the same mechanism. Before sorting
a finding, read its lines, and check any claim of "one caller" or "no
setter" with `rg`. Then sort it into one of three piles:

- **Land.** One module, no test assertion changes, names what it removes
  (AGENTS.md's admission rule).
- **Operator.** Every `spec:` finding, and any other finding that touches a
  control, removes a feature or deletes a guarantee row. List each with the
  spec line it changes and why. Stop until the operator answers. A yes
  lands with its ARCHITECTURE.md edit in the same commit.
- **Rejected.** Everything else, each with the condition that would
  re-admit it.

## 5. Land

Edit directly, one commit per module, and name what was removed in the
message. Each commit passes:

```sh
cargo fmt --check
cargo clippy --all-targets --locked -- -D warnings
cargo build --locked
```

After a commit that touches a guarantee's mechanism, run that guarantee:
`cargo test -p e2e --locked g03_`. Before the push, run the whole suite on
the host with `cargo test -p e2e --locked`.

A rework redesigns a module rather than deleting from it. Don't land it;
write it as a ticket: a title, plus a body that names what it removes.

## 6. Report

Repeat step 1. Write `docs/archive/YYYY-MM-DD-code-cleanup.md`:

- the step 1 numbers, before and after, with the delete/add ratio
- each commit and what it removed
- the tickets
- the operator's list with the answers
- the rejected list with its conditions

Before a release, the operator's spec pass over ARCHITECTURE.md and
README.md goes in the same file: cut restatement and rationale, reconcile
the guarantees table with the tests, close resolved open questions.
