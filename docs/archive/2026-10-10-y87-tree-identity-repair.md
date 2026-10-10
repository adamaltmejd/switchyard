# Y-87 host validation and tree identity repair

The operator ran the complete real host suite on candidate
`12cae4ba89b4e78778652c0a2d84878d2607d992`. It ended with 0 passed,
53 failed in 40.10 seconds, exit 101. The operator's log is
`/tmp/switchyard-y87-12ca-host.log`. Sync and migration consistently failed
with `invalid: configuration commit has no tree`.

The admission input reader passed `commit^{tree}` to `Git::rev_parse`.
That method resolves commits only and appends `^{commit}`. The resulting
`commit^{tree}^{commit}` asks Git to treat a tree as a commit. External host
Git confirmed that `HEAD^{tree}` resolves the real tree and that the extra
commit peel refuses it. The earlier boxed fmt and clippy results did not
establish host runtime behavior.

The repair adds `Git::root_tree` at the existing bounded Git boundary.
It resolves `commit^{commit}^{tree}` with `rev-parse --verify` and returns
the real root tree identity. The existing Git runner supplies explicit cwd,
scrubbed environment, bounded output and time. `Git::rev_parse` and every
existing caller retain their commit-only contract. Admission records this
tree directly; no identity is synthesized.

All existing and added test assertions remain unchanged, including the
fixture's external `base^{tree}` expectation. Existing host CLI sync and
historical migration controls caught this defect, so no duplicate test was
added. The original test-audit gate still applies. Legacy unknown evidence,
current template lookup, approvals and the dependent Y-85 freeze scope are
unchanged.

For the repair, `cargo fmt --check` and
`cargo clippy --all-targets --locked -- -D warnings` passed. Host e2e was
not run in this candidate environment, as instructed. The failed candidate
has no approval or landing authorization. The operator will rerun the full
host suite on the new exact candidate.
