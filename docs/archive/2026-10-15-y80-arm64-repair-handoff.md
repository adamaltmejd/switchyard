# Y80 arm64 repair candidate handoff

2026-10-15. Product/test/spec source is
`8e6f4c7068d2a08ecf5a65b2b8a707daf327ca85`. The following evidence-only
commit adds this record. Repairs are bounded to jobs (1176c3f), the existing
G2 scenario (fcf0e7f), and the existing G9 scenario (8e6f4c7).

The [G2 record](2026-10-15-y80-arm64-g2-repair.md) retains the exact failed
arm64 gate, passing x64/native gates and their supplied identities. The
[G9 record](2026-10-15-y80-arm64-g9-observer.md) records the premature
continuation boundary. Neither old passing gates nor successful asset jobs
qualify this changed source. No unchanged retry, host e2e or sabotage ran
in this worker box.

Passed here: `cargo fmt --check`,
`cargo clippy --all-targets --locked -- -D warnings`, `cargo build --locked`,
`cargo build -p yard --release --locked`, and `git diff --check`.

The boxed GNU artifact is staged separately at
`/yard/proof/y80-artifacts-8e6f4c7/yard-0.0.5-x86_64-unknown-linux-gnu`.
It reports version 0.0.5, boundary 34555c8749eca77c. SHA256:
`34555c8749eca77c67d38198d16d29f9862f50a614046c379c24fc0966ea398b`.
Compiler: rustc 1.99.0 (b940084d7), LLVM 23.1.1,
x86_64-unknown-linux-gnu. This is a boxed build artifact, not a qualified
static Linux or Mac release asset. Earlier artifacts remain separate.

Relative to 06e3bae, before this evidence-only file: production Rust adds
9/deletes 1 line; G2 adds 54/deletes 0; G9 adds 8/deletes 1;
ARCHITECTURE adds 6/deletes 2. These repairs add no machine, historical or
debugger build, box, scripted response or model request. Actual full-suite
HTTP counts still require the existing FakeModel teardown observations
with `--show-output`; no count or elapsed saving is inferred here. The
original cleanup measurements remain dated historical evidence.

Operator validation still requires isolated old-guard G2 falsification,
restored-guard G2 and G9 execution, then fresh exact-source full platform
and artifact qualification. Preserve failures and observe model-request
counts at each fixture teardown, including nested machines and failures.
Original real 0.0.3 upgrade/separate old restore, model trials/statistics,
cleanup/spec, Mac runtime/security and release handoff requirements remain.
No release readiness, installation, publication or construction switchover
is claimed. No .yard or installed runtime was changed.
