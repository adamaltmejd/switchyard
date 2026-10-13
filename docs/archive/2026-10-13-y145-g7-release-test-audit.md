# Y-145 G7 checked-release observation repair

The reported dependency validation on head
`6675b55a145a42d1da9e264dab3afd7daad1ac4d`, GitHub CI run
`38420517969`, arm64 job `115376295542`, used Pinfold 0.2.3.
The full real native suite failed with 78 passing scenarios and one failure
in 391.76 seconds. The failed log remains at
`/home/yard/validation/pinfold-0.2.3-pin-verification-20261013/ci-arm64-115376295542.log`.
That host log is not available in this candidate clone; these failure details
come from Y-145. No retry or later green run supersedes this failure.

The failing scenario is
`g07_usage::codex_failed_and_interrupted_resume_usage_remain_execution_local`.
Its full-row comparison at the validation head's line 427 took the third
worker snapshot immediately after `stopped:failed` attention. Before restart
the row had `reserved=true` and `teardown=false`; afterwards it had
`reserved=false` and `teardown=true`. Input, output, partial provenance,
result and supplier/session identities were identical. Terminal attention
does not establish checked release. Store reservation release has its own
`execution.released` audit event.

Disposition: retain the scenario and every existing assertion. Wait through
the real `status --history --since` stream for `execution.released` naming
the interrupted attention's execution. Before taking the restart snapshot,
the CLI row must name that execution with `reserved=false` and
`teardown=true`. Checked teardown and release commit together. The fixture's
next host model request remains held through the real daemon kill/restart.
The same full execution object is compared after reopening. The wait observes
the required historical evidence after actual cleanup; a deadline panics,
never passes. No projection hides ownership fields.

Test-audit Gate answers:

1. G7's Codex resumed usage clause requires local fixture tokens, partial
   failed/interrupted provenance and unknown cost across reopening. Its
   Shown by text now explicitly names full-row immutability after checked
   release. The added identity/teardown assertion establishes the snapshot
   boundary for that existing historical comparison.
2. The test comment names adding cache/reasoning subsets, copying predecessor
   totals, losing terminal/nonterminal counters, labelling interruption
   complete, and rewriting historical session-local evidence on reopen.
   These break the literal usage, provenance or full-row assertions.
3. No twin scenario is added. Fresh completed Codex controls cannot expose
   resumed baselines or lost terminal frames. Pi and Claude exercise different
   harness readers. This existing case fails a resumed tool turn, interrupts
   another with its next real host request held, reopens the daemon, then
   completes a resumed control. The held request remains held before restart;
   the exact released identity and settled CLI row prove the historical state
   outside the binary before killing the daemon.
4. Token expectations remain literal host model values. Partial/unknown
   expectations come from G7 and the missing terminal/cost measurements.
   The externally read attention identity binds the release event and CLI
   row. Checked teardown supplies the expected false/true ownership fields.
   Equality across reopen is the promised immutable historical evidence,
   including supplier/session identities, result and usage provenance.

No assertion block or binary line is removed. One sabotage comment line is
replaced. No product semantics, fixture holds, concurrency, deadlines, flags,
runtime seams, sleeps or retries change. This is the Gate for one observer
repair, not a release sweep. No `.yard` file changes. Y-145 is independent of
Y142 and does not discharge any parent redesign scope.

Validation: `cargo fmt --check`,
`cargo clippy --all-targets --locked -- -D warnings`, `cargo build --locked`
and `git diff --check` pass. The real host suite is required as the
post-approval landing gate; the ticket explicitly forbids running
`cargo test -p e2e` in this clone. Neither the native suite nor runtime
sabotages were run here.
