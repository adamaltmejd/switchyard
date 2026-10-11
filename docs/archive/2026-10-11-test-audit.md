# Y-77 G7 assertion gate

The new Codex scenario in `g07_usage.rs` is kept. No assertion is removed
or weakened. Y-66's fresh completed/nonzero/zero and legacy-null scenarios
remain the controls. Responses envelopes now report cached input and
reasoning output as subsets of the same fixture totals. No second zero-cost
scenario is added.

G7's new clause promises meaningful Codex token totals, unknown unreported
cost and execution-local increments through failed, interrupted and resumed
turns. The new block runs real Codex in real boxes against the scripted host
Responses fixture. After an ordinary completion, a resumed tool turn fails
at a non-retryable fixture response. Another resumed tool turn is held at its
next model request while the host ends the actual box. The daemon settles
that execution before it is reopened. A later resumed execution completes.
This is interruption coverage, not a daemon crash-at-a-ref scenario.

Expected token values come from the host fixture: each tool and text reply
reports 1000 input and 100 output, with 500 cached input and 50 reasoning
output already included. Failed and interrupted invocations each receive
one measured reply. The final successful invocation receives one tool reply
and one text reply. The external observer is CLI attempt JSON backed by the
store. The host latch proves the interrupted request remains held; the
read-only store supplies the actual box handle. The reopened execution JSON
must preserve the same interrupted record and exact resumed identity.

Sabotage is adding either subset again, saving cumulative predecessor
spend, discarding nonterminal counters, or calling interrupted totals complete.
The existing Y-66 controls cannot catch resumed baselines or lost terminal
frames. Pi and Claude have different usage transports. The successful resumed
completion is the positive control in the same project. The old nullable and
fresh zero controls are reused unchanged.

Test lines removed: 0. Binary lines removed: 33 in the final diff, chiefly the
store's resumed label-only guard and replaced stdout normalization. No control
or feature is removed. The existing nix dependency enables its `dir` feature
for descriptor-relative, no-symlink session discovery. No dependency is added.
No public endpoint, binary test flag, sleep, retry loop or test-only barrier
is introduced by the scenario.

Validation: `cargo fmt --check` and
`cargo clippy --all-targets --locked -- -D warnings`. Host e2e and sabotage
execution are deferred to the landing gate, as the ticket requires. The added
scenario is acceptance coverage, not a claim that the host suite passed.
