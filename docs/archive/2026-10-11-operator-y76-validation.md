# Operator validation of Y-76, 2026-10-11

Y-76 normalizes Claude API/login tokens and optional estimates. Both routes
use the same reader. Tokens are invocation-local; reported resumed cost is
session-cumulative and cannot be stored as an execution estimate. With no
observed invocation-start baseline, resumed local cost remains null/unknown.
Fresh finite nonnegative estimates, including zero, remain numeric harness
data. No token-price arithmetic or subscription bill is inferred.

## Initial source and assertion gate

Attempt72, initial revision3, base4faa8da6f0b0f24185c6d80ff8ccd5d8b535bbce.
The operator read the complete779line diff of4f55db7 and protected G7 row.
The source added assistant observations by message id, replacing repeated
content blocks. The daemon persists partial observations before settlement;
terminal token totals replace them. Failure and interruption preserve partial
usage; resumed token completeness is no longer forced unknown for Claude.
The existing execution lifecycle and daemon-only store writer are retained.

The assertion gate failed before any host run. API and login entrypoints
called the identical four-phase accounting scenario, differing only in auth
setup. Their mechanism and observation were twins. Supported revision4
preserved the full original scope and required one hardest common accounting
scenario plus independently necessary route evidence. Existing G8 API/login
authentication controls remain; no route-specific usage difference was invented.

Repair5a32f644047df52c1fa3357adf263a8fded2b49b removed the twin and boolean
dispatcher, retaining all accounting phases. Its full226line repair delta,
surviving assertions and dated audit were read. Source/test-audit gates passed;
checks759-761 passed. Old review1444 stopped on edit, not passed.

## Host failures and pinned frame evidence

Exact5a32 host65447 ended101:59passed/1failed/236.65s.
Log: /tmp/switchyard-y76-5a3-host.log. Interrupted execution3 recorded input1100,
output0, null cost and partial harness metadata; the test expected output100.
Independent API/login controls and inherited scenarios passed. The provider
fixture sends output0 in message_start and output100 in message_delta, but
Yard receives Claude stream-json snapshots. The failure alone did not prove
a parser defect. The clean exact head was verified after the run.

Supported revision5 preserved scope and required actual emitted-frame evidence.
Repair779c4e6860bf577dcf53e9ad92e97c9fa7a1b302 uses every raw output report for
its one provider response as the interrupted oracle. Source/test-audit passed.
Focused host40928 ended101:0passed/1failed/59filtered/27.28s.
Log: /tmp/switchyard-y76-779-focused-host.log. Captured raw interruption frames
show two assistant snapshots with the same id msg_fake_3, each input1000,
cache read70, cache creation30 and output0. Init and two user frames accompany
them; no result or later nonzero output report exists. The interrupted oracle
passed. The run reached final resumed completion, then failed solely because
serde_json Value equality distinguished raw integer0 from stored float0.0.
Tokens2200/200 and complete metadata agreed. Clean exact779 was verified.

Supported revision6 required numeric USD comparison without null defaulting.
Repairab95eeff16cff29be71e938ff49926a76ee5ab2f converts both observed and
external reported cost with as_f64, retaining numeric/null and metadata checks.
Its full test/archive delta passed the gate. Focused host72492 ended101:
0passed/1failed/59filtered/21.46s. Log: /tmp/switchyard-y76-ab95-focused-host.log.
Every earlier phase passed; raw resumed-estimate proportionality failed.

A separate diagnostic run added only test-side printing of unmodified raw
result summaries. It ended101/17.65s at the same cost assertion. Log:
/tmp/switchyard-y76-ab95-cost-diagnostic.log. The diagnostic patch was restored,
and clean exactab95 verified. It was evidence gathering, never a candidate pass.

| Execution | New input/output/read/write | Raw session USD | modelUsage cumulative input/output/read/write |
| --- | --- | --- | --- |
| 1 | 2000/200/140/60 | 0.009267 | 2000/200/140/60 |
| 2, failed resume | 1000/100/70/30 | 0.0139005 | 3000/300/210/90 |
| 4, successful resume | 2000/200/140/60 | 0.0231675 | 5000/500/350/150 |

Raw modelUsage.costUSD matches total_cost_usd. Result tokens are invocation-local;
result cost is session-cumulative. The initial reader's local-cost assumption
was false. The source and invalid proportionality oracle required correction.
No inferred prices, vendor charges or recovered interrupted bill were claimed.

## Cumulative cost and fixture sequencing repairs

Supported revision7 preserved the full contract and supplied these observations.
Repaira8fe459a3f811489df1d42e107e2790acd6c4f30 reads authoritative resume identity
from the execution row before launch. Claude withholds resumed session cost;
fresh observed amounts survive. Pi and Codex only receive the reader signature
argument; their normalization is unchanged. No new schema, owner, lifecycle,
worker grant or dependency is added.

The one scenario retains missing initial cost, failed resume, interrupted
multi-block response, reopening and resumed completion. Raw positive/increasing
session amounts establish the cumulative trigger, while execution rows require
null/unknown local cost. Independent fresh nonzero/zero controls establish that
valid local estimates survive. Tokens/cache totals and terminal replacement
remain externally counted; interruption uses unmodified actual harness frames.

Source/test-audit rejected the fixture sequencing before a host run ofa8fe.
It left Y-1 and Y-2 live awaiting approval in a two-lane fixture before creating
Y-3. Supported revision8 preserved all phases and required supported retirement
after observations. Repair16ede12326d43412340d3e9caa7008651a758bc0 parks each
observed ticket, abandons its attempt, then retires the ticket. History waits
for cleanup before the next control. Execution ids6 and8 include cleanup5/7;
zero transformation targets8. No capacity increase, assertion deletion, sleep,
retry, mock, public model endpoint or binary barrier was introduced.
The full delta, protected G7 row and65line audit were read. Gates passed.

## Exact repaired-head acceptance and first approval

Focused host82853 at clean exact16ede ended0:1passed/0failed/59filtered/30.71s.
Log: /tmp/switchyard-y76-16ed-focused-host.log. All resumed unknown-cost and
fresh nonzero/zero controls passed. Post-run clean exact head was verified.
Full host87351 ended0:60passed/0failed/262.01s.
Log: /tmp/switchyard-y76-16ed-host.log. Post-run clean exact16ede was verified.

Candidate checks776-778, executions1472/1473/1477, passed fmt/clippy/build.
Review1479/check779 passed round1 with no findings. Protected attention171
named revision8 and those checks. The fresh full diff matched the reviewed
base/head. Approval86 used the structured head/proof, then watch was rearmed.
Landing1486 conflicted only in docs/ARCHITECTURE.md; no landing was recorded.
The target had advanced through Y-89 and its validation record to16804764.

## Combined merge repair and fresh approval

Repair1487 produced4570883143316b4e9525e8a3590ef7e2db726d17 on actual base
16804764c0600a1aba0f6c8387cf0b8cdc640174. Exclusive inherited graph source,
assertions and archive blobs match that target exactly. Retained Claude,
harness, model and accounting-test blobs match the accepted16ede exactly.
Shared supervisor/store-execution deltas and the complete protected usage
contract/G7 row were read. Graph routing and Claude accounting remain intact;
no assertion was lost. Source/test-audit gates passed again.

Combined exact-head full host23421 ended0:61passed/0failed/260.85s.
Log: /tmp/switchyard-y76-457-host.log. Clean exact457 was verified afterwards.
Fresh boxed checks783-785, executions1489-1491, passed fmt/clippy/build.
Fresh review1492/check786 passed round1 with no findings. Protected attention172
atseq5153 names exact457/revision8 and these checks. Fresh full diff matched
the reviewed actualbase/head, ignoring only the CLI terminal newline.
Approval87 bound structured head/proof, then watch was rearmed. No isolated
host suite was active during fresh queue validation.

## Fresh recorded landing and checkout

Landing1493 completed. Fresh children1494 fmt,1495 clippy and1496 build passed.
Host1497 passed61scenarios with zero failures in295.97s on the actual merged
tree. The earlier isolated result did not replace this gate. No host suites
overlapped. Supported history records landing.recorded5171: candidate and
actual merged head both4570883143316b4e9525e8a3590ef7e2db726d17, oldtarget
16804764c0600a1aba0f6c8387cf0b8cdc640174. Ticketdone5172, attemptended5173 and
cleanup1498 removed5175 followed. The operator read actual gate outcomes and
these events before supported consume. The next command rearmed watch.
Post-consume Git inspection verified the clean exact landed head before this
operator-only record commit and supported import.

Y-90's passed review and host acceptance remain distinct from its pending
landing; its published coverage finding is mandatory in full parentY-60
integration. Y-77 is dependency-ready for the free lane. Y-81 stays parked.
No push, publication, install, frozen daemon replacement or model change.
Full release through Y-80 remains incomplete.
