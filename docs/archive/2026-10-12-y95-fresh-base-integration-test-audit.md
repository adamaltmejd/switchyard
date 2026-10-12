# Y-95 fresh-base integration and test-audit

## Inputs and boundary

This attempt starts at canonical
`52ad81116d3442c46fe54054d2a4b49ffbc1d066`. Its tree is clean before
integration. Git ancestry confirms actually landed Y-97
`0c5ae61d64ce56c959e534ee0b5e99f4ce2a6ea6`, Y-98
`db7b965c07a6b7c6587092d17439ce176e4bc02d` and Y-92 merge
`9c106745a01ddc443c937d1a543b8e45fb017ef2` are ancestors.
No old clone, remote fetch or missing target import is used.

The complete operator ticket and public patches were read. The binding
acceptance, identity, threat and G2/G5/G8 controls, current admission,
eligible-check reader, persisted decision reader and queue holds were read.
The Y-97, Y-98 and Y-92 operator validation records and isolated-FUSE host
record were read. Runtime statements below are inherited operator evidence;
no real host CLI reproduction, host e2e or sabotage is run in this box.

The carried Rust blobs match the reviewed patch exactly: G2 `a0d983d`
and admission `229a43f`. The three supplied historical archives are added
verbatim, with Git blob prefixes `9ba2a98`, `85e86b5` and `b0da48a`.
Their dated validation statements belong to those historical attempts.
No pre-existing archive is edited. This record describes the new attempt.

## Complete acceptance contract

| Original clause | Actual inherited observation and current source | Evidence boundary |
| --- | --- | --- |
| Full default repair and successful review-limit acceptance | Y-97 default failed gate repairs and always-blocking seat reaches limit; operator approval lands in the same project. Y-98 full default landing precedes selected omissions. | Exact Y-97 focused G8 and full69/actual69; exact Y-98 focused/full70/actual70 passed. Fresh combined head is unrun. |
| Selected failure/block acceptance and deterministic continuation | Admitted retained nodes select operator failure handling. Advance raises attention on eligible checks, reads its resolution, leaves the result untouched and visits remaining selected nodes. Y-97 accepts gate and blocking seat, including accumulated final block. | Landed runtime and current source, not a new execution here. |
| Actor, nonempty text and all exact identities | Candidate attention binds target, base/head/proof, ticket revision, admitted plan revision and full node/check/execution/result evidence. Resolved attention is the sole failure-decision reader. Explicit skipped decisions bind admitted omissions in the ordinary approval transaction. | Y-97/Y-98 store observers; proof-bearing extension inherited from historical 0175 focused run. Fresh head unrun. |
| Failed/blocking verdicts and results survive acceptance, landing and restart | Y-97 store snapshots stay equal after acceptance, actual canonical landing and quiescent reopen; held final-seat restart retains earlier block. Approval audit names accepted evidence. | Passed exact 0c5a scenarios. No rewrite or new lifecycle in this delta. |
| Changed candidate, ticket, plan or selected evidence refuses without a row | Held edit exposes revision then head staleness. Wrong plan revision and preceding check refuse by named identity. Skipped decisions also refuse old attention and retained node. Queue rereads selected identities. | Expected/current identities and unchanged audit/approval observations are retained. Wrong named plan is tested; replacement of immutable admitted plan is not claimed. |
| Required proof omission/mismatch and valid same-project control | Real worker writes literal evidence; host reads snapshot. Missing proof names expected empty/current digest, mismatched proof names both values, malformed numeric proof stays invalid. Correct proof accepts; retained checks/decision survive queue landing/reopen. | Same carried scenario passed on historical 0175, focused13247 1/0/29.44s and full50736 70/0/406.68s. Fresh head unrun. |
| Selected skipped results remain truthful | Y-98 retained gate completes first; accepted omissions create no execution, result or passing check; empty selected panel stays unreviewed through landing/reopen. This scenario now also retains worker proof. | Landed db7 observations and historical 0175 extension. All original assertions remain. |
| Infrastructure and held publication cannot be accepted | Y-97 held publication exposes no eligible result/acceptance item; delayed acceptance writes nothing and release gives completed control. Gate infrastructure error refuses; same-head real retry passes and lands. | Inherited focused/full-suite controls. Eligibility still requires ended, unreserved, checked teardown and result output. |
| Live protected authority and trusted operator capability | Queue holds rereads live protected policy and manual approval policy, selected checks and candidate bindings under target serialization. Worker grants and HTTP session containment are unchanged. | Inherited full-suite G1/G2/G8/G13/G14 observations; no new acceptance-policy race run is claimed. |
| Single writer, shared capacity, exact ownership and containment | Only existing project transactions resolve attention or persist approval. Current Y-92 ownership/publication/normal-judgment/candidate/cleanup/unstarted-child controls remain. Native Pi package/pins, registration and exact lock/teardown remain unchanged. | Actual Y-92 merged host1808 passed72/72 in455.63s. Fresh combined head unrun. |
| Target serialization/CAS and intent recovery | Acceptance uses ordinary queue holds and selected-check rereads. Landing failures and landing omissions have no acceptance exit. | Inherited G3/G5/G10 host observations; no landing exception added. |

Y-97 actual landing1671/host1675 recorded5795 and cleanup1676 removed5799.
Y-98 actual landing1715/host1720 recorded5963 and cleanup1725 removed5967.
Y-92 actual landing1804/host1808 recorded6243, ticket.done6244 and
cleanup1809 removed6247. The current base already includes all three.
Approval or a documentation record alone is never completion.

Historical 0175 review1731/check911 passed, but approval98 did not land.
Actual landing1736/host1740 failed68/2/447.74s. Fixture-init listing timed
out before publication-restart assertions; Codex linked-skill observed
ownership refusal rather than the expected path refusal. These remain
failures. Pressure and later healthy probes do not establish their cause.

The current operator-owned host command selects the isolated FUSE profile
under the CT2015 backup hold. Y-92's operator record reports package/full
validation and actual merged-host success using that input. This is a changed,
validated runtime input, not proof of the earlier native timeout cause or
permission for an unchanged native retry. No .yard change is made here.
Fresh host evidence must use the operator's authorized current command.

## Bounded source and spec integration

The only production delta is the reviewed absent-proof branch in
`skipped_decision`. Missing required proof compares as empty named digest
and returns stale before writing. Optional empty proof behavior and supplied
malformed-type invalid refusal remain. The existing transaction, current
attempt guard, proof requirement and result eligibility remain authoritative.
No ownership or infrastructure repair is inferred from this proof change.

ARCHITECTURE is the current 52ad text plus four additions: threat-model
omission refusal; selected-policy expected empty/current digest and malformed
invalid; G2 omitted/mismatched required proof; G8 retained worker-written
proof bound to checks and decisions. The supplied patch's truncated word
`decisio` is completed as `decisions`. All other text, including whole G5
and every Y-92 G8 clause, remains byte-identical to the base.

## Test-audit Gate

Disposition: keep and extend G2's existing selected-skipped identity scenario;
G8 also names retained proof evidence. No scenario or assertion is removed.
This is the change Gate, not the release's whole-tree sweep.

1. G2's Shown by names omitted/mismatched required proof with exact identities
   and no row. G8 names the worker snapshot retained by selected checks and
   explicit decisions through landing/reopen.
2. The test comment names sabotage: classify omitted required proof as invalid
   or ignore proof binding. Existing comments retain false-result, plan,
   revision/head, panel and persisted-decision sabotage.
3. Ordinary proof approval does not enter `skipped_decision`. Proof-less
   omission coverage cannot expose this failure. Extend the held edit that
   changes revision before head, with the valid same-project decision and
   real queue landing/reopen; add no twin or new crash boundary.
4. Evidence bytes, invalid inputs, node names and revisions are fixture
   literals. Host git supplies heads and canonical ancestry. Snapshot bytes
   establish real proof independently of decision output; digest agreement
   spans host snapshot, retained execution/check and persisted decision.
   No-write and immutability use external store snapshots, as the guarantee
   requires. Successful current decision/landing controls the refusals.

Test lines removed: 3, replaced by proof-aware calls/observations.
Binary lines removed: 5, replaced by the narrow comparison branch.
No sleeps, retries, synthetic rows, private state writes, test-only hooks,
new barriers, deadline changes, concurrency masking or process kills occur.
Existing quiescent restart follows cleanup and reservation release.

## Validation and remaining gates

Fresh boxed `cargo fmt --check` passed. Fresh
`cargo clippy --all-targets --locked -- -D warnings` passed in 17.46 seconds,
compiling yard and the extended e2e targets. `git diff --check` passed.
Git blob checks confirm the carried sources and archives match the supplied
patch. Reversing only the four proof additions restores ARCHITECTURE
byte-for-byte to 52ad, confirming all other controls and rows are unchanged.
Host e2e and sabotage remain unrun here by explicit operator instruction. No host runtime
pass on this new candidate is claimed. Fresh exact-head host validation,
published review and actual landing gates remain required before Y-95 ends.
The original parent, Y-96, Y-61 and redesign/release through Y-80 remain
required. Y-81 stays deferred. Models, reviewer and frozen operational daemon
are unchanged. No backlog construction work is manually completed.
