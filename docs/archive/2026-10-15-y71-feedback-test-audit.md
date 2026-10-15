# Y-71 feedback namespace repair

Review1765 identified a collision between a declared candidate source named
`feedback` and the implementation execution's internal repair input. Admission
now reserves that name for internal repair on candidate-producing writers.
It rejects a declared source reference before creating an attempt or execution.
No source binding, publication, comparison or repair behavior is removed.

The existing `integration_retains_partial_work_and_approves_only_its_output`
scenario gains one refusal control for G2's integration input identity clause.
The operator submits the otherwise supported preset with A renamed to
`feedback`. The socket reply must name `nodes.integrate.inputs.feedback`
and `invalid_input`; read-only store queries must show no attempt or execution.
The same parked ticket then starts the supported preset. Its original held
restart, A/B source content, containment, gate, C approval and landing assertions
remain intact. The conflict scenario also retains its original assertions.

Sabotage: remove the reserved-name validation. The new assertion fails on the
admission reply before any integration can loop. Expected identity and reason
come from the binding spec and fixture request; expected empty store rows come
from the pre-admission contract. Existing scenarios never declared a source
with the internal repair name, so they cannot catch this collision. No new test
scenario or assertion twin is added. No assertion or binary line is removed.

Validation: `cargo fmt --check` and
`cargo clippy --all-targets --locked -- -D warnings`.
Host e2e execution remains the operator's landing gate and is not run here.
