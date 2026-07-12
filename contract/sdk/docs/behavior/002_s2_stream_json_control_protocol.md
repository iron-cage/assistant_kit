# Behavior S2: SDK Drives `claude` via a Bidirectional stream-json Control Protocol

### Scope

- **Purpose**: Document that the SDK's `query()` uses a fundamentally different, bidirectional wire mode than `claude_runner`'s current single-shot invocation.
- **Responsibility**: Authoritative instance for behavior S2.
- **In Scope**: The relationship between the SDK's `Query` control methods (`interrupt()`, `setPermissionMode()`, `setModel()`, `streamInput()`, etc.) and the underlying `--input-format stream-json --output-format stream-json` CLI mode already documented in `contract/claude_code`.
- **Out of Scope**: The CLI flags themselves, already fully documented (→ [`../../../claude_code/docs/param/034_input_format.md`](../../../claude_code/docs/param/034_input_format.md), [`044_output_format.md`](../../../claude_code/docs/param/044_output_format.md)); `clr`'s current simpler mode (→ [S1](001_s1_sdk_wraps_same_binary.md) Description).

### Behavior

**Status**: 🎯 Observed | **Certainty**: 85% | **Since**: SDK GA | **Evidence**: E2, E4

`clr`'s current invocation (confirmed this session via `ps` ancestry: `claude --print --output-format json -c "<msg>"`) is single-shot: one process, one prompt in, one JSON blob out, process exits. The SDK's `Query` object exposes an entirely different shape — `interrupt()`, `setPermissionMode(mode)`, `setModel(model)`, `streamInput(stream)`, `stopTask(taskId)`, `close()` — none of which are expressible against a process that exits after one response. These only make sense against a **long-lived subprocess** that the SDK keeps open and exchanges a continuous stream of control/response messages with over stdio, matching the `SDKControlRequestMessage` / `SDKControlResponseMessage` variants listed in the `SDKMessage` union.

`contract/claude_code` already documents the CLI-level primitives this almost certainly rides on: `--input-format stream-json` ("accepts a stream of newline-delimited JSON message objects, enabling multi-turn structured input... full bidirectional JSON streaming") and `--output-format stream-json` ("emits a stream of newline-delimited JSON objects as chunks arrive"). The SDK's `query()` most plausibly spawns `claude` once with both flags set to `stream-json` and keeps the child process's stdin open for the lifetime of the `Query` object, rather than the text-mode single-shot flags `clr` uses today (`--print --output-format json` with no `--input-format`, implying default `text` input).

**Not directly confirmed**: no test in this crate has captured the literal argv `claude` is spawned with by the SDK (would require intercepting the subprocess spawn, e.g. via `pathToClaudeCodeExecutable` pointed at a logging shim — see [`../pattern/002_rust_bridge_strategies.md`](../pattern/002_rust_bridge_strategies.md) for that exact technique as a verification strategy). The inference is from control-surface shape, not from an observed argv.

### Evidence

| ID | Supports | Type | Source | Location | Content |
|----|----------|------|--------|----------|---------|
| E2 | S2 | Doc | `https://code.claude.com/docs/en/agent-sdk/typescript` | `Query` interface | `interrupt()`, `setPermissionMode()`, `setModel()`, `streamInput()`, `stopTask()`, `close()` — a live, mid-session control surface |
| E4 | S2 | Doc | [`../../../claude_code/docs/param/034_input_format.md`](../../../claude_code/docs/param/034_input_format.md), [`044_output_format.md`](../../../claude_code/docs/param/044_output_format.md) | Description | `stream-json` is described as enabling "full bidirectional JSON streaming" at the CLI level, independent of the SDK |

### Cross-References

| Type | File | Responsibility |
|------|------|-----------------|
| entity | [readme.md](readme.md) | Master index |
| behavior | [001_s1_sdk_wraps_same_binary.md](001_s1_sdk_wraps_same_binary.md) | Establishes a subprocess is spawned at all |
| api | [../api/004_query_control_object.md](../api/004_query_control_object.md) | `Query` object control methods |
| api | [../api/005_sdk_message_stream.md](../api/005_sdk_message_stream.md) | `SDKControlRequestMessage`/`SDKControlResponseMessage` variants |
| doc | [`../../../claude_code/docs/param/034_input_format.md`](../../../claude_code/docs/param/034_input_format.md) | `--input-format stream-json` CLI-level primitive |
| doc | [`../../../claude_code/docs/param/044_output_format.md`](../../../claude_code/docs/param/044_output_format.md) | `--output-format stream-json` CLI-level primitive |
| pattern | [../pattern/002_rust_bridge_strategies.md](../pattern/002_rust_bridge_strategies.md) | How a Rust caller could speak this same protocol directly |
