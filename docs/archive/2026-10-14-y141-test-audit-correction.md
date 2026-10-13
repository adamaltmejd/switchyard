# Y-141 static integrity correction

Candidate 0bf7b4a selected replacement holds from the requested model and
staged skill marker. An original-settings fault could miss the hold and
fail only at the fixture deadline. A model name in the handoff and an
admitted execution settings row did not prove the controller's actual
requested model. This corrects the qualification claimed in the prior
2026-10-13 record; that archive remains unchanged.

The existing g08_durable_package control now selects its three replacement
holds from a host-owned phase and request opening. No model, instruction,
skill or reference value selects a hold. The driver advances the phase
only while the prior completed publication is held by the pinfold wrapper.
The replacement's opening is held before checking tested request values.

After each independent hold the host asserts the request body's model is
packaged-adopted-model. The initial controller explicitly requests
fake-model. Settings-only replacement must carry the original skill;
staging-only and image-only replacements must carry the adopted skill.
Reference expectations also come from the host phase, never the observed
skill. All existing staged byte, Git/blob/context, package identity,
handoff, historical row, reservation/release, sandbox, bounded read,
publication and fresh-state assertions remain. The settings, staging and
image revisions and corrected stale CAS positive control remain separate.

## Test-audit Gate

- Row and clause: G2's held single-node packaged independent source
  adoption clause, with G6 committed inputs, G9 frozen launch settings and
  G13 exclusive ownership. The current spec already names this behavior.
- Sabotage: launch the replacement controller with the original attempt's
  model, or stage its original skill after staging adoption. Either reaches
  the independently selected hold and fails the host's actual request
  assertion. The test comment names these faults. Native fault injection
  and full host e2e remain post-approval obligations.
- Disposition and distinctness: correct the existing control in place.
  Its previous model-name prompt assertion proves handoff consumption,
  and its execution row proves recorded settings; neither observes the
  actual requested model. No new scenario or twin was added. The held
  publication and distinct source-only replacements retain the hardest
  existing case for packaged revision.
- External expectations: model names and phase transitions come from the
  declared host fixture. Skill/reference markers come from the fixture's
  selected committed bytes. Existing file and image expectations remain
  host Git values. No expectation branches on the request under test.

The diff replaces 27 test lines and zero binary lines. No assertion block
or product path was deleted. No deadlines, concurrency limits, retries,
sleeps or binary barriers changed. No whole-suite sweep was requested.

Validation: cargo fmt --check and cargo clippy --all-targets --locked --
-D warnings. No host e2e or runtime faults were run in this box. Full
packaged and parent integration acceptance remains open after this static
correction.
