# Y-60 ordinary graph integration

This is the complete parent integration pass after the actual prerequisite
landings. It covers the original contract, including the carried landing-child
infrastructure evidence gap. It does not replace exact-head host validation,
fresh published review or fresh landing gates.

## Actual landed inputs

The candidate starts at `38ebcdb`, the operator's Y-90 validation record.
The following heads are ancestors, with actual landing evidence in the named
2026-10-11 operator validation records:

| Ticket | Actual landed head | Operator runtime evidence |
| --- | --- | --- |
| Y-59 | `6bc59c0574092f67946aef3bfc43ee5f19161441` | Landing1374, host1378: 56/0, 243.90s |
| Y-88 | `39dd783fa42bae8df57e055815ee9a74d2cb4605` | Landing1412, host1416: 58/0, 253.44s |
| Y-89 | `725aed8e7cbd650709e95b59f974631e4bffe575` | Landing1474, host1480: 60/0, 303.38s |
| Y-90 | `0a2acdcf324d5b266a42ed99d0473555a190872a` | Landing1532, host1536: 62/0, 285.29s |

These are recorded landings, not earlier approved or conflicted candidates.
Y-90's exact combined-head host62409 also passed 62/0 in 293.25s and
review1531/check806 passed without findings. Those results cover inherited
scenarios on the old head. They do not validate this candidate's new assertions.

## Contract and scenario mapping

The following source conclusions come from the landed implementation. Runtime
claims below are inherited from the operator records above; no e2e ran here.

| Original clause | Source and inherited scenario |
| --- | --- |
| Graph eligibility schedules the existing lifecycle | jobs/mod.rs advance follows admitted complete and judgment routes; queue.rs land follows landing routes. Both use executions::start/end and capacity::effect. G7 ordinary_candidate_routes_repair_then_reach_approval selects command order different from configuration in both phases, crosses candidate failure and panel block, then lands. |
| Exact admitted plan, node, inputs and results | load_attempt re-resolves selected admission against its immutable source. executions::binding records plan revision, node, producer or external supplier and exact values before effects. G7 a_ticket_lands_end_to_end_and_leaves_only_rows compares CLI/store bindings and Git-derived candidate/merged parents; the routed G7 scenario observes predecessor panel and repair feedback agreement. |
| Continuation is not a gate object | supervise::start_implementation records implementation start reasons and ticket body/revision; pending edits and deliberate starts retain their existing precedence. G9 an_edit_reaches_the_next_prompt_and_stop_delivers_it_sooner exercises edits, stop and restart. No continuation execution kind or lifecycle was added. |
| Publication differs from end; no successor before checked teardown | review::publish records a check while review::run ends separately. RESULT_ELIGIBLE requires ended/unreserved/checked teardown and a produced result; reviews require registration. G8 a_seat_killed_after_publishing_has_published holds a published pass, permits another ticket to advance, observes no successor, kills/restarts and binds one interrupted result to the same execution and inputs. |
| Held ambiguous repair edge reconciles without duplication | G8 no_repair_starts_until_a_blocking_seats_box_is_gone observes the held box and running row outside Yard before restart. Existing G5 reconciliation removes ownership, preserves one review result and starts exactly one repair. No new crash barrier. |
| Declared failure differs from missing publication/infrastructure | executions::end separates infrastructure from judgment and emits no output without a judgment or candidate/report. G7 routed repair and G8 a_seat_that_always_blocks_gets_max_rounds establish declared judgments. G8 a_seat_that_exits_without_publishing_is_a_review_error and the original candidate portion of a_gate_errors_start_reruns_that_gate establish null output/no repair and same-project successful retry. |
| Landing-child infrastructure error | queue::land retains approval, ends error and raises landing attention without repair; supervise::gate applies truthful classification to children too. New assertions extend a_gate_errors_start_reruns_that_gate with real selected landing box-up refusal and supported same-approval retry. This new runtime evidence is UNRUN. |
| Bounded back edges create new executions, preserving history | advance permits one back edge per eligible failing result and review::blocked enforces the admitted repeat bound plus operator extra rounds. G7 routed repair retains failed and blocked predecessors; G8 always-blocking scenario reaches the limit, grants another round by edit and preserves exact override approval. |
| Default parity and deletion of superseded scheduling | plan::workflow_plan supplies ordinary routes; ordinary_linear admits all ordinary nodes exactly once, fixed panel order and unchanged repair bounds. G7 default round trip and selected routed round trip cover both. Source inspection finds one candidate/review route walk and one queue landing route walk; configured candidate/panel scheduling and configuration-order landing loops are gone. Historical translation in Plan::admitted is in memory and uses those same walks. |

## Safeguards and carried findings

This pass changes no execution control. runnable still blocks running or
reserved predecessors, current approval and unresolved attention. Capacity
uses one project limit and ownership proof. Queue admission still rereads
approval and live policy under target serialization; target CAS, durable
intent and recovery remain intact. load_attempt retains frozen definitions,
and egress intersects admitted and live policy. Merge and cleanup remain
ordinary non-node executions.

Inherited G2 identity and policy scenarios, G5 crash/intent scenarios, G6
selected admission, G10 target/capacity scenarios, G15 legacy upgrade and
G1/G8/G14 worker-grant, harness and containment controls remain unchanged.
In particular, the feature-target repair scenario freezes templates, gates,
review instructions, Dockerfile and trusted inputs across repair/restart.
These are inherited host results, not new runs in this box.

Review1394/check737 and repaired review1407/check745 carry one lifecycle
wording mechanism. checks::current actually requires eligible checked-teardown
results for node-bound gates AND reviews, and ended/unreserved executions
for historical checks. Exact identities and review registration still apply.
Historical node/input/result/teardown evidence stays unknown. The source
comment and binding architecture now state those conditions precisely.
No safety exception was introduced for ambiguous prose. The earlier alleged
immutable serde_json read panic remains unsupported: binding uses checked
node lookup, and missing/non-object immutable JSON reads return Null.

Review1488/check787's landing coverage finding is addressed by a focused
extension, not another full matrix. The fixture re-arms its external pinfold
wrapper after candidate retry and a target advance, before approval. The first
landing child refuses box-up. Store observations require error, null judgment
and route, empty outputs, retained approval, no repair and unchanged canonical.
Git supplies the failed landing's exact merge and parents. After supported
start, fresh real gates land a fresh exact merge of the same parents under
the same approval. Commit timestamps may differ; no old merged OID is reused
as an expectation for the new merge. Existing candidate and abandonment
observations remain; missing publication and both held-review cases are intact.

Restoring the old parent exemption in the Ok(GateResult) branch only changes
non-completed result classification, currently timeout. Box-up refusal uses
the Err branch and would not detect that particular sabotage. No clock test
is introduced. The timeout classification remains SOURCE-ONLY evidence;
the test names a sabotage of the exercised landing error branch or queue
repair routing. The new landing-child case and sabotage have NOT RUN here.

## Candidate validation and remaining operator gate

Cargo fmt --check and cargo clippy --all-targets --locked -- -D warnings
both passed before committing. Clippy compiled all targets, including the
extended e2e scenario. This verifies compilation, not runtime behavior. The
operator must run the extended G8 scenario and full suite on this exact head,
read a fresh published review and validate the actual merged landing input.
The inherited passes do not clear check787's runtime evidence gap by themselves.
No construction backlog was opened by a development binary. No .yard file
changed. Fan-out, result reuse, classifier models and later redesign tickets
remain outside this implementation; Y-81 remains deferred.
