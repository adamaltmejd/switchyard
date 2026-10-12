# Y-107 operator validation — 2026-10-12

Y-107 landed quiescent append-only plan revisions and exact result
qualification. Y-108 live handoff, Y-109 definition adoption and the full
original Y-62 integration remain required. This is not release completion.

## Identity and landing

The candidate and actual merged commit are
`cd0c220e41c5b36eb00dfcefaa7ce7c28b4b1ad7`, based on
`329619373ca312bdec1ac4dec7b26a51c5b814e2`. The merged tree is
`1564daa09ec77a7d2036109458e04216aeb7e7a1`.
The live candidate diff matched the detached operator checkout exactly:
SHA-256 `2328d484089245771a651b6bc919c1b3bcf13dee14817937e28b5d2033f3e50e`.
The original implementation, repairs and protected architecture changes
were read before approval. The prior sizing-only candidate was not landed.

Boxed fmt 2199, clippy 2200 and build 2201 passed as checks 1144–1146.
DeepSeek review 2202 passed as check 1147 with no findings. Protected
attention 278 named ticket revision 4, the exact candidate and empty-proof
digest. Operator approval 126 admitted that identity to `main`.

Landing 2203 passed fmt 2204, clippy 2205 and build 2206. Its real host
gate 2207 passed all 76 scenarios with zero failures in 574.40 seconds.
The recorded landing is audit sequence 7647. Y-107 is done and attempt
104 ended landed. Cleanup 2208 ended removed. Sync consumed the exact
landed commit after those outcomes were verified.

This host measurement is about 9 minutes 34 seconds. It exceeds the
architecture's historical five-minute landing budget. Final release
measurement and budget review remain required; no scenario was removed
or weakened to meet the budget. This is Linux x86_64 evidence only.

## Retained failures and repairs

The first implementation, `fa653455`, passed its original G2 control.
Changing only the admitted fixture profile instructions refused the first
revision with `unsupported_revision` at G2 line 89. This exposed that the
equivalent-profile control did not establish an actual setting change.

The next candidate, `b64d940`, passed the meaningful final-profile control.
Its review check 1143 reported P2: an ordinary complete/blocked profile
could not be selected for a non-final complete-only seat. An operator
probe changed only the first revision target to that non-final seat and
failed directly with `unsupported_revision` at G2 line 100. The candidate
was rejected. The same review's P3 identified unshown routing retention.
Both failed probes remain preserved; later passes do not erase them.

The landed repair interprets profile outcomes as capabilities while the
ordinary seat supplies actual outcomes and bounded routes. The scenario
now selects the same profile on final and non-final seats. A third seat
lets a literal routing revision remove the middle supplier while retaining
the first seat's completed evidence. The final consumer takes fresh
evidence bound to its actual replacement supplier.

## Host controls and sabotage

The exact landed candidate's focused G2 scenario passed: one passed,
zero failed, 75 filtered, 75.36 seconds scenario time and 90.4926 seconds
total. The command was `cargo test -p e2e --locked
quiescent_plan_cas_retains_only_exact_node_and_producer_evidence -- --nocapture`.
It used the established isolated runtime profile and durable fixture inputs.

Named complete-route sabotage removed only the two normalization calls
in `executions::qualifies`. The real host scenario failed at G2 line 299:
the selected checks omitted the retained first reviewer check. Exit 101,
29.35 seconds scenario time and 29.4292 seconds total. Later assertions
were not reached. This was a direct assertion failure, not a timeout.
Exact source bytes were restored, and `cargo build --locked` passed.
Restored executions.rs SHA-256 is
`aec8bde1f424231a03c886c93c7ac4a55cf2f0ea283db65dba3e00b8c391a243`.

Earlier typed-producer and selected-digest sabotages also produced direct
failures. Their logs retain the first failed assertion and unreached-block
limits. The landed executions.rs bytes match `b64d940`; its digest sabotage
was not repeated after an unrelated guard and scenario repair.

The changed blocks establish G2/G5/G9. Expected graph, instructions,
outcomes, revisions and execution order come from fixture literals;
candidate identity comes from Git. Outside store observations establish
identity agreement and immutable history. The three-seat case separates
successor changes from actual supplier changes, while retaining live and
stale refusal, reopen and corrected approval controls.

Raw controls, mutations, failures, restoration and landing evidence remain
in the private operator validation directories. No raw transcript or
credential was added here. The frozen operational daemon and reviewer
were preserved; Y-81 remains parked.
