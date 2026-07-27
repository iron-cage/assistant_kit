# Behavior S4: No Official Rust Binding Exists

### Scope

- **Purpose**: Document the exact set of officially-published Agent SDK language bindings, and confirm Rust is not among them.
- **Responsibility**: Authoritative instance for behavior S4 — the foundational constraint motivating this entire crate and the linked task file.
- **In Scope**: Officially published packages (`@anthropic-ai/claude-agent-sdk` on npm, `claude-agent-sdk` on PyPI) and their minimum runtime versions.
- **Out of Scope**: The architectural options this constraint leaves open for a Rust workspace (→ [`../pattern/002_rust_bridge_strategies.md`](../pattern/002_rust_bridge_strategies.md)).

### Behavior

**Status**: ✅ Confirmed | **Certainty**: 99% | **Since**: SDK GA | **Evidence**: E1

The official Agent SDK overview page documents exactly two install paths — `npm install @anthropic-ai/claude-agent-sdk` (TypeScript) and `pip install claude-agent-sdk` (Python, requiring "Python 3.10 or later") — and the "Agent SDK vs Claude Code CLI" / "Agent SDK vs Client SDK" / "Agent SDK vs Managed Agents" comparison sections consistently refer only to "Python or TypeScript" as the library's target languages. No Rust package, no `cargo` install path, and no Rust code samples appear anywhere in the fetched documentation.

This is the reason `claude_runner`/`claude_runner_core` — both pure Rust crates — cannot simply add a dependency and call the SDK the way they might add any other Rust crate. Any SDK-mode integration must cross a language boundary one way or another; see [`../pattern/002_rust_bridge_strategies.md`](../pattern/002_rust_bridge_strategies.md) for the two concrete strategies this leaves open, and `assistant_kit/task/claude_runner/414_implement_sdk_protocol_run_command.md` for the task tracking which strategy to actually build.

### Evidence

| ID | Supports | Type | Source | Location | Content |
|----|----------|------|--------|----------|---------|
| E1 | S4 | Doc | `https://code.claude.com/docs/en/agent-sdk/overview` | "Get started" step 1; changelog/reporting-bugs sections | Only `@anthropic-ai/claude-agent-sdk` (npm) and `claude-agent-sdk` (PyPI) are listed; changelog links point to `claude-agent-sdk-typescript` and `claude-agent-sdk-python` GitHub repos only |

### Cross-References

| Type | File | Responsibility |
|------|------|-----------------|
| entity | [readme.md](readme.md) | Master index |
| pattern | [../pattern/002_rust_bridge_strategies.md](../pattern/002_rust_bridge_strategies.md) | Architectural options given this constraint |
| doc | `assistant_kit/task/claude_runner/414_implement_sdk_protocol_run_command.md` | Task tracking the actual Rust integration decision |
