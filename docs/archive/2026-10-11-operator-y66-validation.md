# Operator validation of Y-66, 2026-10-11

Y-66 introduces optional raw usage values and per-field provenance, with
fresh completed Codex normalization and truthful historical migration.
Pi, Claude and advanced Codex normalization remain in their own tickets.

## Candidate and source judgment

Attempt 68, ticket revision 3. Base:
`0e23cf7a9fb5ba57c175f422dcd5f00f7ba6bb82`.
Candidate: `fdb72b23988d2ad87309f3f492455e201d826ff8`.
Proof is the empty snapshot digest
`e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`.

The operator read the full 13-file, 745-line diff, including protected
architecture, both new G7 scenarios, the Responses fixture wrapper and
the historical-binary fixture visibility change. Source and test-audit
gates passed. No existing assertion was removed or weakened.

Fresh completed Codex totals retain numeric zero and unknown monetary cost.
Usage values stay in the nullable columns; per-field metadata is additive.
Store and trusted MCP read that same record. Migration labels old rows
legacy with unknown completeness without changing old values or nulls.
Pi and Claude retain prior numeric semantics labeled adapter and unknown.
Resumed completeness stays unknown pending the separate Codex work.
No billing estimate, complete resumed total, aggregate or cap is claimed.

The two new scenarios exercise different mechanisms. Real boxed Codex
against the host Responses fixture produces nonzero and explicit zero
terminal frames, checked against fixture values and host request counts.
Store and MCP expose null cost and literal metadata. The real old binary
writes usage and nullable non-worker rows; two new-binary reopenings keep
the externally observed values and label them legacy. No binary test flag,
mock, public model endpoint, sleep or retry was added.

## Exact candidate validation

Candidate checks 730-732, executions 1385, 1386 and 1388, passed fmt,
clippy and build on this exact head, proof and revision.

The isolated validation checkout was clean at the exact candidate.
`cargo test -p e2e --locked` ended zero with 58 passed, zero failed in
238.19 seconds. Both new usage scenarios and all inherited lifecycle,
capacity, freeze and migration scenarios passed. The complete log is
`/tmp/switchyard-y66-fdb-host.log`. Post-run Git inspection confirmed a
clean checkout at the same candidate. No host run is still active.

## Review and decision

Correctness review execution 1389, check 736, passed round 1 with two P3
findings. The old stats endpoint exposes bare values without usage metadata;
resumed totals still need normalization. The operator read both findings
and their source. The full raw record is available through attempt show.
Resume normalization and usage drilldowns remain the existing Y-77/Y-78
scope. Both findings stay carried into final audit, not described as fixed.
The below-blocking default applies; no speculative child was filed.

Protected attention 164 at seq 4820 named the exact head, proof, revision
and checks 730-732/736. The fresh full diff matched the read source. Approval
waited for the other isolated host suite to finish, avoiding host-suite overlap.
A first command mistyped the proof digest; Yard refused with expected/current
identities and changed no state. The next watch returned the exact payload.
Approval 79 used the head and proof from that read payload. The next command
rearmed the watch.

## Fresh landing and follow-up

Landing execution 1395 ran against merged head
`b22ab00d405ce5d6d904803e6750c9d8f67f0026`, with old target
`0c6d8ec83eb6376e76347d11a86c567274445cd7`. This includes the operator
partition record and is distinct from the reviewed candidate. Executions
1396-1398 passed fresh fmt, clippy and build. Host execution 1399 passed
58 scenarios, zero failed, in 259.48 seconds.

The supported audit stream recorded landing.recorded at seq 4836, ticket
done at 4837, attempt ended at 4838 and cleanup removed at 4840.
Supported sync consumed the actual merged head at seq 4844; the next
command was watch. Git inspection confirmed that exact clean checkout.
No push, installation, publication or frozen daemon replacement occurred.

The two published P3 findings were attached through supported ticket edits
to existing Y-77 and Y-78, both revision 4 and parked. Full byte readbacks
preserved their original scopes and dependencies. Y-77 carries execution
local resumed accounting; Y-78 carries provenance-aware drilldowns and
aggregates. Neither finding is claimed fixed by this landing.

Y-88's independent exact-head host suite passed 56 scenarios and its
correctness review passed. Its findings and protected approval remain the
next operator decision. Y-60 retains full integration scope through
Y-88/Y-89/Y-90. Y-81 stays parked. The release through Y-80 is incomplete.
