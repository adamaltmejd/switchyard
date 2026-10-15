Read AGENTS.md, then the `## Guarantees` section of docs/ARCHITECTURE.md
and the sections its rows cite. Read `crates/e2e/src/lib.rs` and
`model.rs` whole, then your files whole. Edit nothing.

Judge whole fixture cost as well as assertion validity. Recommend a compact
keep set and concrete scenario or phase deletions. Unique secondary options,
lifecycle combinations and historical variants may be deliberately untested.
Never propose replacing each deleted case with another test.

One line per assertion block that fails the bar:

`<file>:L<a>-<b>: <tag> <what>. <delete|rewrite>. [G<N>]`

- `surface:` unique coverage whose failure impact does not justify its fixture
  and maintenance cost. Name the deliberately untested behavior.
- `taut:` the expected value comes from yard's own output or a copy of its
  logic, and the guarantee is not that two outputs agree.
- `easy:` a weak case of the same mechanism. Name the harder one that
  subsumes it. Preserve positive controls.
- `unrelated:` a refusal that passes for another reason: a timeout, a box
  that never started, or a guard or reason token the row does not name.
- `twin:` the same mechanism and observation as another block. Name the
  survivor.
- `detector:` asserts argv shape, file layout, help text, log or reason
  prose, or a count the row does not name.
- `promise:` the name or comment claims more than the scenario exercises.
- `seam:` an observation through yard's internals or a test-only binary
  hook instead of a user seam. A helper's caller count is not a defect.
- `clock:` a sleep, a retry, or a deadline poll where `status --history
  --since` would do, or a scenario that waits out a timeout.
- `unheld:` a crash at a point that doesn't hold until the kill (seeing a
  row or a ref is not holding), or a restart before the state is proved
  from outside.
- `nocontrol:` a refusal with no allowed version succeeding in the same
  project.
- `nosabotage:` no sabotage in the comment, or one that would not fail this
  block. Say whether you doubt it.

Prefer deletion. Use rewrite when a retained core failure needs a better
external observation. Do not preserve every Shown by clause as mandatory
coverage. Name any high-impact core risk left without representative proof;
absence alone does not mandate a new test. Product controls stay exact.

End with the kept scenarios, deleted scenarios and real fixture/build/box
work removed. Do not infer runtime savings from line or scenario counts.
