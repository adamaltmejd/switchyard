# Y-92 packaging inputs and assertion gate

Revision 3 supplied scoped package coordinates and official v1.0.4 APIs.
The public archive decoded to 40805 bytes and SHA256
`1bb203c632ece6abe5a6d63bdd5fa2eed83a5fd4914b708bd98a69fec5746d7d`.
Every member was a regular relative file; no link, absolute path or traversal
member was extracted. Extraction used candidate scratch, not a host grant.

The package.json and package-lock.json are copied unchanged. Lock SHA256 is
`38d2d7a889d74d6e1863ea2a2be86b5ca8ed0ab9e1cef0b288195e6fe12ab33e`.
The lock records 115 package entries and all four Earendil-family pins at
1.0.4. Node 26.11.1 checksums and the supplied MIT license are copied.
Primary sources are the official Pi v1.0.4 Durable README, Pi AI README
excerpts and metadata exports supplied by the operator. The historical
blocker archive was not edited.
The verified original public archive is retained as
`/yard/proof/y92-public-build-inputs.tar.gz` for candidate review. It is
proof data, not an installed runtime bundle or a merged repository asset.

## Assertion gate

Read the test-audit skill and its sweep tags. Read each new block in
`g08_durable_package.rs` against G8/G12/G13 and existing G8 publication and
G13 mount scenarios. No old assertion was removed or weakened.

| Disposition | Clause and sabotage | External expectation and distinct mechanism |
| --- | --- | --- |
| Keep | G8 pre-call refusal; skip installed identity checks | Fixture deletes a sealed entrypoint after build; the host route observes zero requests and the stored report is absent. Corrected image publishes fixture content in the same project. Ordinary Pi tests do not use Durable packaging. |
| Keep | G8/G13 bounded declared reads; remove size bound or allowlist | Oversized fixture supplies content; spec supplies 32768 bytes and undeclared_file. Allowed source and selected trusted reference succeed in the same conversation. Ordinary harnesses expose different read tools. |
| Keep | G8 candidate code exclusion and trusted skill; install CodingTools or load candidate code | Fixture plants executable and extension sentinels and asks for an unavailable shell tool. Host model observes its selected skill/reference, and guest observes no sentinel. This exercises the trusted Durable registry, not Pi discovery flags. |
| Keep | G12 route/guest/state/log/report containment; put key in boxed client or persist it | Host fixture supplies the key and receives it in Authorization. Guest env/source/state/package probes, model body, logs and stored report lack it. Durable checkpoints are a separate persistence mechanism from Pi sessions. |
| Keep | G13 source/trusted read-only and distinct writable paths; change fixture mounts or state writability | Guest writes to source/trusted fail; scratch and checkpoint writes succeed in the same box. Spec and fixture supply paths and permissions. This verifies fixture inputs, not Y-93 staging. |
| Keep | G8 bounded internal turns; remove request cap | Fixture requests another read on every turn; host route sees exactly the spec's 12 requests and log names audit_limit. A fresh explicit publication in the same project controls it. No timeout supplies a pass. |
| Keep | G8 bare-answer refusal; synthesize report from text | Fixture supplies a terminal answer; stored report stays absent and controller names missing_report. A fresh native MCP publication in the same project controls it. This checks the controller, not daemon eligibility. |

The host fixture replaces the guest command through the external pinfold
seam. The production controller runs in a real daemon-owned box with the
real execution bearer, injecting model route and native daemon MCP. No
fake Pi frames, daemon selection or eligible result is claimed. The current
Pi reader's infrastructure attention is only a completion wait; assertions
observe controller reasons, route traffic, guest effects and publications,
not attention prose. No crash scenario, sleep, retry or timeout wait is added.

Host setup supplies `YARD_E2E_DURABLE_INPUTS` with the official target Node
tar.gz and `packages/` holding the complete locked public npm archives.
Tests copy them into isolated image build contexts. No package installation
or evaluation runs on the host. Missing inputs fail setup; tests do not skip.

Test lines removed: 0. Rust binary lines removed: 0. No Rust runtime change.
No e2e suite or packaged runtime was executed in the candidate. Acceptance
remains the operator's exact-head isolated host package tests, whole suite,
fresh review and actual landing gates. Source/scaffold availability and
Rust checks alone do not establish that acceptance.

`cargo fmt --check` and
`cargo clippy --all-targets --locked -- -D warnings` passed. Shell syntax
checks passed; supplied package.json and lockfile compare byte-for-byte.
JavaScript syntax checking is part of the isolated image build and was not
run here because this candidate box has no Node runtime.
