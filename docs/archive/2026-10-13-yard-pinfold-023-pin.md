# Yard Pinfold 0.2.3 pin

The dependency validation branch now selects Pinfold 0.2.3 in CI.
README.md states that same current pin. Other dependency refresh pins remain.
The source baseline is be68df5b83010322bd9e0d5a225fcc1ea59b3ff3.
Unlanded Y142 changes are excluded.

The upstream annotated tag object is
0e4fb51cdfdd96fbde60b87e1c341286ed65a9ff. Its source commit is
4b8141cedd6b63e118b39fe6b10027e8075777a3.
All four release assets match GitHub asset digests. The three binaries
also match the release SHA256SUMS file. The isolated Linux x86_64 binary
reports `pinfold 0.2.3`.

Linux x86_64 SHA256:
d89172077c000759fab378b3cd9b515b5fe9cd4321294bd98cdc9d2e720fec18

Linux arm64 SHA256:
b1385581e9fdfd1e97e2c6b5839fb3b2496e171631151b6d1e0f3b4a27c0606d

Mac arm64 SHA256:
194c5bb126a8b7e8cbd58dc9a7a65b33b2a19f87e5206816df2237f6ceeb62a3

`cargo fmt --check` and `cargo clippy --all-targets --locked -- -D warnings`
passed. The previous dependency CI run tested 0.2.2 and does not qualify
this pin. A fresh exact-head Linux x86_64/arm64 run is required.
The installed host Pinfold remains 0.2.2. Yard's operational binary and
daemon are unchanged. No Mac validation or Pinfold issue resolution
is claimed by this update.

Verification manifest and isolated artifacts:
`/home/yard/validation/pinfold-0.2.3-pin-verification-20261013/manifest.json`.
