# Y144 continuation prepared-ref correction

The correctness review found a G3 observer error after exact acceptance of
the selected pre-intent failure. The accepted continuation has its own
execution and intent, but reuses the original failed parent's prepared input.
It has no `refs/yard/landings/<continuation>` ref. The previous host Git lookup
would fail before open and historical intent controls could run.

`queue::land` validates and reuses the prepared predecessor's ref before
returning from its reuse branch. Only fresh preparation pins a new ref;
`executions::prepare` requires `resumed IS NULL`. Correct the test lookup to
the failed parent's ref, already independently observed before acceptance.
Keep the open-intent execution identity for all refusal and final accounting
assertions. No product source or assertion changes.

## Test-audit Gate

| Gate question | Answer |
| --- | --- |
| Guarantee and clause | G3's exact prepared merged input, accepted failure continuation and open/historical intent refusal with no writes. |
| Named faults | Existing selected-definition evidence and both independent intent-guard mutations remain required. Their comments and assertions are unchanged. |
| Distinct hardest case | A real selected failure is accepted before intent. Its continuation must reuse the pinned preparation while canonical movement later retires its intent. Correcting this observer lets the existing scenario reach those controls; no new scenario or twin. |
| External expected value | Host Git resolves the original prepared ref. The selected gate's direct HEAD observation and target-only input checks use that head; later store assertions compare the continuation's intent_merged against it. No expected head comes from yard output. |

Static Gate accepts the observer correction. Test lines removed: 4 lookup
lines replaced by one lookup line and an explanatory comment. Assertion
blocks removed: 0. Binary lines removed: 0. Exact child/check/revision,
selected committed Gate command, checked ownership release, acceptance and
immutable history remain unchanged. No new test, deadline or concurrency
change is made.

Earlier archives, gate2770/gate2780/gate2795 failed full native results and
the surviving original-landing-decision-evidence diagnostic remain preserved
and unqualified. Host e2e and named runtime faults remain post-approval gates;
they were not run in this box. Parent and packaged integration obligations
remain mandatory. Review1409 P3 stays carried into Y140 revision6.

`cargo fmt --check` and `cargo clippy --all-targets --locked -- -D warnings`
passed.
