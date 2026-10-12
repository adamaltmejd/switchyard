# Y-110 operator validation, 2026-10-13

Y-110 landed as a8da0ee848122ce200df88ec2be5758a6d20157f.
Candidate: 7e8a356c622679ef54afffb4379bb9427f02c9a0.
Candidate base: f0de85e5f772610520334e1f98101e87b88b3735.
Canonical parent: 513eca2fbf450af15e764b9b38c6037050e2954e.
Merged tree: 8a7f06f1f2c238c97ef4bd57c1287e363f22b9ff.
Proof: e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855.

## Source and decision

The CLI diff matched committed Git; its SHA256 was
c19fba29c9541625b28c46d97d297b3c9e5682339ada4927edc53521d8048c4b.
Operator inspection covered runtime changes, changed assertion blocks,
fixture isolation and the protected G9 shown-by change. Source guards route
pre-turn preparation failures through the existing acknowledged-stop path.
The latest repair covers proof preparation and packaged staging as two
cohesive result groups. Store and ownership operations remain outside them;
a successful box controller still passes through ordinary checked teardown.

Candidate fmt 2262/check1177, clippy 2263/check1178 and build
2264/check1179 passed. Correctness review 2265/check1180 passed with no
findings. Operator approval 129 names the exact head and proof.

## Prospective integrated-source controls

An isolated real merge with the canonical parent produced the same tree as
the actual landing. This included already landed Y-112's inherited-lock fix.
The established host profile and durable inputs were retained.

`cargo test -p e2e --locked preparation_keeps_ownership_and_prevents_the_turn -- --nocapture --test-threads=4`
passed both scenarios concurrently: 2/0, 77 filtered, 46.46 scenario seconds,
61.267 total seconds including compilation. Distinct fixture roots prevent
collision. Both controls completed their later ownership/release/restart
observations.

`cargo test -p e2e --locked a_packaged_audit_refuses_missing_assets_then_publishes_in_a_corrected_box -- --nocapture`
passed 1/0: 180.24 scenario seconds, 180.309 total seconds. This is the existing
real packaged-audit scenario with the scripted host model. No new scenario
or assertion was added for the operator investigation.

Changing only startup Err handling to propagate before acknowledged stop
made the failed-preparation scenario exit 101 at G4:1078: actual `red`,
expected `stopped`. It took 17.39 scenario seconds, 17.472 total seconds.
The mutation does not reach later ownership/release/startup/restart blocks;
the unchanged-production controls do. Source was restored exactly, build
passed and the prospective merge tree remained unchanged. The source SHA256
was 78b1c7070841f44746fea42051512716bc804209327f0814570b589e75a5c6ea.

These controls do not deterministically fault every filesystem staging call.
Static inspection identifies the routed result groups. Earlier failed and
stalled predecessors remain in their original dated records and evidence;
later passes do not erase them.

## Actual landing

Landing 2266 ran fmt 2267, clippy 2268 and build 2269 successfully.
Host e2e 2270 passed 79/0 in 620.51 scenario seconds. Its execution ran
from 2026-10-12T22:24:15.380Z to 22:34:50.917Z: 635.537 seconds including
compilation. The recorded output is a bounded tail, not a complete raw log.
The measured full gate still exceeds the historical five-minute budget.
No scenario was removed to meet that budget.

Audit sequence 7867 records the landing; 7868 marks the ticket done;
7869 ends the attempt. Cleanup 2271 ended removed. Operator sync consumed
the actual merged head. Private command receipts, logs, review and history:
/home/yard/validation/y110-7e8a35-review-20261013 and
/home/yard/validation/y110-7e8a35-integrated-20261013.

Review 2265 took 841.680 seconds, with 78 assistant message ends,
80 tool results and 87102 reported output tokens, including 77419 reasoning
tokens. Repeated cached input is context accounting. Elapsed time includes
tools and request latency; it is not a provider token-throughput measurement.

The frozen operational daemon and binary, reviewer and fast-tier implementer
configuration were preserved. Nothing was pushed, published or installed.
Y-111's whole-tree checkpoint, remaining redesign and final Y-80 validation
are still required; this landing is not release completion.
