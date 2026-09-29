// Yard's MCP client for Pi, embedded in the yard binary and staged read-only
// into every worker box. The file is the same bytes for every execution: the
// endpoint and bearer come from the process environment, never from a file.
//
// It speaks one subset of MCP over HTTP: one POST per JSON-RPC message, a JSON
// response body, `initialize`, `notifications/initialized`, one page of
// `tools/list`, and `tools/call`. Anything else the server answers is a
// failure, not something to adapt to. No retries: a mutating call repeated on
// its own would be a second call nobody made.
//
// It names no tool. The worker gets whatever `tools/list` answered for its
// bearer, so the grant lives only in the daemon.
//
// It prints one line on stderr before the first turn, `yard-mcp {json}`, and
// throws on failure, which makes Pi exit before any model request.

import type { ExtensionAPI } from "@earendil-works/pi-coding-agent";

const ENDPOINT_ENV = "YARD_MCP_ENDPOINT";
const BEARER_ENV = "YARD_MCP_BEARER";
const PROTOCOL_VERSION = "2025-06-18";
const CLIENT_NAME = "yard-pi-mcp";
const SENTINEL = "yard-mcp";
const REQUEST_TIMEOUT_MS = 60_000;
const RESPONSE_MAX_BYTES = 4 * 1024 * 1024;

interface McpTool {
  name: string;
  description?: string;
  inputSchema?: unknown;
}

interface McpToolResult {
  content?: { text?: unknown }[];
  isError?: boolean;
}

class McpClient {
  private nextId = 1;
  // The server binds the bearer to the session `initialize` opens; the id
  // lives only in this process.
  private session: string | undefined;

  constructor(
    private readonly endpoint: string,
    private readonly bearer: string,
  ) {}

  async request(method: string, params: unknown, signal?: AbortSignal): Promise<any> {
    const id = this.nextId++;
    const body = await this.post({ jsonrpc: "2.0", id, method, params }, signal);
    let envelope: any;
    try {
      envelope = JSON.parse(body);
    } catch {
      throw new Error(`${method} answered a body that is not JSON`);
    }
    if (envelope === null || typeof envelope !== "object" || envelope.jsonrpc !== "2.0") {
      throw new Error(`${method} answered something other than a JSON-RPC 2.0 message`);
    }
    if (envelope.error !== undefined) {
      const code = String(envelope.error?.code ?? "unknown");
      const message = String(envelope.error?.message ?? "no message");
      throw new Error(`${method} was refused: ${code} ${message}`);
    }
    if (envelope.id !== id) {
      throw new Error(`${method} answered id ${JSON.stringify(envelope.id)}, expected ${id}`);
    }
    if (envelope.result === undefined) throw new Error(`${method} answered no result`);
    return envelope.result;
  }

  async notify(method: string, params: unknown): Promise<void> {
    await this.post({ jsonrpc: "2.0", method, params });
  }

  // One message, one POST, bounded in time and bytes, cancellable by the turn.
  private async post(message: unknown, signal?: AbortSignal): Promise<string> {
    const timeout = AbortSignal.timeout(REQUEST_TIMEOUT_MS);
    const bound = signal === undefined ? timeout : AbortSignal.any([signal, timeout]);
    let response: Response;
    try {
      response = await fetch(this.endpoint, {
        method: "POST",
        headers: {
          "content-type": "application/json",
          accept: "application/json",
          authorization: `Bearer ${this.bearer}`,
          ...(this.session === undefined ? {} : { "mcp-session-id": this.session }),
        },
        body: JSON.stringify(message),
        signal: bound,
      });
    } catch (cause) {
      if (signal?.aborted === true) throw new Error("the call was cancelled");
      if (timeout.aborted) {
        throw new Error(`the server did not answer within ${REQUEST_TIMEOUT_MS} ms`);
      }
      throw new Error(`could not reach the MCP endpoint: ${errorText(cause)}`);
    }
    if ((message as { method?: unknown }).method === "initialize" && response.ok) {
      this.session = response.headers.get("mcp-session-id") ?? undefined;
      if (this.session === undefined) throw new Error("initialize opened no session");
    }
    if (response.status === 202) {
      await discardBody(response);
      return "";
    }
    if (!response.ok) {
      await discardBody(response);
      throw new Error(`the MCP endpoint answered HTTP ${response.status}`);
    }
    return await readBounded(response, bound);
  }
}

export default async function (pi: ExtensionAPI) {
  try {
    const endpoint = process.env[ENDPOINT_ENV];
    const bearer = process.env[BEARER_ENV];
    if (endpoint === undefined || endpoint === "") throw new Error(`${ENDPOINT_ENV} is unset`);
    if (bearer === undefined || bearer === "") throw new Error(`${BEARER_ENV} is unset`);
    const tools = await register(pi, new McpClient(endpoint, bearer));
    report({ registered: true, tools });
  } catch (cause) {
    report({ registered: false, reason: errorText(cause) });
    throw new Error(`MCP tools were not registered: ${errorText(cause)}`);
  }
}

// Runs in the extension factory, which Pi awaits before the session starts,
// so every tool is registered before the model is asked anything.
async function register(pi: ExtensionAPI, client: McpClient): Promise<string[]> {
  await client.request("initialize", {
    protocolVersion: PROTOCOL_VERSION,
    capabilities: {},
    clientInfo: { name: CLIENT_NAME, version: "1" },
  });
  await client.notify("notifications/initialized", {});
  const listed = await client.request("tools/list", {});
  if (listed === null || typeof listed !== "object" || !Array.isArray(listed.tools)) {
    throw new Error("tools/list answered no tool array");
  }
  // A second page would be granted tools this worker never saw.
  if (listed.nextCursor !== undefined) {
    throw new Error("tools/list answered a paginated list");
  }
  const names: string[] = [];
  for (const tool of listed.tools as McpTool[]) {
    if (tool === null || typeof tool !== "object" || typeof tool.name !== "string") {
      throw new Error("tools/list answered a tool with no name");
    }
    if (tool.inputSchema === null || typeof tool.inputSchema !== "object") {
      throw new Error(`tools/list answered no input schema for ${tool.name}`);
    }
    registerOne(pi, client, tool);
    names.push(tool.name);
  }
  return names;
}

function registerOne(pi: ExtensionAPI, client: McpClient, tool: McpTool): void {
  pi.registerTool({
    name: tool.name,
    label: tool.name,
    description: tool.description ?? "",
    // Passed through unchanged: Pi validates arguments against it and sends it
    // to the provider, so a rewritten schema would accept something else.
    parameters: tool.inputSchema as never,
    async execute(_toolCallId: string, params: unknown, signal?: AbortSignal) {
      const result = (await client.request(
        "tools/call",
        { name: tool.name, arguments: params ?? {} },
        signal,
      )) as McpToolResult;
      // Yard answers one text block.
      const text = result.content?.[0]?.text;
      if (typeof text !== "string") throw new Error("tools/call answered no text");
      // Pi ignores a returned isError; only a throw reaches the model as an error.
      if (result.isError === true) throw new Error(text);
      return { content: [{ type: "text", text }], details: {} };
    },
  } as never);
}

function report(payload: Record<string, unknown>): void {
  process.stderr.write(`${SENTINEL} ${JSON.stringify(payload)}\n`);
}

function errorText(cause: unknown): string {
  return cause instanceof Error ? cause.message : String(cause);
}

async function readBounded(response: Response, signal: AbortSignal): Promise<string> {
  const body = response.body;
  if (body === null) return "";
  const reader = body.getReader();
  const chunks: Uint8Array[] = [];
  let size = 0;
  try {
    for (;;) {
      if (signal.aborted) throw new Error("the call was cancelled");
      const next = await reader.read();
      if (next.done) break;
      size += next.value.byteLength;
      if (size > RESPONSE_MAX_BYTES) {
        throw new Error(`the server answered more than ${RESPONSE_MAX_BYTES} bytes`);
      }
      chunks.push(next.value);
    }
  } finally {
    await reader.cancel().catch(() => {});
  }
  return Buffer.concat(chunks).toString("utf8");
}

async function discardBody(response: Response): Promise<void> {
  try {
    await response.body?.cancel();
  } catch {
    // Nobody reads this body.
  }
}
