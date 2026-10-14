# Y-149 landing fixture repair

The operator's landing gate on 3b4cfb22 failed one of 79 scenarios. The
packaged audit fixture started the real 559f730 daemon after the candidate
had created schema 22 in the same store. That daemon supports schema 21.
The gate reported 78 passed and one failed in 524.90 seconds. Its failure
results remain evidence of the rejected candidate, not of this repair.

Each historical package admission now creates a fresh real machine and
project under its admitting binary. The same project then upgrades to the
candidate after checked stop and release. This applies to the six-field
controller, the compiled metadata variant and the b9e374 fallback. The
missing-asset refusal and corrected-image control remain in their original
shared project. No binary behavior, schema protection or reuse rule changes.

Focused test-audit gate:

- G8 retains the existing Shown-by clauses for the historical six-field
  controller and the metadata and exact-package capability paths. Their
  sabotage comments, held host requests, explicit revisions, publications
  and frozen-row comparisons remain. These historical paths expose failures
  that current-package controls cannot catch.
- G2's unknown historical evidence clause is checked by retaining every
  old row field and expecting only the added verification field to be null.
  Sabotage: backfill ended executions from current definitions. The expected
  null comes from the spec; the old fields come from the pre-upgrade store.
- The existing privacy observations now apply to each isolated project.
  Host scripted requests and read-only stores remain the observers. The
  credential literal and controller-only model protocol remain fixture
  expectations. No secret value is added to Yard-written artifacts.
- Other expected values still come from fixture settings, archived real
  source, host Git and boxed package bytes. No assertion or guarantee is
  removed. No new scenario, sleep, retry, mock or binary test seam is added.

The diff replaces 223 test lines, largely by indenting the six-field
upgrade block and moving the shared privacy assertions. Zero binary lines
are removed. No operator list or whole-suite sweep applies to this gate.

Worker validation passed: cargo fmt --check, cargo clippy --all-targets
--locked -- -D warnings, and cargo build --locked. Host e2e was not run in
the worker. Fresh review, the affected host scenario and a full landing gate
remain pending.
