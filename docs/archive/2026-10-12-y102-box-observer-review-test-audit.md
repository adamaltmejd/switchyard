# Y-102 box observer review

The correctness review found a host-wide empty-box assertion in the selected
publication ownership control. The presence-only `dev.yard.project` label
includes concurrent fixtures and host projects. The Y-83 observer audit at
`2026-10-10-y83-project-box-observer-test-audit.md` records the same failure.

Keep the G8 assertion block. Capture the publication execution's recorded
box handle before releasing its real exec hold. Assert that specific box is
absent from pinfold's real listing while the fixture holds its host lock.
The earlier absence check names the stopped predecessor, a different box.
Unrelated boxes do not affect either assertion.

The test-audit gate retains G8's shown-by clause: native publication followed
by real host lock refusal remains reserved and ineligible despite box absence,
with no cleanup successor until checked release. The existing test comment
names the sabotage: count publication or terminal success before checked
ownership, or start cleanup while reserved. The hardest case and external
oracles remain the selected native publication, held real listing, same-inode
host lock and fixture report. The store handle anchors the execution-to-box
identity; pinfold supplies actual box presence. This repairs the observer and
adds no scenario or duplicate block. No guarantee is weakened.

Binary lines removed: 0. Test lines removed: 1, replaced by the scoped observer.
No serialization, retries or capacity changes were added. Boxed
`cargo fmt --check` and `cargo clippy --all-targets --locked -- -D warnings`
pass. Host e2e was not run and remains operator-owned after approval.
