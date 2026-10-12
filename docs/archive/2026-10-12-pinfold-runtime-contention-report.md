# Investigation: box listing stalls during concurrent rootless Podman work

Draft for Pinfold maintainers. Not published. Fault ownership is unresolved.

## Observed behavior

On an unprivileged LXC guest, real `pinfold box list --label <fixture-label>`
calls exceeded the caller's 30-second control deadline during an unchanged
Switchyard end-to-end suite. This prevented fixture initialization or proof
that a completed box was gone. Switchyard retained capacity when ownership
was unknown; these refusals were not accepted as successful teardown.

Actual native Podman listing processes were captured at ages 10-29 seconds
with fcntl_setlk waits and verified ancestry to their fixture/test process.
The capture did not identify the exact lock or holder for the two final
timeouts. Earlier observations identified listing waiters on an image-store
lock held by another process. Do not assign those earlier holders to the
final failing listings. An idle native listing later completed in 0.027 seconds.

## Environment

- Pinfold source pin v0.2.2.
- Podman 5.4.2, rootless UID/GID 1000.
- Kernel 7.0.14-19-pve; unprivileged LXC on Proxmox.
- Native overlay on ext4 reports Supports shifting:false.
- No OOM kill, CPU quota throttling, disk media error or filesystem-full
  condition was identified by the host investigation. Idle and cumulative
  resource measurements do not rule out pressure during the failing calls.

## Source observations and hypothesis

Pinfold's Podman list implementation invokes `podman ps --all --format json`.
Pinfold applies requested labels after obtaining the runtime list.
Its rootless run path uses `--userns=keep-id`.

Podman 5.4.2's vendored storage CreateContainer holds layer/image write locks
while finding or constructing a compatible ID-mapped image layer. Overlay
UpdateLayerIDMap mounts the merged filesystem and changes ownership, with
possible copy-ups. This is a plausible source of long shared-lock holds,
not proof that it caused the final two timeouts.

Sources:

- https://github.com/adamaltmejd/pinfold/blob/v0.2.2/crates/pinfold/src/core/runtime/podman.rs
- https://github.com/containers/podman/blob/v5.4.2/vendor/github.com/containers/storage/store.go
- https://github.com/containers/podman/blob/v5.4.2/vendor/github.com/containers/storage/drivers/overlay/overlay.go

## Changed-runtime experiment

The native full suite reported 70 passed/2 failed in 611.10 seconds.
An isolated Podman graph/run root selected fuse-overlayfs through a
process-local PATH wrapper. No global storage configuration changed.
The public base image identity matched native storage exactly.
With /dev/fuse available, the existing package pair passed 2/2 in 136.95 seconds
and the unchanged full suite passed 72/72 in 409.02 seconds. An observer that
records actual descriptor lock identities and holders saw no listing process
aged ten seconds or more in either successful run.

The experiment changed both storage implementation and graph/cache state.
It is not a controlled demonstration that FUSE alone fixes the problem, or
that FUSE is a required Pinfold dependency. No timeout, test concurrency,
user namespace, security grant or assertion changed. Real Pi and real boxes
were used. Test model replies came from the existing scripted host fixture.

## Request

Please classify the behavior as a Pinfold issue, Podman/storage issue or
unsupported host configuration, and identify the appropriate upstream owner.
A useful next reproduction would correlate every slow listing's actual lock
identity/holder with concurrent build or keep-id preparation on native
overlay, without changing deadlines or reducing workload concurrency.
Consider whether Pinfold can provide bounded control behavior or diagnostics
for this condition while preserving truthful ownership and containment.

## Evidence available privately

Native full-suite log and actual-process/lock observations; isolated-profile
focused/full logs and enhanced lock/pressure observations; exact candidate
SHA 23e55b2d0f7d38bfd14a6ce11ba6dcc837cebf5e; runtime/profile identity manifest;
host investigation and backup-hold handoff. Paths and fixture labels should
be redacted before publication. The actual merged-tree Yard landing passed
all 72 tests in
455.63 seconds and was recorded by the queue. That confirms the merged
validation result; it still does not identify the native-overlay root cause.
