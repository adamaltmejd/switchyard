# Y-157 nested Git inspection review — 2026-10-15

The P1 review reproduced a nested clean filter executing through packaged
final status. Top-level filter overrides do not disable independently named
filters configured inside initialized submodules.

Packaged final inspection now refuses an index or HEAD containing Gitlinks.
Bounded `ls-files --stage -z` and `ls-tree -r -z HEAD` read metadata before
status, with the same scrubbed environment and hooks/fsmonitor overrides.
Neither traverses submodule worktrees or runs content filters. Checking HEAD
as well as the index covers staged Gitlink deletions. Command failure also
refuses inspection. Because box exec retains a bounded output tail,
inspection refuses any metadata, filter or status output reaching that
bound; truncated listings cannot hide a Gitlink or configured filter.
Repositories without Gitlinks retain the existing full
status check and dirty-tree outcome; no submodule-ignore flag hides changes.
No image, package identity, lifecycle, grant or unsafe replay policy changed.

## Test-audit gate

Extend the retained selected cleanup handoff case for G13 containment and
G14 untrusted Git. The material failure is predecessor code executing with
writable source/checkpoint access outside the bounded unsafe tools.

After the original successful interruption, continuation and landing
sequence, the same project starts another selected handoff. Its ordinary
implementer creates a real committed, initialized submodule with tracked
attributes and a local clean filter that writes a host-visible marker.
Ordinary predecessor Git must execute it. While the cleanup model response
is held, the host removes the marker and changes the nested file's mtime.
A boxed native Git status with the scrubbed environment and hooks/fsmonitor
overrides must succeed with empty porcelain output and recreate the marker.
This proves the adversarial repository is clean and its nested filter can
execute through the exact kind of inspection under review.

The host removes the marker again, changes mtime again, then releases the
report response. The exact cleanup execution must end as refused and release
ownership without recreating the marker or starting repair. Canonical main
must still contain the first attempt's landed source. Expected markers,
source bytes, Gitlink and cleanliness come from the fixture, native Git and
host files; execution identity and terminal outcome are observed through
history. The adjacent comment names the sabotage: remove the Gitlink guard
and let status execute the nested filter. The original successful handoff
is the allowed control in this same project.

Reuse avoids a new fixture machine, image or build. Added work is three
worker boxes (planning audit, ordinary implementer, packaged cleanup), five
scripted model responses and one boxed Git probe. The cleanup response is
held by the host model, without a binary barrier, sleep, retry or public
endpoint. No original assertion is removed or relaxed. Deliberately untested:
each nested filter type, uninitialized submodules and staged deletion
variants. The guard refuses all Gitlinks, rather than supporting recursive
configuration handling. G13 Shown by names the representative refusal.

Test diff: 126 lines added, 1 replaced. Binary diff: 28 lines added, 2
replaced. This is the test-change gate, not a suite sweep.

## Validation

`cargo fmt --check` and
`cargo clippy --all-targets --locked -- -D warnings` passed. Clippy compiled
the extended fixture. No e2e suite or scenario was run in the candidate
environment; real box acceptance remains the host landing gate. No runtime
measurement or host pass is claimed. Earlier failed evidence is unchanged.
