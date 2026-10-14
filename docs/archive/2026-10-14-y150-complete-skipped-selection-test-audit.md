# Y150 complete skipped selection expectation repair

Retain all four required host failures and the earlier archive evidence.
Hoste6b413 failed 0/1 at g02:444 in 65.45 seconds suite time and 79.6344
seconds wall time. The reduced graph and first exact candidate failure
acceptance succeeded. Approval correctly exposed all three omitted nodes;
the fixture expectation still named only the candidate omission.

The exact approval payload assertion now names the concrete fixture's
candidate-gate-unknown, landing-gate-closed and landing-gate-unknown. Its
expected list is literal, independent of Yard output. The assertion remains
an exact equality and runs after both failed-candidate acceptances.

Reviewed every following selection list in this scenario. Candidate
accept_skipped stays limited to candidate-gate-unknown. Approval's
observe_landing_skipped, the prepared-merge identity's skipped list and
prepared-merge accept_skipped each name only landing-gate-closed and
landing-gate-unknown. Selected landing verification still requires exactly
landing-gate-other to pass on the host-Git prepared head. All fixture counts,
passing-reference observations, failed row comparisons, exact decisions,
queue checks and quiescent reopen checks remain unchanged.

Test-audit gate: retain the existing G2 complete-selection and immutable
reference/failure/omission integration assertion. Its sabotage comment names
manufacturing an omitted check; losing a declared omission also breaks this
exact assertion. G3 selected landing verification remains covered. This
second bookkeeping revision followed by queue and reopen differs from the
existing omission API cases by retaining an immutable passing reference
beside exact failed-candidate decisions. Expected node names come from the
fixture graph's declared omissions, not a projection of the observed list.
One stale expected line is replaced; zero assertion blocks or binary lines
are removed. No production or spec contract changes are needed.

Required worker validation is fmt, clippy and build. No host scenario or
full suite runs in this worker. The operator reruns the same required case;
later queue and reopen observations still require host validation.
