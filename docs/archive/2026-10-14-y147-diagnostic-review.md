# Y-147 diagnostic review

The review's P3 diagnostic-gap finding is fixed. A missing `/proc/locks`
read now sets `incomplete` to true. This covers open/read failure and the
256 KiB input cap. Failure to read the lock file's metadata also explicitly
reports `incomplete: true`. Neither case is evidence that no lock exists.

Test-audit Gate disposition: no assertion block is added, removed or changed.
G2, G5 and G7 coverage and their expected values, held points and sabotage
comments remain unchanged. Binary lines removed: 0. Fixture lines removed:
1. Assertion lines removed: 0. No new guarantee coverage is claimed.

The P1 ticket-not-met finding remains valid. The diagnostic candidate cannot
be approved as the restart fix. No host lock snapshot or actual errno was
provided with the review; the retained host evidence directory remains
unavailable in the worker. The original ticket requires these observations
before choosing a fix and prohibits running host e2e inside the worker.

The operator must obtain the actual failing-host errno/dev/inode, holder
PID/start identity and matching descriptors before the fix attempt resumes.
That checkpoint must preserve Y-147's restart, live-writer refusal and
containment requirements. Diagnostic approval cannot close Y-147.

Validation passed: `cargo fmt --check`,
`cargo clippy --all-targets --locked -- -D warnings`, and
`cargo build --locked`. Host e2e was not run, as instructed. These checks
validate the preparation's build, not the unresolved restart behavior.
