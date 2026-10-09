# Dependency update

Pinfold moved from 0.1.0 to 0.1.1 in CI and the README. The host's
release binary was replaced after its published SHA256SUMS verified.
`pinfold update --check` reported 0.1.1 current.

The [release](https://github.com/adamaltmejd/pinfold/releases/tag/v0.1.1)
adds an explicit updater and profile and image notices. Its box JSON
fields and exit codes are unchanged. Yard needed no adapter change.
Both releases carry Pi 0.99.1, Claude Code 2.1.285 and Codex 0.159.2.

The existing Cargo requirements resolve to the latest stable direct
dependencies. `cargo update` changed no package. Rust 1.98.1 matches
the stable distribution manifest, including the project image's pin.

`actions/checkout` moved from 5.1.0 to 7.0.1 in both workflows, pinned
to commit `3d3c42e5aac5ba805825da76410c181273ba90b1`. Its Node 24
runtime matches the previous action. These workflows use none of the
privileged pull request triggers whose defaults changed in version 7.
Upload artifact 7.0.1, download artifact 8.0.1 and rust-cache 2.9.2
were already the latest releases.

## Validation

On yard-sthlm with pinfold 0.1.1:

- `cargo fmt --check` passed.
- `cargo clippy --all-targets --locked -- -D warnings` passed.
- `cargo build --locked` passed.
- `pinfold build --profile default` passed.
- `cargo test -p e2e --locked` passed all 36 scenarios in 125.08 seconds.

The GitHub action changes were checked against their release source.
GitHub runners and macOS were not exercised here.

## Update propagation

Pinfold's [nightly workflow](https://github.com/adamaltmejd/pinfold/blob/v0.1.1/.github/workflows/bump-pins.yml)
checks harness pins daily. When automatic releases are enabled, changed
pins pass Linux and Mac gates before publication. This does not update
installed binaries or Switchyard's CI pin.

Pinfold downloads and mounts its pinned harness when a box needs it.
Yard reads the pins once at daemon startup. A host update therefore
needs closed pinfold boxes, `pinfold update`, then `yard daemon restart`.
A harness version change ends session continuity. Automatic version
bumps and host scheduling were not added in this update.
