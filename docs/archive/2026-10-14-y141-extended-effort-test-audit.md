# Y-141 extended effort correction

The supplied pinned SDK excerpts reveal a gap in c282787's effort path.
pi-ai 1.2.0 models.js accepts xhigh and max only when thinkingLevelMap
explicitly maps them. Without those entries, clampThinkingLevel selects a
different supported level. The controller now maps both levels to their
declared wire values. The existing settings-only phase adopts max from
initial medium, so its actual host request detects the clamp. Prior dated
records remain unchanged.

## Source and wire trace

The operator verified the public pi-ai and pi-durable 1.2.0 archives against
the committed lock integrity. The supplied model-schema declarations permit
xhigh/max string map entries. models.js:681-713 requires those entries before
clampThinkingLevel can preserve either level. pi-durable AgentState exposes
thinkingLevel; generation.js:119-132 sends it as reasoning unless it is off.
pi-ai's simple-options uses the model clamp. openai-completions.js:682-727
maps reasoning effort through thinkingLevelMap to OpenRouter reasoning.effort
or the supported OpenAI-compatible reasoning_effort field.

Yard's selected execution effort still enters audit.json and the inner root
agent. The custom model enables reasoning only for explicit effort. Omitted
effort leaves that capability disabled. The change adds no provider, route,
credential, grant, dependency or fallback. Low/medium/high use the SDK's
normal mapping; xhigh/max have explicit identity mappings.

## Test-audit Gate

- Row: G2's held packaged independent-source clause now observes initial
  medium and adopted max on actual OpenRouter controller requests. G6/G9's
  frozen-source and execution controls remain intact.
- Fault: remove the extended-level map, drop effort or launch predecessor
  medium while keeping the current model and handoff. Each opens the same
  phase-selected hold and fails the raw outside effort assertion. The test
  comment names the mapping fault and the independent effort faults.
- Distinctness and hardest case: max exercises the SDK's extended-level
  clamp in the existing settings-only replacement phase. Medium/high alone
  cannot detect that fault. The staging-only and image-only phases keep max,
  preserving their existing frozen-settings checks. No scenario or assertion
  is added or removed. Holds, deadlines and concurrency remain unchanged.
- Expectations: medium/max are literals committed in the fixture settings.
  ModelRequest.body is the parsed host HTTP request, not admission or prompt
  metadata. The expected field comes from the pinned OpenRouter wire mapping.

Disposition: retain every existing source-byte, Git/blob, context, package,
handoff, history, publication, ownership, CAS and sandbox assertion. Three
wire assertions and the adopted fixture value change from high to max.
Zero test lines and zero binary lines are removed. The binding current proof
clause names the actual selected values and the extended-level contract.

Validation: cargo fmt --check and cargo clippy --all-targets --locked --
-D warnings pass. Host e2e and independent runtime faults were not run here;
they remain post-approval gates. Full running packaged and parent integration
acceptance remains required. Stopped packaged activation remains out of scope.
