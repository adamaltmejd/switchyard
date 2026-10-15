# Y-72 repair feedback through cleanup

The blocked candidate redirected repair to cleanup but dropped feedback at a
packaged runner. The following repair started as first work without the failed
judgment or rejection instructions. The fix retains the exact feedback object
in existing execution inputs and forwards it after eligible cleanup release.
Original check, finding-contributor and rejection identities are not rewritten.
Matching retries retain the same object in the checkpoint identity; generic
retry bookkeeping is still excluded from the packaged model's inputs.

## Test-audit disposition

Keep and extend the existing G2 cleanup-input case. Use packaged cleanup in
place of ordinary cleanup on the same two passes. Its first pass uses the
existing real write/commit tools. Its second pass publishes a no-op report.
The existing operator rejection supplies externally known instructions and
the exact rejected Git head. Assert both cleanup and repair retain that exact
feedback object, and the host scripted model observes the repair instructions.
The existing allowed control still approves and lands the exact current
candidate after the rejection. Existing source/proof and immutable-reference
assertions remain unchanged.

This protects G2 exact repair feedback and G9/G13 selected role handoff. Losing
failure instructions can repeatedly republish unfixed code or exhaust review
rounds. Sabotage is named in the test: discard feedback at packaged cleanup or
dispatch repair as first work. Either loses the expected input or prompt.
Expected feedback comes from the operator's rejection text and Git-bound
candidate, and expected prompt content is observed by the host model. Store
equality checks the promised unchanged binding through the handoff.

No new fixture machine, image, build, box or model execution is added. The
existing fixture remains 16 boxes and 12 model executions. Replacing the first
cleanup shell call with write and commit adds one response, while skipping the
second cleanup shell call removes one: 32 scripted responses remain planned.
The existing hard Durable recovery tests retain their source-write and unsafe
continuation assertions. Separate gate-failure and selected-group transport
variants are deliberately not added; both use the same exact feedback object
carrier. Their existing source construction and prompt rendering are retained.

No host e2e ran in the candidate box. The operator's focused and full host
landing gates remain required. Candidate validation uses formatting, clippy
and the locked build.

At audit, the test diff adds 47 lines and removes 7. The binary diff adds
71 lines and removes 8. No assertion is removed or weakened.
