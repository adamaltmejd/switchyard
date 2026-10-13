# Y142 native qualification

Y142 landed candidate `4d962e04559810b8f2afbcf3420aec03d26fea0d` as
merged source `294f8f93b25c0756580d88cc5b8ea2c2a03c9baf`. Landing execution
2723 ended landed with a resolved intent. Native gate 2727 passed all 79
scenarios in 555.00 seconds. Its full 15502-byte log matches the terminal tail.
Raw SHA256: 32f1ecf2d4acadf49042845c3b69994d7812cd2001c3b609374effdd05556396.

Five independent source mutations ran sequentially on the exact candidate
in an isolated checkout. Its product code, G2 scenario, build pins and
runner assets match the actual merged source. Each ran the real binary,
daemon, project, Pi and boxes against the scripted host model. No assertion,
deadline or concurrency changed. Source was restored after every mutation.

| Mutation | Direct behavioral failure |
|---|---|
| Frozen resolver overwrites adopted definitions | G2:503 host gate counter omitted the adopted invocation |
| Stale tool identity | G2:596 omitted required fresh consumers |
| Stale skill identity | Independent skill-only filter failed G2:596 |
| Whole-panel invalidation | Host fixture G2:72 caught an extra unchanged-model review after gate-only adoption |
| Completed-producer staging exception | G2:596 replayed only two reviews instead of the implementer, both gates and both reviews |

All mutations returned native exit 101 at those assertions. The whole-panel
fixture assertion caused later fake-model 500 and red attention; those
consequences are not separate evidence. No timeout or compile failure counts
as falsification. The rebuilt restored G2 control passed in 50.50 seconds.
The isolated source remains clean, and the runtime artifact stayed unchanged.

Full logs, binaries, source diffs, commands, operator verdicts and hashes:
`/home/yard/validation/y142-4d962e-host-controls-20261013-evidence/qualified-manifest.json`.

Runtime was preserved Pinfold 0.2.2, SHA256
7d1a82ad1496fc855ff19c706f3a41e83419c7ba500bd6288ba0d63c0747a27f,
with the existing authorized isolated FUSE profile. The separately qualified
Pinfold 0.2.3 dependency CI matrix is distinct evidence. No installation occurred.

Original failed native results 2691 and 2717 remain retained. The rejected
producer exception remains rejected. Only bounded ordinary quiescent
definition/staging adoption is qualified here. Y143, Y144 and full ordinary
and packaged parent integration remain mandatory. No Mac result or original
Pinfold89 resolution is claimed. The operational Yard binary, daemon, reviewer
and unrelated user files are preserved.
