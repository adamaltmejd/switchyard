# Y-54: direct Anthropic in Claude

The ticket authorizes direct Anthropic authentication for the existing
Claude adapter. The selected reviewer is `claude-opus-5-5` with medium
effort. This change does not configure the construction project or claim
that the host account has access to that model.

## Protocol contract

The adapter targets the repository's Claude Code 2.1.283 pin and the
Anthropic Messages API version `2023-06-01`:

- Host origin: `https://api.anthropic.com`, messages at `/v1/messages`.
- Claude reads `ANTHROPIC_API_KEY` and `ANTHROPIC_BASE_URL`. The guest
  receives `yard-placeholder` and `http://anthropic.yard` respectively.
- Pinfold's injecting route replaces `x-api-key` with the host value of
  `ANTHROPIC_API_KEY`, with no prefix. The harness supplies the API version
  and its other protocol headers.
- `YARD_ORIGIN_ANTHROPIC` selects a host origin, including the local fixture.
  It does not carry a credential or change the guest base URL.

Upstream reference locations:

- [Claude gateway configuration](https://code.claude.com/docs/en/llm-gateway).
- [Anthropic API overview](https://platform.claude.com/docs/en/api/overview).
- [Anthropic TypeScript SDK client, v0.67.0](https://github.com/anthropics/anthropic-sdk-typescript/blob/v0.67.0/src/client.ts),
  the versioned reference for base URL, API-key and version headers.

These locations could not be retrieved in this worker environment. The
proxy returned HTTP 403 for the documentation, raw GitHub and npm package;
direct access could not resolve DNS. No fetched upstream text or fresh
upstream verification is claimed. The protocol values above are the pinned
implementation and fixture contract. The host gate must establish the
pinned real Claude harness behavior; it has not run here.

## Implementation review

`Config::parse` already calls the adapter's `accepts` and names the agent in
its refusal. Claude now accepts only Anthropic provider mode or its existing
subscription login, and refuses any simultaneous provider/login selection
as `provider_login_conflict`. Pi still accepts only its two existing
connections. No other provider or harness is enabled.

`read_machine` already reads operator variables only on the host.
`ModelRoute::secret` already passes a key in the scrubbed `box up` child
environment. The serialized route contains only its variable name.
`worker_spec` already gives both modes the same MCP grant and containment.
Registration, publication, candidate mounts, identity and outcome handling
are unchanged. No schema, dependency or execution lifecycle was added.

## Validation

`cargo fmt --check` and
`cargo clippy --all-targets --locked -- -D warnings` passed. Clippy compiled
the new e2e scenario. No daemon or development binary was started here.
The real host e2e suite is reserved for the landing gate, as instructed.
No public model endpoint was called. `.yard/` was not edited.
