# Y-89 explicit restart repair

The operator's exact-head host run on candidate
5343f2f9ebfd6dd656cc972f5d99e74042fe0cf7 passed 58 scenarios and failed one
in 259.02 seconds. The existing G9 scenario
`an_edit_reaches_the_next_prompt_and_stop_delivers_it_sooner` observed
`first, edit` after an edit, stop and explicit start. Its fixture and the
supported operator exit require `first, restart`. All new Y-89 scenarios
passed. The host log is `/tmp/switchyard-y89-534-host.log`.

The repair restores recorded next-execution intent before automatic edit
selection and removes the blanket reason and feedback override from
`start_implementation`. That function still captures the current ticket
revision and body in the execution's start transaction. Failed gate and
blocking review routes retain their transactional `superseded_by_edit`
guards. A pending edit prevents those automatic repairs from recording an
execution or consuming review rounds. Repair feedback keeps its source
execution identity.

Test-audit disposition: retain the existing G9 scenario and every assertion
unchanged. The observer is the CLI, store and host model fixture; the expected
restart reason comes from the operator's explicit start and G9. Restoring
either edit override sabotages its existing reason assertion. No new scenario
is needed for the same mechanism. No test lines or controls were removed.

Boxed validation: `cargo fmt --check` and
`cargo clippy --all-targets --locked -- -D warnings` pass. Runtime acceptance
remains pending on the host: the focused G9 scenario, exact-head full suite,
fresh review and fresh landing validation. No e2e suite ran in the box.
