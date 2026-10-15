# Y-157 debugger needle correction — 2026-10-15

The operator's exact 1170ed2 affected native case failed 0/1 in 158.30
seconds, 161.75 seconds wall. Ordinary implementer execution 3 published
its candidate and packaged cleanup execution 4 started. The debugger
observer generated invalid JavaScript by putting the write-tool needle's
single quotes inside a single-quoted JavaScript literal.

`debugger_holds_tool` now serializes its needle with `serde_json::to_string`
and substitutes that complete string literal into the observer script.
The selected write needle remains `return text('source_written');`, after
the awaited file sync and before returning the tool result. Both existing
read-checkpoint cases keep `const path = allowed.get(args.file);` as their
needle. No production barrier, tool, pause point, assertion, expected value
or deadline changes.

## Test-audit gate

This repairs the external observer used by the retained G2/G5 checkpoint
cases and G9/G13 selected cleanup case. They protect checked predecessor
release, exclusive ownership and interruption without unsafe automatic
replay. Their existing comments name binary sabotages: reuse before
release, omit checkpoint exclusion or replay an unsafe tool. The helper
fix makes the selected write breakpoint syntactically valid without moving
the externally held point or weakening those observations.

All three cases reuse the same observer. No machine, historical build,
box, model turn or scenario is added. Expected source bytes come from the
scripted fixture; Node's debugger supplies the held point, with source state
proved from the host before restart. The existing native Git, checkpoint,
pinfold, execution identity and history observations remain. Test diff:
2 lines replaced; binary diff: 0 lines. This is a test-change gate, not a
suite sweep.

## Evidence and validation

The fourth failed evidence directory remains separate and unchanged:
`/home/yard/validation/y157-1170ed-durable-20261015-evidence`.
The earlier 3af1e6, 5e19c0 and 4a6f71 evidence directories remain separate.
The operator observed cleanup ending failed with `missing_report` while
the model response stayed held; `Watch.said_before_end` then failed at
`debug-armed`. The stop preflight found the fixture already gone and made
no mutation. No operator interruption or successful unsafe-write restart,
continuation or landing is established by that run.

No e2e runs in this candidate box. The complete affected real-host case and
final prepared full 50-case host gate remain required before approval.
`cargo fmt --check` and
`cargo clippy --all-targets --locked -- -D warnings` passed for this repair.

The requested target merge is not yet prepared. Commit
`9c34dd56b220badb5db9f5c2a98f9f6b29527ad6` is absent from this clone:
`git cat-file -t` fails. `git remote -v` is empty. Fetching that exact commit
from the original canonical path in `.git/FETCH_HEAD`,
`/home/yard/code/switchyard/.yard/local/canonical`, fails because that
repository is unavailable in the box. `/yard/input` supplies no bundle.
A reachable repository or bundle has been requested. Target contents and
combined-spec validation cannot be inferred from the reported merge-tree
result; no merged candidate or merged checks are claimed.
