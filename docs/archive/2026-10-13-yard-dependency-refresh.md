# Yard dependency refresh, 2026-10-13

User requested current Yard-owned pins and a remote CI validation branch. Pinfold-owned updates remain with the user. Source includes verified landed Y134 and Y135; this is not final Y80 release readiness.

| Pin | Before | Updated |
|---|---|---|
| Host and boxed Rust | 1.98.1 | 1.99.0 |
| Durable, Pi AI, Chord, telemetry | 1.0.4 | 1.2.0 |
| upload-artifact | v7.0.1 | v7.0.2, cf430e030ddbb5b0abf93d22962f4752f3646cd9 |
| download-artifact | v8.0.1 | v8.0.2, 9000827ccba6bdab643e8b6fd33ac0654aef8333 |

Cargo refreshed 16 compatible entries and removed an unused futures-channel entry. Node 26.11.1, checkout v7.0.1, rust-cache v2.9.2 and released Pinfold 0.2.2 were already current when checked. Both official Node archive checksums matched. Exact package and lock identity guards remain; package scripts stay disabled and installed assets sealed. No new direct Rust dependency.

Durable 1.2 changed ToolExecutionResult from content to output. The first full native run failed 78/79 in 550.74s because the scripted model could not find a read result. That failed log is preserved. The controller now uses output and explicitly allows the JSON encoding of a bounded 32768-byte read, without changing the read, wire, request, tool or publication bounds. No test assertions were removed or weakened for this update.

Future .yard host gates use a separately copied/hash-verified 1.2 input archive cache. The old shared cache and active admitted inputs were preserved.

Fmt, locked all-targets clippy with warnings denied, locked build and the actual copied Rust image build passed. The image reports Cargo 1.99.0 and Clippy 0.1.99. The corrected combined source passed all 79 native host scenarios in 788.26 seconds. The first failed run remains failed and retained. Remote CI is not dispatched at this report point; no arm64 or Mac pass is claimed. No operational installation occurred.

These indirect instances remain constrained by their upstream parent versions/ranges. No unsupported major-version overrides were added:

| Ecosystem | Package | Resolved | Upstream latest at audit |
|---|---|---|---|
| rust | hashbrown | 0.16.1 | 0.17.1 |
| rust | r-efi | 6.0.0 | 7.1.0 |
| rust | rsqlite-vfs | 0.1.1 | 0.2.0 |
| rust | sqlite-wasm-rs | 0.5.5 | 0.6.1 |
| rust | wasi | 0.11.1+wasi-snapshot-preview1 | 0.14.7+wasi-0.2.4 |
| rust | windows-link | 0.2.1 | 0.100.0 |
| npm | @anthropic-ai/sdk | 0.129.0 | 0.133.0 |
| npm | @aws-sdk/client-bedrock-runtime | 3.1127.0 | 3.1150.0 |
| npm | @aws-sdk/token-providers | 3.1127.0 | 3.1150.0 |
| npm | @babel/runtime | 7.29.10 | 8.0.7 |
| npm | @google/genai | 2.21.0 | 2.29.0 |
| npm | @stablelib/base64 | 1.0.1 | 2.0.1 |
| npm | @types/retry | 0.12.0 | 0.12.5 |
| npm | bignumber.js | 9.3.1 | 11.1.5 |
| npm | data-uri-to-buffer | 4.0.1 | 8.0.0 |
| npm | diff | 8.0.4 | 9.0.0 |
| npm | fetch-blob | 3.2.0 | 4.0.0 |
| npm | gaxios | 7.3.1 | 8.3.0 |
| npm | agent-base | 7.1.4 | 9.0.0 |
| npm | https-proxy-agent | 7.0.6 | 9.1.0 |
| npm | gcp-metadata | 8.1.2 | 9.2.0 |
| npm | google-auth-library | 10.9.1 | 11.3.0 |
| npm | google-logging-utils | 1.1.3 | 2.1.0 |
| npm | node-domexception | 1.0.0 | 2.0.2 |
| npm | openai | 7.19.0 | 7.32.0 |
| npm | p-retry | 4.6.2 | 8.0.1 |
| npm | protobufjs | 7.6.6 | 8.8.1 |
| npm | typebox | 1.3.27 | 1.3.37 |
| npm | undici-types | 8.9.0 | 8.11.2 |
| npm | web-streams-polyfill | 3.3.3 | 4.3.0 |

The installed Yard and Pinfold hashes remain unchanged. Operational daemon/reviewer replacement is outside this dependency work. Installation should follow complete Y80 validation and migration/restore readiness. Mac validation is guide-only; no SSH connection is authorized.


Validated code and asset source: `450f75136bdef51099e7177e6d7b0e394e81ede2`. The following report commit changes only this archive. Host raw log SHA256: `63b9d752c7b3998addf9ec57f46c683d1e4f0bf1974840c913b840a9391bea13`. Compiled Yard SHA256: `597600b001013b663430c70d5e5a81c2c6bab37635b31e3e5d5381169798581b`.

Upstream references: [Rust 1.99.0](https://blog.rust-lang.org/2026/10/01/Rust-1.99.0/), [Node release index](https://nodejs.org/dist/index.json), [Durable 1.2.0 registry metadata](https://registry.npmjs.org/@earendil-works/pi-durable/1.2.0), [upload-artifact release](https://github.com/actions/upload-artifact/releases/tag/v7.0.2), [download-artifact release](https://github.com/actions/download-artifact/releases/tag/v8.0.2).
