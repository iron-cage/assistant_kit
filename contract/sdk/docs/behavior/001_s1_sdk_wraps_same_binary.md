# Behavior S1: SDK Wraps the Same `claude` Binary

### Scope

- **Purpose**: Document that the Agent SDK is a client library over a locally-spawned `claude` subprocess, not a hosted API service with its own wire protocol.
- **Responsibility**: Authoritative instance for behavior S1 — defines the behavior statement, certainty level, and supporting evidence.
- **In Scope**: `query()`'s default subprocess-spawn behavior; the `pathToClaudeCodeExecutable`/`spawnClaudeCodeProcess`/`executable`/`executableArgs` escape hatches that only make sense if a local process is being spawned.
- **Out of Scope**: The exact wire protocol spoken over that subprocess's stdio (→ [S2](002_s2_stream_json_control_protocol.md)); Rust-side implications of this fact (→ [`../pattern/002_rust_bridge_strategies.md`](../pattern/002_rust_bridge_strategies.md)).

### Behavior

**Status**: ✅ Confirmed | **Certainty**: 95% | **Since**: SDK GA (renamed from Claude Code SDK, Sept 2025) | **Evidence**: E1, E2

The TypeScript `Options` interface documents `pathToClaudeCodeExecutable?: string` ("default: Auto-resolved") and `spawnClaudeCodeProcess?: (options: SpawnOptions) => SpawnedProcess`, plus `executable?: 'bun' | 'deno' | 'node'` and `executableArgs?: string[]`. None of these fields are meaningful unless `query()` locates and spawns a `claude` binary as a child process by default — `pathToClaudeCodeExecutable` exists specifically to override *which* binary gets spawned, and `spawnClaudeCodeProcess` exists to override *how* the spawn itself happens. The TypeScript package additionally "bundles a native Claude Code binary for your platform as an optional dependency," so a fresh `npm install @anthropic-ai/claude-agent-sdk` is self-sufficient without a separate `claude` install — but the binary being bundled *is* the same `claude` CLI binary `contract/claude_code` documents, not a distinct SDK-only runtime.

This reframes the relationship between `contract/claude_code` and this crate: `contract/claude_code` documents the binary's own CLI-level contract (flags, storage, JSONL); this crate documents a *second, higher-level client* of that same binary — the SDK adds its own request/response typing, control-message protocol, and language ergonomics on top, but ultimately still execs `claude` and talks to it over stdio, the same way `clr` (this workspace's own wrapper, confirmed via `ps` ancestry) already does with a simpler flag set.

```typescript
// Default: SDK resolves and spawns its own bundled/PATH-found `claude` binary
for await (const message of query({ prompt: "..." })) { /* ... */ }

// Override: point at a specific already-installed binary instead
for await (const message of query({
  prompt: "...",
  options: { pathToClaudeCodeExecutable: "/usr/local/bin/claude" },
})) { /* ... */ }
```

### Evidence

| ID | Supports | Type | Source | Location | Content |
|----|----------|------|--------|----------|---------|
| E1 | S1 | Doc | `https://code.claude.com/docs/en/agent-sdk/overview` | "Get started" step 1 | "The TypeScript SDK bundles a native Claude Code binary for your platform as an optional dependency, so you don't need to install Claude Code separately." |
| E2 | S1 | Doc | `https://code.claude.com/docs/en/agent-sdk/typescript` | `Options` interface | `pathToClaudeCodeExecutable`, `spawnClaudeCodeProcess`, `executable`, `executableArgs` fields |

### Cross-References

| Type | File | Responsibility |
|------|------|-----------------|
| entity | [readme.md](readme.md) | Master index: evidence table, overview table |
| behavior | [002_s2_stream_json_control_protocol.md](002_s2_stream_json_control_protocol.md) | The protocol spoken over the spawned subprocess |
| api | [../api/001_query_function.md](../api/001_query_function.md) | `query()` signature |
| param | [../param/012_path_to_claude_code_executable.md](../param/012_path_to_claude_code_executable.md) | `pathToClaudeCodeExecutable` field detail |
| pattern | [../pattern/002_rust_bridge_strategies.md](../pattern/002_rust_bridge_strategies.md) | Rust integration options this fact enables |
