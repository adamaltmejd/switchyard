# Y-157 canonical observer correction — 2026-10-15

The operator's real host run of exact
`3af1e6782989d0860abeb34053ba0b3c89c1b342` failed the focused selected
cleanup case, 0/1, after 111.34 seconds (123.75 seconds suite wall time).
Audit write refusal and checked release were reached. The first canonical
content observation failed with ENOENT because canonical.git is bare.
The operator retained the raw log, terminal and fixture store at
`/home/yard/validation/y157-3af1e6-durable-20261015-evidence`.
This change does not alter or remove that evidence.

## Test-audit gate

Correct the three observers in the existing G2/G9/G13 selected cleanup
handoff case to use native `git show refs/heads/main:chain.txt` in the
canonical repository. Preserve all three assertions and their expected
values: base after read-only audit, base before approval, repaired after
landing. These values come from fixture source and worker commits, outside
Yard. Native Git reads the intended ref in the real bare repository.

These assertions protect read-only audit authority, approval before source
landing, and truthful landing outcomes. The adjacent comment names the
sabotages: mutate main through audit, move main before approval, or report
landing without updating main. This repairs the already retained hard case;
no fixture machine, build, image, box, model turn or assertion is added or
removed. Test diff: 5 lines added, 3 replaced. Binary diff: zero.
No production behavior, guarantee, schema or coverage description changes.

## Validation

`cargo fmt --check` and
`cargo clippy --all-targets --locked -- -D warnings` passed. The e2e suite
was not run in the candidate environment, per the ticket. Unsafe write,
restart, continuation and landing still require a passing real host run;
the failed run is not evidence for these later phases. The separate boxed
Git hardening repair remains in commit `2618a6d`.
