# Y-157 relative worktree fixture repair — 2026-10-15

The operator's exact 82d4b740bde759522ea5996b9254cf02c6aace76 native
affected case failed 0/1 after 100.93 seconds (105.875 seconds wall).
Ordinary implementer execution 3 could not publish: host fetch/upload-pack
exited 128 because `core.worktree=/workspace/.git/alternate` named a path
that did not exist on the host. Cleanup had not run. Evidence stays at
`/home/yard/validation/y157-82d4b7-durable-20261015-evidence`, log SHA256
`a00e865e0dc76b6dce48d131fa2464940ae818ed5f469426a3f2264008fd4e93`.

The existing implementer script now sets `core.worktree=alternate`.
Git resolves it relative to `.git` on both host and box. The archived
alternate tree still matches HEAD during ordinary publication. The host
still substitutes its bytes at the existing cleanup hold. Both production
`--work-tree=/workspace` bindings remain. No fetching or safety control
changes. ARCHITECTURE.md already specifies the implemented binding; the
product contract and G13 Shown by remain accurate.

## Test-audit gate

- Retain the existing G9/G13 hard case. Committing substitute bytes or
  inspecting another tree can corrupt the handoff or reject valid cleanup.
- Existing sabotage comments name removal of either production binding.
  Exact HEAD bytes catch the commit error. Dirty redirected status and
  successful repair catch the inspection error.
- Expected bytes come from the scripted model and host edits. Native Git
  supplies the status and ancestry observations. All original allowed and
  refused controls, recovery identities, debugger holds, FIFO/history
  observers, repair and approval/landing assertions remain unchanged.
- Add no machine, build, box, model turn, scenario or assertion. No coverage
  is removed. Test lines removed: 1, replaced by 2. Binary lines removed: 0.
  This is the changed-test gate, not a suite sweep.

## Validation and evidence

A native Git probe at `/tmp/y157-relative-worktree-V492bb` passed. It uses
the relative setting and confirms real bare `--no-hardlinks` cloning,
redirected status hiding a source edit, explicit worktree add/commit
recording exact `cleaned` bytes, redirected status reporting the alternate
file dirty and pinned status reporting the source clean. This probe runs
no daemon, worker, package or model and is not boxed acceptance.

The c314449 host pass remains separate: 1/1 in 66.11 seconds suite,
69.52 seconds wall, at
`/home/yard/validation/y157-c31444-durable-20261015-evidence`.
It covers earlier recovery/handoff controls, not the worktree finding.
Earlier failed runs and the e1b93d operator-interrupted diagnostic retain
their original outcomes. No existing archive or host evidence is changed.

The operator must run the complete affected case and prepared full 50-case
host gate. No e2e runs in this candidate box. Canonical target preparation
remains the host-owned landing-conflict return after qualification.

`cargo fmt --check`, `cargo clippy --all-targets --locked -- -D warnings`
and `cargo build --locked` passed for this repair. No repaired host pass
is claimed.
