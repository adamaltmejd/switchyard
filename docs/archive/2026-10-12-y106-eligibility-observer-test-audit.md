# Y-106 eligibility observer repair

The operator ran the unchanged focused G2 control on exact candidate
`ccaf4e8d955619ef6f13cb2057e90295ffd64215` on the real host without
an overlapping host test. Cargo exited 101. One scenario failed in
43.25 seconds at `e2e/src/lib.rs:509`: the execution SQL observation
returned `SqlInputError`, no such column `result_eligible`.

The run passed the repaired history-reader restart path, fresh exact
acceptance and actual landing. It did not reach the remaining eligible
gate assertion, post-landing snapshots and cleanup reopen, ordinary
reduced/empty controls, or explicit-empty pre-mutation decision and
its restart/landing observations. No full scenario pass or sabotage
is established by that run. Earlier failed results and live evidence
remain independently recorded; a later pass cannot erase them.

`result_eligible` is a computed property of supported serialized execution
output, not a SQLite column. The repaired SQL assertion reads the real
stored node, head and outcome. The same fresh landing gate is selected
from `attempt show` executions by gate kind, landing reason and host-Git
merged head. Its serialized node, head, passing outcome and eligibility
must match fixture `landing-gate-keep`, that exact head, `pass` and `true`.
No database column or eligibility expression was introduced.

Scoped test-audit disposition: repair the observer at its existing G2
assertion. G2's exact fresh gate identity, G3's required selected verification
and G8's eligible passing judgment retain their positive evidence. Expected
eligibility is supplied by the spec; the node comes from the fixture and
the head from host Git. The assertion observes the user's store and CLI
seams, not a copied eligibility algorithm. The added comment names sabotage:
land with an ineligible retained gate result. This repairs existing coverage
without creating a twin scenario or weakening any assertion.

All held-child, target-advance, crash-state, stale/no-write, immutable
decision, predecessor-release, omission, restart and empty controls remain.
Y-105's failed-result and infrastructure-error controls and all production
ownership, identity and CAS validation are unchanged.

Replaced two test lines and removed zero binary lines. No shared helper,
schema, runtime, deadline, concurrency, sleep or retry changed. Worker
checks are fmt and clippy; host e2e was not run here. The operator must
rerun the exact repaired control and meaningful sabotage. Full Y-106
and Y-104/Y-96 integration scope remains required.
