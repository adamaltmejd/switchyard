# Y-157 archive filter fixture repair — 2026-10-15

The operator reports exact 581d34906e64a1b4914f76bd0153bfee6cb15337
failed the complete native affected case 0/1 at the archive-byte assertion
with ENOENT. Runtime was 89.86 seconds suite, 94.8247 seconds wall.
Evidence stays at
`/home/yard/validation/y157-581d34-durable-20261015-evidence`.
Retained external tool output identifies the cause:
`fatal: chain.txt: smudge filter trip failed`. The bash tool returned zero
because the pipeline reported tar's success after Git failed.

The fixture configures `filter.trip.required=true` and only a clean
command. Archive applies the missing required smudge filter. The archive
invocation now overrides trip clean, smudge and process to empty and
required to false. These overrides apply only to that command. Repository
configuration and the preceding ordinary helper positive controls remain.
The existing exact host archive-byte assertion remains unchanged.

## Test-audit gate

- Retain the G9/G13 selected cleanup hard case. It protects actual committed
  source and exclusive handoff against predecessor Git configuration.
- Existing sabotage comments name removal of either production worktree
  binding. The exact committed bytes and dirty alternate status remain
  external native Git observations. The archive-byte assertion verifies
  setup against the scripted implementer's exact bytes.
- Change only the existing archive invocation. Add no assertion, scenario,
  machine, build, box, model turn, generic helper, retry or barrier. Test
  lines removed: 1, replaced by 1. Binary lines removed: 0.
- Preserve redirection at the existing initial cleanup hold, alternate-byte
  substitution, source-byte probe, unsafe recovery, every prior refusal,
  repair and landing assertion. Both production bindings remain unchanged.
  No coverage is removed and no coverage-restoration work is added.

This is the changed-test gate, not a suite sweep. ARCHITECTURE.md already
states the implemented product behavior; no contract or design changes.

## Validation and evidence

A native Git probe at `/tmp/y157-filtered-archive-DSYzhn` reproduced the
required-filter archive failure with the fixture's filter configuration.
It observed the ordinary helper markers, then ran the corrected archive
invocation and compared its bytes exactly with `implemented\n`. Required
and clean configuration remained unchanged afterward. It ran no daemon,
worker, package or model and does not establish real boxed acceptance.

All prior failed runs remain separate, including 82d4b7, 8f031df and
581d349. The earlier c314449 host pass and e1b93d interrupted diagnostic
retain their original meanings. Existing archives and evidence are unchanged.

No e2e runs in the candidate box. The operator must run the complete affected
native case and prepared full 50-case host gate. Canonical target preparation
remains the normal host-owned landing-conflict return after qualification.

`cargo fmt --check`, `cargo clippy --all-targets --locked -- -D warnings`
and `cargo build --locked` passed. No repaired native case pass is claimed.
