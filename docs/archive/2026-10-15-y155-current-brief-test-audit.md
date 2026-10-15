# Y-155 resumed candidate fixture repair

The operator deliberately interrupted the complete focused native run at
exact candidate 7551159a. It was neither a pass nor an asserted test failure.
Execution 22 resumed session 5 and received `Changed candidate fixture.`.
The fixture read request.prompt(), which returns the first user text. It
wrote the original bytes, Git had nothing to commit, and the scenario waited
for approval of a new head that the fixture could not produce.

Read request.last_user() only for the changed-body decision. Keep the real
resumed session and expected changed bytes and head. The remaining prompt
uses in this callback identify the stable Graph project, declared reviewer
instructions and profiles, or frozen selector inputs. They do not decide
the current ticket body. Keep those reads unchanged. Tool-result counts and
last_tool_result() already read the selection publication exchange.

Test-audit Gate disposition: retain all G2/G8/G13 assertions and the existing
ownership, restart and native landing observations.

- A changed candidate must use fresh exact review and selection inputs.
  Reusing historical selection evidence can authorize effects on the wrong
  candidate. The repaired fixture reaches this existing hard case.
- Existing sabotage comments name selection by prose, index or latest
  result, lost contributors, erased raw findings and prior-selection reuse.
  No assertion or expected value changes.
- Use the same scenario, machine, real boxes and model turns. Add no fresh
  session, empty commit, sleep, retry, deadline, barrier or concurrency limit.
- The edited ticket body supplies the fixture decision. Git supplies the
  changed head and exact landed bytes. Public store rows and audit history
  supply exact source identities, immutable old rows and publication counts.

One test line replaced and one intent comment added. Zero assertion lines
and zero binary lines removed. No product control, Shown by, operational
configuration or .yard change. Prior watcher and historical-row repairs stay.

The operator retains native.log, terminal, confirmed-stalled-store.sqlite,
execution22-evidence.jsonl and operator-interruption.json under
/home/yard/validation/y155-755115-join-20261015-evidence. This record uses the
supplied diagnosis without claiming independent verification of host files.
The evidence and earlier failure records are untouched.

Worker validation passed: cargo fmt --check, cargo clippy --all-targets
--locked -- -D warnings, and cargo build --locked. Native e2e is deferred to the
operator. The complete affected case must run once at the exact repaired
candidate before another full prepared host qualification.
