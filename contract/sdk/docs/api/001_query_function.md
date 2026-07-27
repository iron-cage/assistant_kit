# API: `query()`

### Scope

- **Purpose**: Document the SDK's primary entry point — the function that starts the agent loop and returns a streaming result.
- **Responsibility**: Authoritative signature reference for `query()` in both TypeScript and Python.
- **In Scope**: Parameter shape, return type, minimal usage.
- **Out of Scope**: Individual `Options` fields (→ `param/`); the `Query` object's own methods (→ `004_query_control_object.md`).

TypeScript:

```typescript
function query({
  prompt,
  options
}: {
  prompt: string | AsyncIterable<SDKUserMessage>;
  options?: Options;
}): Query;
```

`Query extends AsyncGenerator<SDKMessage, void>` — the return value is both an async-iterable message stream and a live control object (see `004_query_control_object.md`). `prompt` accepts either a plain string (single-shot) or an `AsyncIterable<SDKUserMessage>` (multi-turn streaming input, requiring the underlying `stream-json` input mode — see [`../behavior/002_s2_stream_json_control_protocol.md`](../behavior/002_s2_stream_json_control_protocol.md)).

Python (from official overview page example):

```python
from claude_agent_sdk import query, ClaudeAgentOptions

async for message in query(
    prompt="Find and fix the bug in auth.py",
    options=ClaudeAgentOptions(allowed_tools=["Read", "Edit", "Bash"]),
):
    print(message)
```

Python's `query()` is an async generator consumed via `async for`, rather than TypeScript's `Query`-object-with-methods; the official docs did not surface Python equivalents of the TypeScript `Query` object's mid-session control methods (`interrupt()`, `setPermissionMode()`, etc.) in the pages fetched for this crate — treat that gap as unconfirmed, not as evidence those methods don't exist in Python.

There is also a `startup()` function (`function startup(params?: { options?: Options; initializeTimeoutMs?: number }): Promise<WarmQuery>`) that pre-warms the CLI subprocess before a prompt is available, returning a `WarmQuery` with its own `.query(prompt)` and `.close()` — relevant for a Rust integration that wants to hide subprocess-startup latency behind an explicit warm-up phase (→ [`../pattern/002_rust_bridge_strategies.md`](../pattern/002_rust_bridge_strategies.md)).

### Behaviors

| File | Relationship |
|------|--------------|
| [../behavior/001_s1_sdk_wraps_same_binary.md](../behavior/001_s1_sdk_wraps_same_binary.md) | `query()` is what triggers the subprocess spawn documented there |
| [../behavior/002_s2_stream_json_control_protocol.md](../behavior/002_s2_stream_json_control_protocol.md) | The protocol `query()` drives once the subprocess is running |

### Params

| File | Relationship |
|------|--------------|
| [../param/readme.md](../param/readme.md) | `options` argument's field reference |

### Patterns

| File | Relationship |
|------|--------------|
| [../pattern/002_rust_bridge_strategies.md](../pattern/002_rust_bridge_strategies.md) | `startup()`/`WarmQuery` relevance to a Rust bridge's startup-latency handling |
