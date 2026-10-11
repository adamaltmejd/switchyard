# Y-94 operator validation and failed landing

The operator supplied these observations in the ticket edit on 2026-10-11.
They are historical evidence for candidate
ba8a271d1f0d3a9ffe65a340c534e39c0b3ada56, not independent observations
made in this worker and not validation of its later image-isolation repair.

- Exact-head focused host run: 1 passed, 0 failed, 52.44 seconds.
- Exact-head full host run: 63 passed, 0 failed, 322.00 seconds.
- Boxed checks 825–827 passed. Published review 1579/check 832 had no findings.
- Approval 93 bound the exact head and proof.
- Actual landing 1585 passed fmt 1586, clippy 1587 and build 1588.
- Actual host gate 1589 failed: 62 passed, 1 failed, 304.35 seconds.
  The only failing scenario was inherited G8
  `a_gate_errors_start_reruns_that_gate`.
- No landing.recorded, consume or Y-95 unpark occurred. Approval 93 retired,
  the landing ended red, and Yard started repair 1590.

The G8 failure happened while its helper waited for the initial read-only
attempt.ended. Fixture execution 1 raised red:error during image build,
before the selected gate-error assertions. Pinfold reused cached image
c483175b78b397664d637de5345e5e23defd7fd970bd006f7ec56162d92f7f47,
then COMMIT pinfold/image-yard-ddfdc48ce7 failed to locate that image ID.
This establishes an actual runtime precondition failure. It does not prove
an omission defect or identify the destructive image-lifetime operation.
The earlier green suite does not make the later red landing successful.

The fixture repair in 67b9cbd is described and gated in
2026-10-11-y94-gate-error-image-isolation-test-audit.md. It follows the
ordinary committed machine-specific Dockerfile content and supported sync
already used by Y-77. All G8 gate-error and Y-94 omission assertions remain
intact. No retry, sleep, skip, capacity change, fake runtime flag, production
accommodation or manual runtime deletion was added. The full admitted
omission scope and dependency Y-60 are unchanged.

Fresh exact-head focused and full operator host runs, published review and
actual landing gates remain required for the repaired candidate. No local
host e2e run or repaired-head host pass is claimed. This evidence-only
addition changes no assertion or binary line and leaves prior archive
records unchanged. Static checks passed: cargo fmt --check and
cargo clippy --all-targets --locked -- -D warnings.
