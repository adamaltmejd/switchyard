# Y-87 explicit fixture definition test gate

The operator reviewed candidate
`8a02a19815186e8c001b4d9e59c59805847de96a` and found that G10's
provenance definition expectations used undeclared numeric defaults.
Neither the spec nor README defines those numbers. Their origin was
`Config::parse`, which fails the outside-expected-value rule. No host run
is claimed for that candidate.

Disposition: correct the existing G6 definition assertion block in the
named-target conflict scenario. Its `configured()` fixture now declares
workflow access `write`, instructions `Implement the ticket.`, session
limit 7, inactivity limit 45 minutes and total-work limit 300 minutes.
It declares review timeout 35 minutes and gate timeouts 40 and 41 minutes,
respectively for `counter` and `image`. Both gates explicitly declare their
candidate stage, host or box placement, and empty environment-name lists.
Literal provenance expectations match those declarations. Existing shared
fixture declarations already supply the implementer, panel, review rounds,
blocking priority and agent selection.

This establishes G6's shown clause for fixture-defined resolved admission
definitions alongside host input identities and repair/restart persistence.
Sabotage: substitute parser defaults during provenance capture. Non-default
session limits, timeouts and instructions differ, so complete definition
equality fails. The test comment names the sabotage and fixture origin.
The earlier plan-inspection block does not observe these workflow or
timeout settings; no twin scenario was added. The existing conflict repair
and quiescent restart remain the hardest transitions for admission identity.

All original G10 behavioral assertions, full input equality, repair/restart
comparisons and historical migration assertions are retained. All five
trusted source paths remain literal expectations with independently resolved
fixture Git blob identities. Runtime capture, the commit/tree fix, current
template consumers and the provenance guarantee are unchanged.

Test assertion blocks removed: 0. Binary lines changed: 0. The changed test
lines replace fixture declarations and literal definition values only.

`cargo fmt --check` and
`cargo clippy --all-targets --locked -- -D warnings` passed. Host e2e was
not run here, as instructed. The operator will perform the complete host
validation on the final exact candidate.
