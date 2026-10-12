# Y-105 target merge and scoped test-audit

Merge target 7a4c95a582f8db79224ca321ef4a33cb6e78f0e7 into candidate
6e2e706084d3f5d01bcc27b9063f3c1b92109169. The only textual conflict is
ARCHITECTURE.md's G8 evidence row. Retain the target's current packaged-audit
clauses and the candidate's exact selected merged-failure decision clause
and observers. No guarantee or evidence clause is removed.

The overlapping admission, plan and operator-schema changes merge without
manual source edits. Source review confirms that selected ordinary landing
nodes still permit operator_failure, landing answers retain the exact input
and canonical-ref rechecks and shared predecessor proof, and trusted
accept/repair schemas retain all landing bindings. Packaged audit admission
retains its separate supported single-node shape on the same execution
lifecycle. No worker grant, teardown requirement or final CAS is changed.

## Scoped test-audit disposition

The imported g08_durable_package.rs is byte-identical to the target. Its
G8/G12/G13 selected instruction, asset integrity, publication, secret and
isolation blocks retain the target's audit records and literal external
fixture oracles. The candidate's G1 selection, G3 prepared continuation and
G8 failed-result/error-retry test files are byte-identical to the pre-merge
candidate. Their existing audit records, sabotage comments, external Git and
store observers and positive controls are retained. Packaged instruction
selection and merged failure decisions exercise different mechanisms and
observations. No scenario or assertion is added, deleted or weakened by the
conflict resolution. Resolution-only test lines removed: 0; binary lines
removed: 0. This is a scoped merge review, not a whole-suite sweep.

## Validation limits

cargo fmt --check, cargo clippy --all-targets --locked -- -D warnings and
git diff --check passed on the merged tree. The worker ran no host e2e or
sabotage. Target Y-101's recorded host pass applies to its own merged head.
It does not validate this combined candidate. Y-105 host1969 remains failed
with 72 passed, 3 failed in 922.03 seconds. Its storage diagnosis and two
unproven adjacent failure causes remain as recorded. Fresh exact merged-head
operator host validation and the original sabotage scope remain required.
Y-104, Y-96 and Y-61 retain their full incomplete contracts. No .yard file
or construction runtime/state is changed.
