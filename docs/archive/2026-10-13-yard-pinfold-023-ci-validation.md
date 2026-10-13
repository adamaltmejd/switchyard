# Yard Pinfold 0.2.3 CI qualification

The authorized dependency validation branch selects Pinfold 0.2.3 in CI
and README.md. Its qualified source is
3eef224d92d0ee9f79d480e68eec1b584349c9ce. It includes the landed Y145
checked-release observer repair and the earlier Rust, Cargo, Node, Durable
and Action pin refresh. Unlanded Y142 is excluded.

Both jobs in [CI38425784733](https://github.com/adamaltmejd/switchyard/actions/runs/38425784733)
passed formatting, locked all-target Clippy and the real native suite.
Logs confirm that exact source head and Pinfold 0.2.3.
Linux arm64 job115394699590 passed79/0 in400.34s.
Linux x86_64 job115394700034 passed79/0 in565.84s.
Their raw log SHA256 identities are respectively
e202d26e043f7a652559a71e1fc15a9c11160ce3e38664432f6ee9c0b7e06f17
and5b963167472fb4fa50b5c638f0d1ae4f402347b8df1b5007620229f584681d89.

Original CI38420517969 on6675b55 remains failed: arm64 passed78/79,
then compared a pre-release execution snapshot with its settled row after
restart. Usage and immutable result/session identities matched; reservation
and teardown changed. Y145 waits for the exact execution.released event,
preserving every usage and full-row reopen assertion. This is changed-source
qualification; no unchanged-source retry erased the failure.

The original exact Pinfold0.2.3 host run passed79/0 in593.48s.
Y145 actually landed8721dff after native79/0 in564.10s on the preserved
installed host runtime. The corrected G7 scenario on3eef224, using the
isolated verified Pinfold0.2.3 binary, passed1/0 in15.47s.
Product code and compiled Yard identity are unchanged by Y145 and the pin:
524c1a00510ff5a4b17344197d9903a6b0daf9ac53936d940e6014922c3380a4.
This archive is a report-only commit after the qualified source.

All release artifacts matched GitHub asset digests and SHA256SUMS as
recorded in the earlier pin archive. Full logs, artifact identities and
terminal manifests remain under
/home/yard/validation/pinfold-0.2.3-pin-verification-20261013.

No Mac result or original Pinfold89 resolution is claimed. The operational
Yard binary, daemon and reviewer remain unchanged. No installed Pinfold
replacement occurred. The full Y80 scope and its final matrix remain open.
