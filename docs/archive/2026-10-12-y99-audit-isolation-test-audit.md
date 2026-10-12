# Y-99 audit isolation and test audit, 2026-10-12

Y-99 implements the first bounded G13 isolation child of Y-93. It adds no
runner selection, package change, checkpoint resume or independent lifecycle.
The execution row's existing attempt, node and plan identity owns its new
`audits/<execution>` directory. The admitted execution reservation, inherited
host-child lock, checked teardown and whole-attempt cleanup remain authoritative.
Y-92 publication and ownership settlement are unchanged.

Read-only implementations now clone their recorded head or admission base
into a fresh detached checkout. They copy available proof through the existing
bounded, link-refusing snapshot function; absent proof produces an empty
snapshot. Both mounts are read-only. Harness state and launch input are fresh
per execution. No predecessor session is selected or mounted. Host preparation
never enters a predecessor's guest-written state. An existing execution path
is refused rather than reused. Cleanup uses the existing link-refusing whole
attempt removal after ownership proof. Writable implementations and review
seats retain their mounts, staging and resume contracts.

## Test-audit Gate

Keep and extend
`a_packaged_audit_refuses_missing_assets_then_publishes_in_a_corrected_box`.
Keep its existing G8 packaging/preflight/declared-read assertions and G12
secret isolation assertions. Extend its corrected-image G13 control rather
than add another packaged runtime scenario.

1. Exact G13 shown-by clause: frozen source, trusted input and proof writes
   fail while scratch/checkpoint writes succeed; an edit starts a fresh audit
   on the same attempt while the predecessor checkpoint remains on the host;
   the replacement cannot read or modify it, resumes no session and retains
   its admitted node; guest checkpoint links cannot change a host sentinel
   through whole-attempt cleanup. The box table and audit ownership contract
   change in the same commit.
2. Sabotage: reuse attempt state for read-only executions, select a predecessor
   session, omit the proof mount or make it writable, or follow checkpoint
   links during host cleanup. The corresponding fresh-marker, resume-row,
   mounted-proof/write-refusal or host-sentinel block fails. Source mount
   sabotage remains covered by the existing source/trusted-write block.
3. Hardest distinct case: the first real packaged child has published, but
   its outer Pi is held by the scripted model. A real box exec writes a
   fixture checkpoint and links a checkpoint entry to a host sentinel
   directory. An edit supersedes that execution. The replacement Pi is held
   before invoking its child, while the first checkpoint is still present on
   the host. Absence caused by cleanup cannot satisfy isolation. Its eventual
   real child publication and writable scratch/checkpoint probes control the
   refusals. History observes attempt completion and cleanup, without sleeps,
   retries, timeout changes or private launch-input writes.
4. Outside oracles: checkpoint text and sentinel bytes come from the fixture.
   The host reads the retained checkpoint before and during replacement.
   Real pinfold exec observes mount presence, denied writes, absence of the
   prior checkpoint in fresh state, and denied reads/writes using its host
   path. CLI/store observe G13's null resume and the fixture plan's implementer
   node. The existing packaged child supplies the successful report and
   regular source/trusted input controls. Cleanup history is followed by a
   host sentinel read. No expected checkpoint value is derived from Yard.

Twin review: G13's gate/seat scenario freezes an implementer's proof for
review and landing; it does not create an audit successor or retain audit
checkpoint state. G14's writable repair scenario replaces a planted models
file and refuses malformed worker Git; it does not isolate fresh audit state.
G14's proof-link scenario refuses a worker-produced candidate by name; it
cannot show a read-only audit's proof mount. G8's superseded-audit scenario
checks report identities and refusal, with a one-execution session bound;
it does not observe retained checkpoints or guest state links. Keep those
blocks. The package's existing source/trusted-write and scratch/checkpoint
blocks survive; no parallel copy of those observations is added.

Test lines removed: 2 (the closure declaration and corrected-run helper call
replaced by the held successor scenario). Binary lines removed: 27 (shared
path/proof handling and audit candidate fetching replaced by frozen inputs).
No guarantee or existing shown-by promise is weakened.

## Validation

`cargo fmt --check` and `cargo clippy --all-targets --locked -- -D warnings`
pass in the candidate box. Host e2e is not run here, as the ticket directs.
The real daemon/box suite remains the operator-owned landing gate. The new
scenario has been reviewed and compiled, not claimed as a host runtime pass.
Y-93 remains the final full-scope integration ticket.
