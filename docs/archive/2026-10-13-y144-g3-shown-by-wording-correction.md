# Y144 G3 evidence wording correction

Review1413 P2 identified stale wording in G3's binding Shown-by row and
the matching scenario comment. Both now name `candidate:landing` failure
evidence. The selected `operator_failure: true` path raises kind `candidate`
and reason `landing`, as the architecture contract and corrected test specify.

Test-audit disposition: wording only. No assertion block, expected value,
product source, deadline or concurrency changes. Exact child/check/revision,
selected committed Gate command, checked ownership release, exact acceptance
and immutable history obligations remain. Test lines removed: 0 executable
lines; two comment lines replaced. Binary lines removed: 0. The prior Gate's
selected-definition mechanism, external fixture/Git oracles and named faults
remain unchanged; no new scenario or twin is added.

Earlier archives are unchanged. All three failed full native results
(gate2770, gate2780 and gate2795) and the surviving
`original-landing-decision-evidence` diagnostic remain preserved and
unqualified. Full native and named independent runtime gates remain
post-approval obligations. Host e2e was not run in this box. Review1409 P3
remains carried into Y140 revision6, outside this correction. Parent and
packaged integration obligations remain unchanged.

`cargo fmt --check` and `cargo clippy --all-targets --locked -- -D warnings`
passed.
