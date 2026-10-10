# Y-87 committed scaffold input expectation correction

The operator ran the complete real host suite on candidate
`bf25cf9ef0df9841a3d9c76d62af2d7a9e1d764d`. It ended with 52 passed,
1 failed in 204.88 seconds, exit 101. The operator's log is
`/tmp/switchyard-y87-bf25-host.log`. G10's admission input equality at
line 284 expected only the custom admission skill and `AGENTS.md`.

The recorded inputs correctly included the committed `yard-drive`,
`yard-file` and `yard-operator` skills. Real initialization writes those
three embedded repository skills; `Project::new` then stages and commits
them with `git add -A`. The operator independently hashed the repository
skill sources with host Git. In that order, their blob identities were
`7e5173235843c3ffc2571cc658c99befdf88e1d0`,
`f2918b9c48e62c8396e8db9f4f61701e5d28e30d` and
`b2d72ea7165c367f6d31f07ef029914f3174c0e3`, matching recorded inputs.
The failed temporary project had already been cleaned up.

The correction names each of those three paths literally in the existing
fixture expectation. Host Git in the live fixture resolves each blob from
the admission base, just as for the custom skill and `AGENTS.md`. Expected
values are not reconstructed from admission output or an allowlist copy.
Complete equality and every existing assertion remain in place. No source
is removed from capture or from the fixture. The root-tree fix and current
template consumers are unchanged.

Test-audit disposition: keep the existing G6 input identity block in the
G10 named-target conflict scenario, correcting its incomplete expectation.
It establishes G6's shown clause for host Git input identities that remain
fixed across repair and quiescent restart. Sabotage: omit the committed
scaffold skills from provenance capture; full equality fails. The comment
now names that sabotage alongside repair overwrite and startup regeneration.
Other blocks do not inspect this complete committed input set across those
transitions. The fixture supplies literal paths and executable bits; host
Git supplies the blob identities. No new scenario or duplicate assertion
block was added. Repair/restart immutability and historical unknown evidence
assertions are retained.

Test lines removed: 0. Binary lines removed: 0.

`cargo fmt --check` and
`cargo clippy --all-targets --locked -- -D warnings` passed for this
correction. Host e2e was not run here, as instructed. The failed candidate
has no approval or landing authorization. The operator will rerun the
complete host suite on the new exact candidate.
