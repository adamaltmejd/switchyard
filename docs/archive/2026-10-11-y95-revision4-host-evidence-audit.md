# Y-95 revision 4 host evidence audit

The complete original scope and the operator's actual-landing carry were
read. This is new evidence, not an update to either historical Y-95 record.
No code repair or permission for an unchanged host retry is claimed.

## Operator-supplied observations

Exact candidate `0175eddc2e9f73328ae4e494ed798d2445e5d0c0` passed
standalone focused13247, 1/0 in 29.44 seconds, and full50736, 70/0 in
406.68 seconds. Published review1731/check911 passed with no findings.
These are operator-supplied runtime observations, not source-only claims.
They establish the proof-bearing scenario on that head; they do not replace
the actual landing gate.

Approval98 did not complete. Landing1736/host1740 failed 68/2 in 447.74
seconds. Approval retired; canonical did not advance. Target remains
`e4ce7d57e5537679f78125a00f46a3ab4dc7d725`.

The publication-restart test failed before its scenario during `yard init`:
pinfold box-list for label `fe0f64e6ad` timed out. The older Y-98 27dd
fixture-init listing failure is also undiagnosed. Its later healthy probe
and standalone passing suites do not establish a cause or a fix.

The Codex containment test observed `red:ownership` for linked-skill Y-2,
execution4, instead of its expected `red:error` naming `.agents/skills`.
The event says `external owner not proved gone; capacity retained` and
`lock_contract=true`. CPU/IO pressure was observed on the host, but the
operator explicitly reports that it does not prove the cause.

## Additional source trace

`jobs/mod.rs::spawn` runs the review inside `capacity::effect`. Review
preparation creates its checkout through real Git and loads trusted agent
environment inputs. Codex argv construction calls `workspace_skills`.
`agent_env::walk` refuses a symlink in any root component, naming the path.
This refusal occurs before review box launch. These are source observations;
they do not prove the failed host fixture reached that refusal.

After work returns, `effect` drops its own host lock, then calls `release`.
Release requires both successful box-list absence and a free inherited lock.
On unknown ownership it raises ownership and effect returns the generic
ownership failure, rather than the work's original error. `spawn` records
that failure through its existing execution/error path. This explains why
a preparation error can be obscured by a failed ownership probe. It does
not identify which probe failed in the observed fixture, or prove that the
original error was the expected linked-skill refusal.

Keeping the reservation and refusing eligible teardown is required by G5.
Changing that guard merely to expose `red:error` would be wrong. No proof
handling, assertion, lifecycle, timeout, capacity or thread setting changes
are justified by these source observations.

## Access boundary and required evidence

This worker checked the actual available filesystem and tools. The original
host checkout and `/tmp/ye-3757721-g8-codex` fixture are absent. `/yard/proof`
contains no supplied diagnostic artifacts. Pinfold is not installed here,
and no host execution capability is exposed. No real host probe or new CLI
reproduction was run. No runtime or process state was guessed from source.

The operator was asked for the original box-list/host-lock outcome for
execution4 and the pinfold/runtime logs for the init timeout, or access to
those observations. Both probes must be distinguished before attributing
the ownership event. The existing parked proposal203 remains a diagnostic
proposal; no duplicate ticket is filed and it is not treated as completion.

Any repair must address a demonstrated cause at its own mechanism. The
unchanged proof code and all G8 assertions remain. No sleeps, retries,
deadline extensions, thread-cap masking, manual pruning, process kills,
fabricated frames, synthetic rows or acceptance of unknown liveness occur.
No .yard, model, reviewer or operational daemon change occurs.

Test-audit disposition: keep all tests and assertions. This source review
and evidence audit changes no assertion, guarantee or binary line. Test
lines removed: 0. Binary lines removed: 0. `cargo fmt --check` and
`cargo clippy --all-targets --locked -- -D warnings` passed before commit;
e2e is not run in this box.

Y-95 remains unresolved pending host diagnosis and demonstrated repair,
followed by fresh exact-head validation, published review and actual landing.
This audit commit cannot serve as the fix allowing an unchanged host retry.
Original parent and release scope remain required.
