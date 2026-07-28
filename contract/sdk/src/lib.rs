//! Behavioral contract documentation for the Anthropic Agent SDK (`@anthropic-ai/claude-agent-sdk`
//! / `claude-agent-sdk`) — the official TypeScript/Python library that wraps the same `claude`
//! binary `contract/claude_code` documents.
//!
//! No official Rust bindings exist. This crate documents the SDK's API surface (`docs/api/`),
//! observed behaviors relevant to a Rust integration (`docs/behavior/`), the `Options`/
//! `ClaudeAgentOptions` fields most relevant to porting `claude_runner`'s invocation model
//! (`docs/param/`), and reusable integration patterns (`docs/pattern/`) — as design-reference
//! material for `assistant_kit/task/claude_runner/414_implement_sdk_protocol_run_command.md`.
//!
//! Documentation-only crate. No invalidation test suite yet — see `docs/behavior/readme.md`
//! Out of Scope for why, and the task file above for the tracked follow-up.
