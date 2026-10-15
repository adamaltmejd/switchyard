# Y-70 automatic picker admission

Review3382 at 43ebe0b identified stale automatic scheduling on both sides
of picker execution. Activation could overwrite a newer exact operator
choice. Startup could append a picker after that choice. Downstream binding
requires the latest selector execution to equal the chosen execution, so
the unnecessary row could prevent selected gates from starting.

Automatic activation and startup now share one existing project transaction.
It rereads the live attempt and requires the observed choice, candidate,
proof and plan revision, no superseding ticket edit, no pending continuation
and no blocking attention. Current eligible declared inputs must equal the
observed inputs. The latest selector execution must still be the observed
one. Running or reserved selectors still block scheduling. For an ended
selector, authoritative current selection eligibility is checked inside the
transaction. An eligible result is activated there; otherwise a new picker
is created there. An obsolete observation returns without activation, row
creation, choice invalidation or approval withdrawal.

The old outside-transaction activation decision was removed. The existing
early running/reserved check remains, and the transaction checks current
ownership again. The attention guard preserves explicit retry after picker
failure if attention appears after the scheduler's initial read. Current
input binding remains the sole source of picker input eligibility. No new
selection owner, reservation or lifecycle was introduced.

Read together: writer scheduling and named-reader admission; automatic
activation and startup; explicit operator identity checks and selection
activation; current selection result/input eligibility; execution binding's
latest-selector requirement; selected gate admission and automatic repair.
The fieldwise gate identity and exact named-reviewer producer/result guards
from 43ebe0b remain unchanged. Picker history and raw results are untouched.

No test assertions changed. No external held point in the micro-window is
demonstrated, so no timing race, barrier, sleep or retry was added. Existing
repeated worker selection, operator choice, source-change and explicit failed
picker retry controls remain the allowed native qualification paths.

Validation: cargo fmt --check, cargo clippy --all-targets --locked --
-D warnings and cargo build --locked passed. Native/e2e was not run as
instructed. Affected native
qualification and the full host gate remain operator-owned after review.
No .yard configuration changed.
