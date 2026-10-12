# Isolated FUSE host validation

The construction landing gate selects a separate rootless Podman storage
profile through PATH for its own process. Operational Podman storage is
unchanged. This is a temporary validation arrangement under a backup hold,
not permanent FUSE operation or a release deployment.

## Evidence

Candidate 23e55b2d0f7d38bfd14a6ce11ba6dcc837cebf5e was tested unchanged.
The native full suite failed with 70 passes and two failures in 611.10 seconds.
Real listing processes waited on filesystem locks and exceeded the bounded
control deadline. Their exact final lock holders were not identified.
Earlier observations identified shared image-lock contention. An idle native
listing took 0.027 seconds. These observations do not prove the precise cause
of the final failures.

The isolated profile uses Podman 5.4.2 and the installed fuse-overlayfs.
Its graph root is /home/yard/validation/podman-shifting-profile/storage.
Its run root is /home/yard/validation/podman-shifting-profile/run.
It reports Supports shifting:true; native storage reports false.
The public base image has the identical ID in both stores:
c5fc428fad13dbabcee7880a41a2715d48d17ee15b041aad3b1c3858b126e978.
The wrapper delegates arguments, streams and exit status to /usr/bin/podman.
Pinfold user mapping, seccomp, grants, tests and timeouts are unchanged.

With /dev/fuse available to UID 1000, both existing package scenarios passed
in 136.95 seconds. The unchanged full suite passed all 72 tests in
409.02 seconds. Both previously failing cases passed. The observer captured
no listing process aged ten seconds or more. This establishes this run's
result, not a guarantee that contention cannot recur.

Evidence files on yard-sthlm:
- /tmp/switchyard-y92-23e-fuse-enabled-package-host.log
- /tmp/switchyard-y92-23e-fuse-enabled-full-host.log
- /tmp/switchyard-y92-23e-fuse-enabled-full-observations.jsonl
- /tmp/switchyard-y92-23e-fuse-enabled-validation.json
- /tmp/homelab-yard-fuse-handoff-20261011.txt

## Host agreement

The homelab operator hot-added /dev/fuse without restarting the guest or
operational Yard daemon. The existing UID map, outer AppArmor profile,
seccomp and feature flags were retained.

On October 12 the operator excluded CT 2015 from its only scheduled snapshot
job and recorded the same exclusion in the homelab inventory. The hold has
no automatic expiry. Manual snapshot and snapshot/suspend backups are also
held by operator agreement. Other guests retain their backups.

The host inspection at 06:33:52 CEST examined 80 processes and six mount
namespaces with no inspection errors. No guest FUSE mounts, validation
processes or FUSE helpers remained. That inspection does not establish
cleanup after a future run.

Before restoring backups, stop initiating validation and other guest FUSE
use. The homelab operator must inspect every process mount namespace and
prove no guest-served FUSE mounts, FUSE helpers or validation/build processes
remain. Incomplete inspection keeps the hold. Only then remove the 2015
exclusion from inventory and applicable live backup jobs, retaining 9900.
Keep FUSE runs paused through backup; reapply and verify the exclusion before
a later run. No forced unmount, process kill or restart is authorized.
Permanent operation requires a separate backup-compatibility decision.

## Landing evidence

Only the e2e gate command gains the process-local profile PATH prefix.
Models, candidate gates, environment grants and deadlines are unchanged.
Sync changes the gate identity. Current approvals must not be reused.
The actual merged-tree landing gate remains required; the candidate's full
validation result is not a landing record.
