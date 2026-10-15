# Y-157 boxed Git inspection review — 2026-10-15

The P1 review found that final packaged cleanup inspection ran plain Git
status against predecessor-owned repository configuration. The bounded
commit tool's protections did not cover this later command.

Final packaged inspection now uses an explicit Git executable and cwd, a
scrubbed environment, disabled hooks and fsmonitor, and overrides for all
configured clean, smudge, process and required filter keys. Filter discovery
and status retain pinfold's bounded output and 120-second call limit. A
failed discovery, lossy configuration name or failed status yields no clean
listing. Ordinary agent inspection is unchanged. No image installation,
checkpoint identity, lifecycle or tool grant changes.

## Test-audit gate

Retain and extend the existing selected cleanup interruption/handoff case
for G2/G9/G13. Predecessor-configured code running during inspection would
bypass declared-file and unsafe replay boundaries. This is an unauthorized
effect worth observing in the already retained hard case.

The ordinary implementer installs a real fsmonitor and clean filter. Its
native Git calls must execute both helpers in the same project, as proved
by host-visible marker files. The host removes these markers while the
initial cleanup model response is held. After the deliberate resumed write
and commit, a scripted model response holds report completion while the host
sets source mtime to a fixed old value. This forces final status to inspect
content instead of accepting cached stat data. Neither helper may leave a
marker during packaged cleanup. Repair checks their absence before clearing
the hostile configuration and making its own ordinary Git commit; the host
also checks absence after handoff.

Sabotage, named beside the assertions: restore plain packaged status or omit
filter overrides. Expected markers come from the fixture's shell commands,
not Yard diagnostics. Source mtime is set directly by the host; the model
response is held externally, without a binary barrier or sleep. Existing
source-write, ownership, interruption and landing assertions remain.

This adds no fixture machine, build, image, worker box or model turn. It adds
one held model response and ordinary Git calls inside the existing
implementer turn. No separate scenario or assertion block was removed.
Test diff: 41 lines added, 3 replaced. Binary diff: 61 lines added, none
removed. This is the test-change gate, not a suite sweep.

## Validation

`cargo fmt --check` and
`cargo clippy --all-targets --locked -- -D warnings` passed. Clippy compiled
the extended e2e fixture. No e2e scenario or suite was run in the candidate
environment, as instructed; real box execution remains the host landing
gate. No runtime measurement or host acceptance pass is claimed here.
