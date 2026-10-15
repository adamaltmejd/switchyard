# Y72 native stale approval and first packaged revision repair

The affected native G2 case at exact 167ee6d failed 0/1 in 45.21 seconds
(48.825 seconds suite wall time). Planning, implementation, changing cleanup,
current and unknown audits, and the original packaged reference completed.
The stale approval returned `stale` and the exact old/current head hashes.
The fixture incorrectly required an extra `identity: head` field.
`api.rs::Fail::stale` promises `data: {expected, current}`. The repair removes
only that unsupported field expectation. The stale token, Git-established
hashes and unchanged approval rows remain asserted. The later exact current
approval and actual landing remain the positive control.

Evidence is preserved at
`/home/yard/validation/y72-167ee6-reuse-20261015-evidence`.
Log SHA256:
`fc30bc87be844783d6abf00a03f9096ec4d4a5243a1aaf267db23c66e686d1e3`.
This is a normal native assertion failure. It is separate from the earlier
operator-interrupted diagnostic. The parent restored source and left runtime
unchanged. No-op, feedback, effort and native landing acceptance have not
yet completed.

Test-audit disposition: retain the G2/G9/G13 block. It protects stale
authorization after actual changed publication. Sabotage remains retaining
old approval or allowing a stale approval write. Expected hashes come from
the fixture's Git publications; the API defines the stale token and envelope.
One test source line is replaced; no assertion block is removed. No additional
machine, build, box, model turn or scenario is introduced. The binary repair
adds a contract at revision and removes zero binary lines.

Review1974's effort loss is reachable within the existing fences. A handoff
with ordinary writers has `ordinary-linear` runtime. Adding a configured
read-only `cleanup-audit-*` packaged report keeps that runtime and is accepted
by `handoff_selection`. The writer comparison clears routes and freezes each
writer's access, agent and outputs, so adding the audit route does not change
writer grants. At candidate attention, the revision transaction accepts the
change and withdraws approval. No unsupported transition is needed.

Previously this revision populated the new package's current identity through
`Loaded::inputs` but did not record a launch capability. It worked only while
the receiving daemon's compiled package matched. The first selection now
records the same `effort` launch and `exclusive-v1` checkpoint contracts as
initial admission. This occurs only when the old plan has no packaged node
and no launch contract is already recorded. Retained packages keep their
frozen identities and capabilities, including unknown bytes and absent
contracts. All runtime, writer, active-work and transaction fences remain.
The existing `package_inputs` capability reading consumes the persisted
record. No package registry or per-node capability framework is introduced.

The first-package revision boundary is checked by source inspection here;
no extra host scenario is added. The same G2 fixture still observes medium
effort at the scripted host model. The exact 7dbc822 feedback fix and 29a55a2
consumer restrictions remain intact.

Candidate checks: `cargo fmt --check`,
`cargo clippy --all-targets --locked -- -D warnings`, `cargo build --locked`.
No e2e runs in the worker box. The complete repaired native case and full
prepared 51-case host gate remain required before actual landing.
