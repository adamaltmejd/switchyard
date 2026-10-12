# Y-95 operator landing validation, 2026-10-12

Y-95 is done. The operator read the candidate, protected architecture
changes, review and exact-head focused evidence before approval 104.

Candidate: `33a3eaa38375a8c523e09006ac22256e309e85a3`.
Target before landing: `a88a328b15ab133266dcb0df1e898695d73005ec`.
Actual merged head: `14f0dcb2d358c589737fd5aab6ffd785e2cf4f77`.
Attempt 87 ended with outcome `landed`. Landing execution 1866 resolved
its canonical intent. The supported attempt reading reports the ticket done.

The actual merged-tree gates passed: fmt 1867, clippy 1868, build 1869,
and host e2e 1870. The full host suite reports 72 passed, zero failed,
in 555.70 seconds. Cleanup execution 1876 ended `removed`.
This is about 9.3 minutes, above the historical five-minute landing budget.
No test was removed, retried or weakened to meet that budget.
The operator consumed the actual landing with `yard sync`.

Earlier failed host gates 1838 and 1852 remain failures. The G6 observation
was repaired to wait for execution settlement before snapshot comparison.
Gate 1852 reported a missing pre-target image before the legacy first model
hold. Its disappearance remains unexplained. The diagnostic candidate adds
bounded failure evidence; its focused successful control did not exercise
that failure path. This green merged-tree run includes Y-99's landed changes
and does not establish the cause of the earlier missing image.

Host validation used the existing isolated FUSE profile and real boxes.
It does not establish native-overlay, Linux arm64 or macOS validation.
Pinfold issue https://github.com/adamaltmejd/pinfold/issues/89 reports the
separate native-overlay contention observations and asks about avoiding FUSE.
FUSE is not a new permanent Yard requirement.

The frozen operational daemon stayed running. Its binary SHA-256 remains
`05ae25832964703c81525e7c5f5be7a8148954517cfddf69ef9846953bf1ebb2`.
The implementer, reviewer and runtime configuration were not replaced.
Y-96 may now begin. The full release scope through Y-80 remains unfinished;
Y-81 stays parked. No push, installation or daemon switchover is recorded.
