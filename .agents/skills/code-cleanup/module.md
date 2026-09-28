Read AGENTS.md and docs/ARCHITECTURE.md, then your files whole. Edit
nothing. Hunt for what the spec does not ask for, what the platform already
does, and what is said twice. Prefer fewer concepts and paths. A fallback,
option or recovery path needs a current spec requirement or a reproduced
bug; a hypothetical contingency alone is not enough. Keep the spec's
controls exact. Skip rejected findings unless their condition now holds.

One line per finding, no hedging:

`<file>:L<line>: <tag> <what to cut>. <replacement>. [spec: <line or none>] [-<N>]`

- `delete:` dead code; an option nothing sets; a fallback for a state the
  spec rules out; a flag, variable or path that exists only for tests; a
  comment restating the code.
- `yagni:` an abstraction for hypothetical reuse, a layer that only
  delegates, an unused compatibility path, or a feature's own
  lifecycle, owner, lease or recovery path beside the execution state
  machine. Inline it, or make it an execution.
  A single caller or implementation alone is not a defect.
- `stdlib:` hand-rolled code that `std`, `tokio`, `nix`, `hyper`, `serde`
  or `rusqlite` already provides. Name it.
- `native:` code doing what git, pinfold or SQLite already does. Name the
  feature.
- `dup:` the same logic in two places; two readings of one truth
  (readiness, the current check, approval, liveness, merge readiness); one
  thing the spec states twice. Name the survivor.
- `shrink:` the same behavior in fewer lines, including a special case that
  a general fix to the mechanism removes. Show the form.
- `spec:` a cut that touches a control (the threat model, G1 to G15, the
  box table, the worker-tool grants, the child-process rules), removes a
  feature or deletes a guarantee row. The operator decides these.

Example: `jobs/queue.rs:L210: dup: merge readiness computed here and in api.rs. Keep store/checks.rs::current. [spec: L402] [-31]`

A speedup counts only with a measurement behind it. Out of scope:
correctness bugs, test assertions, and log and help wording. A test's
positive control is never bloat.

Rank the biggest cut first. End with `net: -<N> lines, -<M> deps.` or,
when nothing is found, `Lean already.`
