# Command classifier landing omission repair

The operator's real 960d40a command case reached actual command classification
retry and durable exhaustion after the jq repair. It then failed at g02:593:
prepared landing identity named landing-gate-classifier, landing-gate-closed
and landing-gate-unknown, while the fixture expected only the latter two.
The classifier gate is declared but deliberately absent from the selected
normal landing chain. Its omission requires exact explicit authorization.

The failed run remains at
`/home/yard/validation/y73-960d40-integrated-command-20261015-evidence/native.log`
and `terminal.json`: 0/1, child 101, 79.90-second case, 86.31-second wall,
source restored, runtime unchanged. It is not a pass.

## Test-audit disposition

Adapt the existing G2/G3/G8
`candidate_reuses_original_pass_after_bookkeeping_with_unknown_rerun` in
`g02_judgments_bind_identity.rs`. Include landing-gate-classifier in all three
remaining landing sets: approval's observe_landing_skipped, the exact expected
prepared omission identity, and the operator's accept_skipped decision. The
candidate approval's whole-selection expectation already includes it and stays
unchanged. Candidate acceptance still names only candidate-gate-unknown; the
prepared landing decision names exactly its three landing omissions.

Inspect the entire remainder, including final landing and restart observations.
Its final closed-command count also retained nine, although the preceding
retained edit phase already observes ten. Change that expected final count to
ten. Landing runs only landing-gate-other; no omitted closed command runs or
removes an earlier log line. Other remains five and unknown remains five.
All selected merged verification, truthful failed checks, immutable historical
rows, authorized landing and restart observations remain intact.

This protects exact identity and unauthorized landing effects. Expected gate
names come from the fixture's declared configuration and selected graph;
prepared refs come from real Git, command counts from the host log and the
retained phase counts. Existing sabotage comments name false failed-result
reuse, stale acceptance and manufactured omitted-node results. The same
representative case and allowed acceptance/landing controls suffice. No guard
is dropped and no unknown omission is accepted. No fixture, test function,
machine, build, box, model turn or product change is added. Test lines: four
removed, six added. Binary lines added or removed: zero. Shown by is unchanged.

## Qualification

Candidate fmt, Clippy (all targets, locked, warnings denied), locked build and
whitespace checks pass. Host e2e remains operator-run. The known failing
fixture canceled the ongoing review; a current review remains an operator gate.
Production e3b7598 stack repairs, jq supply, both phase cuts and ec241fb's
fresh-session hold repair are retained. No repository .yard, installed runtime,
daemon, binaries, model, reviewer, observer or policy files change here.

The operator separately qualified ec241fb's agent-route case with canonical
observers: 1/1 passed in 35.52 seconds, retained at
`/home/yard/validation/y73-ec241fb-integrated-routes-20261015-evidence`.
That run is not repeated here or represented as a full product pass. All
original failed results remain retained.
