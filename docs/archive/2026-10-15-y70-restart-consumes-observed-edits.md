# Y-70 restart consumes observed edits

The native gate on 9eabcf6 reported 90 passing and three failing scenarios
in 1365.29 seconds. Both packaged-audit scenarios and the ordinary
edit/stop/start scenario stalled after resolving stopped attention. No
successor implementation started. This is a product regression in the new
pending-dispatch guard, not a timeout or fixture defect.

The guard used superseded_by_edit, which rejects both a changed ticket
revision and any unread edit or plan handoff. That is the correct predicate
for judging a candidate that has not read the new inputs. A deliberate
restart instead consumes those inputs. Its current implementation has not
read them yet, so edit_pending remains true until that restart starts.
The guard therefore prevented the execution that would consume the edit.

Pending dispatch now compares the current ticket revision and unread-edit
status with the scheduling observation. Already observed edits and plan
handoffs are allowed. A later ticket change, plan change or unread-edit
change still supersedes the observation before any lane or writer effect.
Exact live attempt, plan, choice, candidate, proof, lane, instructions,
selection eligibility and runnable ownership/attention guards remain.
Lane reservation and writer creation still share the existing transaction
under the admission lock. Ticket edits still retire superseded pending
instructions. Steering still validates the live choice.

Read together: ordinary attention-start publication of next; independent
writer attention-start and workspace-start admission; ticket-edit steering
and pending retirement; plan revision and derived edit/handoff detection;
pending scheduler admission; writer creation and frozen ticket body;
packaged startup on the execution's recorded plan revision. The ordinary
restart and packaged restart use the same pending scheduler path.

Existing native evidence supplies the regression coverage unchanged:

- G9 an_edit_reaches_the_next_prompt_and_stop_delivers_it_sooner holds a
  real worker, edits its body, stops it and deliberately starts it. It
  requires approval, the restart reason and the fixture's changed body in
  the actual host model request.
- G8 a_packaged_audit_refuses_missing_assets_then_publishes_in_a_corrected_box
  includes an actual historical source, retained execution and current
  source adoption. Its stopped controller must restart on the adopted
  package plan and publish through the native controller.
- G8 a_packaged_audit_bounds_iterations_and_requires_native_publication
  adopts a stopped owner's definition while its real inherited lock holds.
  After checked release, deliberate start must reach the fixture-held
  controller and native publication under the adopted revision.

These three tests already fail with the unconditional edit_pending guard.
Their fixture bodies, host model requests, controller publication, Git
sources and checked owner release supply the expected values. No assertion,
test or guarantee was removed or weakened. The five-producer held-scheduler
body-edit control remains unchanged and still requires obsolete B repair
to retire before five fresh sources and fresh exact B repair/landing.
No new timing scenario, retry, sleep or test-only barrier was added.

Validation: cargo fmt --check, cargo clippy --all-targets --locked --
-D warnings and cargo build --locked passed. Host e2e was not run as instructed.
The corrected candidate requires the operator's affected native and full
host landing gate after review. No .yard configuration changed.
