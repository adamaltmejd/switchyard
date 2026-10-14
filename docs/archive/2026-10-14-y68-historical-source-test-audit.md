# Y-68 historical source extraction audit — 2026-10-14

The required nullable migration case on ac2be863 failed while compiling its
real fb3f727 historical daemon, before creating the historical store. Its
archive omitted committed runner assets embedded by pi.rs. The failed native
evidence remains at
/home/yard/validation/y68-ac2be8-migration-20261014-evidence.

The shared archived_binary helper now includes committed runners when that
directory exists in the requested historical Git tree. This follows the
source dependency list in g08_durable_package's historical archive helper.
The pre-target c85369d tree predates runners, so unconditionally requesting
that path would break its archive. Git tree lookup failure is a setup failure,
not an inferred absence. No source bytes or historical binaries are replaced.

All include_str! and include_bytes! invocations were inspected across both
complete historical trees. Their external dependencies are:

| Historical revision | Included source | Archive coverage |
|---|---|---|
| c85369dfaec8f8554da49ddf02a4dbec03bcad02 | Three daemon skill Markdown files; pi-mcp-extension.ts beside pi.rs | Existing .agents/skills and crates paths. No runners tree exists. |
| fb3f727f793524534965480f86e71af06b0ad646 | The same skills and extension, plus durable-audit launch, runner.mjs and package-lock.json | Existing paths plus that revision's committed runners tree. |

The nullable migration scenario was read end to end. It still uses the real
historical daemon to create a fresh store and run real proof publication,
verification, approval, landing and cleanup. It still compares every
historical execution column, including results and owners, and all retained
attempt/check/approval/attention/audit/plan rows through migration and a
second reopen. Only the new scope field is expected null. Host Git supplies
the landed head; the fixture supplies proof bytes. Foreign-key checks remain.
No receiving-schema initialization or synthetic historical rows were added.

The other shared-helper users retain the actual c85369d binary: ended planner
proposals and limit overrides, held legacy pre-box ownership, nullable usage,
and unbound read-only planner recovery. Their historical source, assertions,
positive controls, held crash points and restart order are unchanged.

## Assertion disposition

No guarantee assertion block changed. The added Git-command success check is
fixture setup validation. Existing G5/G6 nullable-migration sabotage remains:
omit a column, remap identities, lose foreign keys or invent historical scope.
The pre-target cases cannot establish preservation of Y-67's bound results
and owner records. Expected historical values still come from the actual
old store; immutability is the asserted relationship. No twin was introduced.
The helper diff replaces 9 test lines; 0 binary lines were removed.
ARCHITECTURE.md's current contract and all production code remain unchanged.

Validation: cargo fmt --check, strict locked all-target clippy, locked build
and git diff --check passed. Both historical revisions were extracted into
fresh temporary directories using the corrected archive selection. Every
included skill, extension and runner file matched its historical Git blob
identity. The actual fb3f727 daemon compiled with locked offline dependencies.
The c85369d locked offline build stopped because this local cache lacks
futures-channel; it was not a source include failure and is not a build pass.
No dependency was downloaded or prerequisite changed. These source checks
neither open a project nor qualify migration or historical runtime behavior.

Native qualification remains required on the repaired head, followed by the
full host landing gate. Earlier standalone, held-down-owner and shared-image
passes do not qualify this head. No native e2e or sabotage run, runtime change
or .yard configuration change was made here, as instructed.
