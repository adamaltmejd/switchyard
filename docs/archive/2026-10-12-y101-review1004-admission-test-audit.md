# Y-101 review 1004 scheduled input refusal repair and test-audit

Review 1004 found that a ready packaged ticket's selected-input refusal
escaped `scheduled` through `jobs::step`. That ended the tick before queue
and cleanup selection. The same malformed ticket could prevent independent
admission on every subsequent tick.

## Source repair

Scheduler preparation now leaves a ticket unadmitted when `Loaded::inputs`
returns `refused`, as the existing per-ticket target boundary does for an
unavailable target. It continues preparing other admissible tickets. No
attempt, execution or reservation is written for the refused ticket.
Internal Git/store failures still propagate. Explicit `attempt start`
continues returning the original selected path, `invalid_input` token and
regular-file or size reason. The refusal is not recorded as successful work.

The admission transaction and its revision, readiness and capacity rechecks
are unchanged. Queue and cleanup still use their existing selections after
scheduled admission. No lifecycle, cache, retry or performance mechanism was
added. Review 996's scoped canonical reads, independently checked selected
identities and authoritative recorded package identity remain intact.

## Assertion audit

The existing missing-assets/corrected-image package scenario gains one
admission-boundary block. Its completed replacement and corrected Y-99
isolation blocks remain. The bounds/bare-answer scenario is unchanged.
Test lines removed: 0; binary lines removed: 2. No existing assertion or
guarantee clause was weakened. This is a scoped test-audit gate, not a suite
sweep.

| Clause and disposition | Sabotage | Distinct hardest case | External oracle |
|---|---|---|---|
| G8: selected canonical regular-file and size refusals leave higher-priority malformed ready tickets without attempts, executions or reservations while an independent valid selection publishes and completes cleanup in the same project; explicit start returns each refused path and reason. Add to the existing package scenario and G8 Shown by. | Propagate `Loaded::inputs` refusal from `scheduled`, or allocate work before validating selected inputs. | Keep both a canonical linked skill and an oversized reference ready at P0 while a separate valid packaged ticket at P2 publishes and completes cleanup in that project. A symlink points to an existing valid skill; the reference exceeds the spec's 1 MiB bound by one byte. | Fixture-selected paths and reason tokens, fixture byte length, host Git blobs for the valid selection, fixture schema-valid report, CLI refusal and ticket state, read-only store attempt/execution rows and identity-bound cleanup history. |

Earlier package refusals run inside admitted executions. They cannot catch
an input refusal aborting admission before any execution exists. The new
block proves independent admission and cleanup with malformed tickets still
open, rather than parking the bad tickets to permit the positive control.
No expected path, blob, report or absence comes from a copy of Yard's logic.
The external store shows no attempts for the bad tickets and no joined
executions; without an execution there is no execution reservation.

Both complete scenarios retain deterministic boxed controller execution,
native registration/session and publication, no outer Pi model call, selected
instruction authority, secret probes, immutable input and image identities,
checked teardown and capacity controls. Existing assertions still cover
missing/corrected assets, declaration and report bounds, bare answers and
the Y-99 execution isolation holds. The new block uses the same real daemon,
Pi, image and scripted host model, and waits on status history for cleanup.
No sleeps, retries, binary barriers, timeout or concurrency masking, mocks
or public model requests were added.

ARCHITECTURE.md now states this per-ticket boundary and its G8 observation.
No target merge was needed; canonical Y-95 additions remain. Earlier archives
and installed `.yard` are unchanged. The canonical-mutex remark is not a
measured performance defect and prompted no scheduler rework. Y-100/Y-93
retain their complete integration acceptance, including the dependent
supervision child's outstanding acceptance.

## Validation

`cargo fmt --check`, `cargo clippy --all-targets --locked -- -D warnings`
and `git diff --check` passed in the candidate box. Clippy compiled both
complete package scenarios, including the new assertion block. Host e2e was
not run by the worker; the extended scenario is operator-owned after
approval. Source inspection and compilation do not establish a host runtime
pass.
