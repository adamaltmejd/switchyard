# Y-141 full-settings correction

The blocked review found that effort reached the outer Pi argv but not the
packaged controller. audit.json now carries the execution's selected effort.
The runner validates null or Pi's admitted low/medium/high/xhigh/max ladder,
passes a declared value as the inner agent's thinkingLevel, and enables the
model's reasoning capability. Null omits thinkingLevel and leaves reasoning
disabled. Invalid values refuse rather than becoming unused metadata.
The outer nonempty placeholder and Pi's check_prompt stay intact.

## Pinned API and observer trace

The operator verified packages/26.tgz and packages/24.tgz against the exact
locked SHA512 and version 1.2.0. This correction uses that supplied source
verification; this box could not download the archives independently.
pi-durable's dist/harness/types.d.ts exposes thinkingLevel on AgentState,
AgentChange and Agent. pi-ai's openai-completions implementation maps thinking
to OpenRouter reasoning.effort when thinkingFormat is openrouter and the model
has reasoning capability. Its supported OpenAI-compatible path sends
reasoning_effort. The runner selects the former for openrouter and the latter
for opencode-go, with supportsReasoningEffort enabled. No dependency changed.

The assertions read ModelRequest.body directly. crates/e2e/src/model.rs parses
the raw host HTTP request with serde_json::from_slice; no effort helper derives
the observed field from admission, model name, prompt or handoff. The fixture
commits initial medium and adopted high in agent settings. Those constants,
and the pinned OpenRouter wire contract, supply the expected values.

## Test-audit Gate

- Row and clause: G2's held packaged independent-source adoption now observes
  initial medium and adopted high effort in actual host controller requests.
  The existing G6/G9 omitted-source and frozen-execution controls remain.
- Sabotage: drop inner effort or launch predecessor medium while preserving
  the adopted model and current handoff. The existing independently
  phase-selected replacement hold opens and the raw effort assertion fails.
  The test comment names this independent fault.
- Distinctness and hardest case: extend the existing settings-only adoption
  after a published controller is held with ownership retained. Model,
  instruction-prefix and execution-row assertions cannot detect dropped
  effort. Staging-only and image-only requests retain high, proving those
  separate source selections keep the adopted settings frozen. No scenario,
  latch mechanism, sleep, retry, deadline or concurrency setting changes.
- External expectations: committed fixture medium/high values and the traced
  provider wire field. Holds remain selected by host phase, independently of
  model, effort, instructions and selected markers.

Disposition: keep the four wire assertions and the initial/adopted fixture
settings. Preserve every existing identity, source-byte, handoff, publication,
history, CAS, sandbox and checked-release assertion. This correction removes
zero test lines and replaces four binary lines. Prior archive claims remain
unchanged. No guarantee is narrowed.

Validation: cargo fmt --check, cargo clippy --all-targets --locked --
-D warnings and cargo build --locked pass. Node is unavailable here, so no local JavaScript execution
was claimed. Host e2e and independent runtime faults, including the effort
fault above, were not run and remain post-approval obligations. Full packaged
and parent ordinary/packaged integration acceptance remains required.
