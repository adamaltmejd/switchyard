# Y80 host request accounting and runtime requirement

2026-10-15. Operator disposition extends the existing final host-metrics/spec
pass. This adds measurement to the existing fixture; it creates no assertion,
fixture, model request, dependency or production execution path.

`FakeModel::drop` reads the length of its existing Arc/Mutex request Vec and
writes only `fake_model_requests=N`. A poisoned mutex still yields its stored
Vec; write errors are ignored so teardown does not introduce an I/O panic.
No body, header, credential, endpoint or port is printed. There is no second
counter, server shutdown protocol or lifecycle. The observation describes
requests recorded at fixture teardown, not a later server total. Ordinary
Rust unwinding drops the model on a failed test too; process abort or forced
termination cannot promise a teardown observation.

Use the already-required final candidate host run:

```sh
cargo test -p e2e --locked -- --show-output
```

Retain stdout and stderr together with exit status, elapsed time and raw-log
hash. Each FakeModel produces one count, including nested machines, zero-request
restore machines and failed tests whose teardown completes. Sum all observed
`fake_model_requests` values in that run for its recorded-request total; keep
failed-test contributions. The count uses a fallible direct stderr write,
so it is visible even when ordinary test output is captured; `--show-output`
also retains captured output from passing tests. Do not run an extra full
suite solely for accounting. Missing teardown output is missing evidence,
not an inferred zero.

Keep actual observed model requests distinct from static script-response
estimates and from machine, historical/current/debugger-build and box counts.
Neither Yard token rows nor script branches substitute for the recorded Vec.
The prior cleanup archive's static resource reductions are still estimates;
this change supplies no new runtime totals or elapsed savings.

README now requires Pinfold 0.2.3 for this release, matching the selected
integration and CI pin. The earlier 0.1.1-or-newer statement was unverified.
No old-version matrix or construction-runtime update is performed or implied.

Test-audit disposition: no assertion or sabotage claim changes. This is the
operator-authorized observation of existing fixture state; cost is one Vec
length read and numeric output per existing FakeModel drop. All G1-G15
controls, deterministic holds and allowed/refused scenarios remain intact.
Production source lines and test assertion lines changed: zero. Boxed fmt,
locked all-targets clippy and locked build are checked before commit. Host
execution and the repaired migration gate remain operator work. The earlier
migration host failure and its correction remain in the dated archives; no
new runtime pass or final release readiness is claimed here.
