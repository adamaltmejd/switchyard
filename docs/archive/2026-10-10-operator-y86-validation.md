# Operator validation of Y-86, 2026-10-10

Y-86 admits selected presets and one-off plans through trusted MCP.
The original Y-59 scope remains open for its final integration pass.
Y-60 and Y-66 remain dependent on Y-59. Y-81 stays parked.

## Candidate and source judgment

Attempt 65, ticket revision 1. Base:
`ac41350f5f389aabc7ed9253e5e4a62d666948c3`.
Candidate: `95924b770f4481c114e5471d8abf4d591986ee2b`.
Proof: the empty snapshot digest
`e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`.

The operator read the full six-file diff, including the protected
architecture clauses and all changed assertions. A fresh supported diff
readback matched the previously read candidate byte for byte.
The source and test-audit gates passed.

Admission calls the shared typed resolver inside the existing transaction,
before attempt and execution rows or lane reservation. Unsupported valid
shapes name their input and shape and create no work. Supported selection
matches an existing workflow chain exactly and consumes its actual frozen
definitions. The attempt records that matching workflow, resolved plan,
selection evidence and revision. The Y-85 frozen-source validation checks
that actual chain. Selection cannot replace a live plan or a stopped retry.
No worker tool grants or general graph runtime are introduced.

The new G6/G2 scenario exercises real operator MCP, daemon and boxed Pi.
An unsupported repeat leaves externally observed work rows and host model
counts unchanged; a supported preset plus override succeeds in the same
project. Literal fixture values establish the selected plan. A held model
request permits deterministic live-selection refusal and template sync;
the admission stays unchanged. Future preset and one-off controls consume
new definitions. Existing boundary and frozen repair/restart scenarios are
reused. No assertions were removed, and no sleeps, retries, mocks or binary
barriers were added.

## Exact candidate validation

Candidate checks 718/719/720, executions 1354/1355/1356, passed fmt,
clippy and build on this exact head and proof.

The isolated host checkout at
`/home/yard/validation/switchyard-y84-20261010` was clean at the candidate.
`cargo test -p e2e --locked` completed with 56 passed, zero failed in
218.06 seconds. The completed log is
`/tmp/switchyard-y86-959-host.log`. The session has ended; no rerun was
started after its observation handle disappeared. A subsequent Git read
confirmed the checkout stayed clean at the exact candidate.

The reviewer also attempted an optional compile. Registry access returned
CONNECT 403 and cargo exited 101 after 131.922 seconds. The wrapper process
exit zero does not establish a passing compile. This is distinct from the
required passing checks and host suite above.

## Review and decision

Correctness review execution 1357, check 721, passed round 1 with one P3
finding. It notes that graph chain equality omits workflow lifecycle limits,
so matching native workflows can have different session or time limits.
The operator read the full finding and matching source. The matcher prefers
the explicitly selected workflow, or default, when it matches; only its
fallback searches workflow names in order. Thus the example's sorted-first
claim does not apply when default matches. The actual chosen native workflow
and lifecycle definitions are persisted and consumed.

The finding remains open for final audit. It was landed under the skill's
below-blocking default, not described as fixed. No identity, containment or
ownership guard was weakened. No speculative follow-up was filed.

Protected approval attention 157, seq 4680, named the exact head, proof,
revision 1 and checks 718-721. A fresh full diff matched the read source
byte for byte, including the protected architecture. Operator approval 77
binds that candidate. The next command rearmed the watch.

## Landing and consumption

Landing execution 1358 ran fresh fmt, clippy, build and host e2e children
1359-1362. All passed. The host suite passed all 56 scenarios with zero
failures in 247.40 seconds. The supported attempt inspection preserves the
actual completed gate output in
`/tmp/switchyard-y86-landed-show.json`.

The supported history stream records landing.recorded at seq 4694, exact
candidate and merged head above, old target equal to the stated base.
Ticket done is seq 4695, attempt ended seq 4696. Cleanup execution 1363
ended removed at seq 4698. The primary watch returned empty attention.
Supported yard sync consumed the landed head; checkout Git was clean at
that exact commit. The next watch returned empty attention at seq 4699.
The saved audit stream is `/tmp/switchyard-y86-landing-history.jsonl`.

The operational CLI and daemon executable still have SHA-256
`05ae25832964703c81525e7c5f5be7a8148954517cfddf69ef9846953bf1ebb2`.
Sol 6.1 medium, fast service tier and the current DeepSeek reviewer are
unchanged. No push, publication, installation, daemon replacement or
reviewer substitution was performed.

Y-59's complete integration and acceptance execution is next. Its original
scope and downstream Y-60/Y-66 dependencies remain intact. This record
establishes Y-86's landing, not completion of the redesign through Y-80.
