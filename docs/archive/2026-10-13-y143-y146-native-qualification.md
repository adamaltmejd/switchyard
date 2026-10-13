# Y143 and Y146 native qualification

Y143 source `ac4c0b556c6e3c9c1363b3d5f9b7ce49827849eb` landed through
resolved execution 2739. Native gate 2743 passed 79 scenarios in 582.11 seconds.
Three independent faults were caught. Its delayed-startup fault reached only
a 300-second latch deadline. That result remains failed qualification evidence.
The fixture routed the initial request using the staged bytes being tested.

Y146 source `c702930a0d421cb34d21bd72ac29f09283fcd4c7` corrects the existing
fixture hold selection. It preserves every assertion and changes no product
code. Execution 2750 landed with a resolved intent. Native gate 2754 passed
79 scenarios in 1069.51 seconds. Its complete log matches the terminal tail.
An independently managed Pinfold suite overlapped that full run; its duration
is not an isolated-host benchmark.

Four source mutations ran independently against the exact Y146 source:

| Mutation | Direct behavioral failure |
| --- | --- |
| Current attempt at delayed old startup | G9:291 old opening used adopted-model instead of fake-model |
| Original attempt implementation settings | G9:330 successor used fake-model instead of adopted-model |
| Omitted consumed-definition map | G9:82 source-only operator diff lacked selected host blob/model identities |
| Suppressed handoff for equal plans | G9:82 actual equal-plan executions lacked the source-only diff |

All four returned native exit 101 at those assertions. No timeout, compile or
infrastructure failure counts as fault proof. Rebuilt restored source passed
one scenario. Each run restored exact source and preserved the runtime artifact.
Focused probes refused to launch while foreign native suites were active;
those guard refusals are not tests. Each actual focused run passed the global
exclusive-host guard at startup.

Full logs, commands, copied binaries, source diffs, hashes and operator verdicts:
`/home/yard/validation/y146-c70293-host-controls-20261013-evidence/qualified-manifest.json`.
Original failed evidence remains under the corresponding Y143 evidence directory.
Runtime was preserved Pinfold 0.2.2 with the authorized isolated FUSE profile.
Pinfold 0.2.3 Linux CI qualification is separate and does not cover these sources.

Y144 and full ordinary and packaged parent integration remain mandatory.
No Mac result, original Pinfold89 resolution or full Y80 completion is claimed.
The operational binary, daemon, reviewer and unrelated user files are preserved.
