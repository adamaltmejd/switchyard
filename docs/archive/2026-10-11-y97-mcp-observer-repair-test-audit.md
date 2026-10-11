# Y-97 MCP observer repair and test audit — 2026-10-11

## Operator focused-host results and observer repair

The operator reported two fresh focused failures on repaired candidate
`c1f5e85428ed9ce41edcc99e574e6278a9c9f22f`. Both reached real candidate
attention and stale-decision controls, confirming progress past the original
store constraint failure, without establishing a focused or full-suite pass.

- G2 session 32982: terminal exit 101, 0 passed / 1 failed, 15.84 seconds;
  log `/tmp/switchyard-y97-c1f5-stale-host.log`. Line 778 read the repair
  success as repaired[accepted], but trusted MCP returned
  {result: {accepted: false, attention: 2, evidence: ...}}.
- G8 session 87521: terminal exit 101, 0 passed / 1 failed, 23.04 seconds;
  log `/tmp/switchyard-y97-c1f5-failures-host.log`. Line 1606 read the gate
  acceptance success as accepted[accepted], but trusted MCP returned
  {result: {accepted: true, ...}}. The later carried-block acceptance had
  the same incorrect observer near line 1777.

Read Operator.call and operator_mcp::tool_result at the actual boundary.
The helper returns structuredContent, which intentionally retains the
operator result/error envelope. Correct only those three success observers
to read result.accepted. Keep the CLI acceptance observer near line 1663
direct, and leave the helper and public contracts unchanged.

Test-audit disposition: retain all three assertions in their existing G2/G8
blocks. Their expected false/true values come from the fixture's explicit
repair/accept choices and the specified operator decisions. Their sabotage,
hard cases and absence of twins remain as recorded in the original Y-97
audit and scenario comments: lose exact identity binding, rewrite failures,
skip remaining checks or consume publication before teardown. This repair
corrects the external observer; it adds no scenario and weakens no assertion.
All stale named identities, no-write comparisons, positive repair/default
controls, held publications, restart, unchanged judgments and landing
observations remain. Test lines removed: 0. Binary lines changed: 0.

No fresh host or sabotage pass is claimed. Both reported sessions are
terminal; the full repaired suite has not run. Fresh exact repaired focused
and full host tests, published review and actual landing gates remain
required. No e2e execution occurred in this candidate.

After the observer correction, `cargo fmt --check`,
`cargo clippy --all-targets --locked -- -D warnings` and `git diff --check`
pass. These source checks do not claim a host runtime pass.
