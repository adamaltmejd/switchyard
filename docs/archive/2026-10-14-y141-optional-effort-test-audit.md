# Y-141 preserved-package optional effort correction

The operator found that the new daemon always added effort, including null,
to audit.json. The original sealed runner accepts exactly the six original
fields. Adding even null would therefore refuse an older admitted package
with no effort setting. Recorded package identity intentionally survives
daemon upgrades; changing it to the current runner is not a repair.

The daemon now inserts effort only for a selected non-null setting. The
current runner accepts exactly the original fields or those fields plus
effort. An explicit effort must be in the accepted ladder; null and other
invalid values refuse. Absence disables reasoning and omits thinkingLevel.
Explicit settings still reach the inner agent through the pinned SDK path.
No hash catalog, capability guess, proxy or fallback is introduced.

pi::valid_package_identity and the native bridge's admitted hash checks are
unchanged. Older sealed controllers receive their original envelope when
effort is omitted. Their strict schema refuses an explicit effort they cannot
control. Nothing swaps or relabels the recorded package.

## Test-audit Gate

- Row and clause: G8's corrected sealed-controller publication control now
  observes omitted effort with disabled reasoning on its real host request.
  Its existing report and result eligibility assertions prove completion.
- Fault: require effort in the current runner, inject null from the daemon,
  or enable reasoning for absence. The first two fail the existing successful
  publication control; the last fails the outside provider request assertions.
  The comment names those faults. G2's explicit medium/max assertions remain.
- Distinctness and hardest case: extend the existing corrected control after
  the sealed-controller integrity refusal. Explicit-effort replacements do
  not observe absence. No new scenario, deadline, hold or concurrency change
  is needed. No key-layout or internal serialization assertion is added.
- External expectations: this control's committed fixture config has no
  effort. The host parses actual HTTP requests; disabled reasoning sends no
  OpenRouter reasoning object or OpenAI reasoning_effort field. The fixture
  report and native eligibility remain the successful outcome observers.

Disposition: keep all model/instruction/effort/source/ownership/history/CAS
controls and previous evidence. Two provider-wire assertions are added to
the existing control; zero test lines are removed. The original sealed
runner's strict schema is the supplied source basis for preserving its
envelope. This does not claim a historical-package runtime was executed here.
All earlier dated records remain unchanged.

Validation: cargo fmt --check and cargo clippy --all-targets --locked --
-D warnings pass. Host e2e and independent runtime faults remain post-approval
gates, including older sealed launch qualification. Full running packaged,
older sealed and parent integration promises remain required.
