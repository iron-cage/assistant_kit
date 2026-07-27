# sdk

Contract documentation for the Anthropic Agent SDK (`@anthropic-ai/claude-agent-sdk` /
`claude-agent-sdk`) — the official TypeScript/Python library that wraps the same `claude`
binary `contract/claude_code` documents.

No official Rust bindings exist. This crate documents the SDK's API surface, the underlying
subprocess/stream-json protocol it drives, and observed behaviors relevant to designing a
Rust-native SDK-protocol integration — design-reference material for
`assistant_kit/task/claude_runner/414_implement_sdk_protocol_run_command.md`.

## Structure

| Path | Responsibility |
|------|----------------|
| `docs/` | Agent SDK contract specifications (4 entity types, 29 instances) |
| `src/lib.rs` | Crate documentation |

## Running

Documentation-only crate; no test suite yet — see `docs/behavior/readme.md` Out of Scope.
