# Behavior Doc Entity

### Scope

- **Purpose**: Catalog observed and confirmed behaviors of the Anthropic Agent SDK (`@anthropic-ai/claude-agent-sdk` / `claude-agent-sdk`) relevant to designing a Rust-native integration.
- **Responsibility**: Master file for the `behavior` collection — lists all 8 behavior instances (S1–S8), provides the shared evidence table (E1–E8).
- **In Scope**: SDK-to-binary relationship (does the SDK spawn `claude` itself, or talk to a service); wire protocol used between SDK and binary; custom-tool execution model; language-binding availability; MCP tool naming; permission-mode surface; entrypoint self-identification; session-identity field surface.
- **Out of Scope**: Full `Options`/`ClaudeAgentOptions` field reference (→ [`../param/`](../param/readme.md)); function/type signatures (→ [`../api/`](../api/readme.md)); reusable integration patterns (→ [`../pattern/`](../pattern/readme.md)); the `claude` binary's own CLI-level behavioral contract, already documented in the sibling crate (→ [`../../../claude_code/docs/behavior/readme.md`](../../../claude_code/docs/behavior/readme.md)).

**Note on test coverage:** unlike `contract/claude_code`, this collection has no invalidation test suite yet (no `tests/` directory in this crate). Confirming these behaviors against a live `claude-agent-sdk` package requires Node.js or Python installed in the container plus a real Anthropic API credential exercising the SDK — out of scope until `assistant_kit/task/claude_runner/414_implement_sdk_protocol_run_command.md` lands an actual Rust integration to test against. Every behavior below is therefore evidenced by official documentation and this workspace's own decompiled-binary findings (already `✅ Confirmed`/`🎯 Observed` tier in `contract/claude_code`), not by a locally-run assertion — see each instance's Evidence table for the exact source.

### Overview Table

Adapted from the `contract/claude_code` hypothesis table format. Status reflects certainty of the observation, not investigation state.

**Status legend:**
- ✅ Confirmed — official documentation states this directly
- 🎯 Observed — inferable from official documentation plus this workspace's own confirmed findings
- ❓ Uncertain — reasonable inference, unconfirmed

| ID | Behavior | Category | Status | Certainty | Since | Evidence |
|----|----------|----------|--------|-----------|-------|----------|
| [S1](001_s1_sdk_wraps_same_binary.md) | The SDK's `query()` spawns the same `claude` CLI binary as a subprocess by default — it is a client library over a local process, not a hosted service | Architecture | ✅ | 95% | SDK GA (renamed from Claude Code SDK, Sept 2025) | E1, E2 |
| [S2](002_s2_stream_json_control_protocol.md) | The SDK drives `claude` via `--input-format stream-json --output-format stream-json`, a bidirectional NDJSON control protocol — not the single-shot `--print --output-format json` mode `clr` currently uses | Protocol | 🎯 | 85% | SDK GA | E2, E4 |
| [S3](003_s3_custom_tools_in_process.md) | Custom tools registered via `tool()` + `createSdkMcpServer()` execute in-process (same Node.js/Python process as the SDK caller) — not through the built-in Bash tool's OS-subprocess harness | Tools | ✅ | 95% | SDK GA | E2, E6, E8 |
| [S4](004_s4_no_rust_binding.md) | No official Rust package exists for the Agent SDK — only `@anthropic-ai/claude-agent-sdk` (npm) and `claude-agent-sdk` (PyPI, Python ≥3.10) are published | Language Support | ✅ | 99% | SDK GA | E1 |
| [S5](005_s5_mcp_tool_naming.md) | SDK-registered custom tools are addressed by Claude using the `mcp__{server_name}__{tool_name}` naming convention, identical to external MCP servers | Tools | ✅ | 95% | SDK GA | E2 |
| [S6](006_s6_permission_modes_richer_than_cli.md) | The SDK's `permissionMode` option accepts 6 values (`default`, `acceptEdits`, `bypassPermissions`, `plan`, `dontAsk`, `auto`) — a richer enum than the `claude` binary's own `--permission-mode` CLI surface | Permissions | 🎯 | 80% | SDK GA | E2, E7 |
| [S7](007_s7_entrypoint_self_reports_sdk.md) | A `claude` process launched by the real SDK sets `CLAUDE_CODE_ENTRYPOINT` to `"sdk-ts"` or `"sdk-cli"` — but a hand-rolled subprocess wrapper shaped like `--print --output-format json -c` self-reports the identical `"sdk-cli"` value without using the SDK library at all | Process Identity | ✅ | 90% | ≤v2.1.197 | E3, E5 |
| [S8](008_s8_session_identity_options_vs_flags.md) | Session identity/continuation is controlled via `Options` fields (`resume`, `sessionId`, `continue`, `forkSession`) that map 1:1 onto `claude_code` CLI flags already documented (B4/B19/B20/B21), but are typed struct fields instead of argv strings | Session | ✅ | 90% | SDK GA | E2 |

---

### Evidence Table

Evidence items are shared across behaviors (M:N relationship).

| ID | Supports | Type | Source | Location | Content |
|----|----------|------|--------|----------|---------|
| E1 | S1, S4 | Doc | `https://code.claude.com/docs/en/agent-sdk/overview` | "Get started" / "Agent SDK vs Claude Code CLI" | Install instructions (`npm install @anthropic-ai/claude-agent-sdk`, `pip install claude-agent-sdk`, Python ≥3.10); "Same capabilities, different interface" comparison table; no third language SDK listed |
| E2 | S1, S2, S3, S5, S6, S8 | Doc | `https://code.claude.com/docs/en/agent-sdk/typescript` | Full `Options` interface, `tool()`/`createSdkMcpServer()` signatures | `pathToClaudeCodeExecutable`/`spawnClaudeCodeProcess`/`executable`/`executableArgs` fields confirm a local subprocess is spawned; MCP naming section states `mcp__{server}__{tool}` explicitly; `PermissionMode` union has 6 members |
| E3 | S7 | Doc | [`../../../claude_code/docs/param/134_entrypoint.md`](../../../claude_code/docs/param/134_entrypoint.md) | Description | `CLAUDE_CODE_ENTRYPOINT` values include `"sdk-cli"` and `"sdk-ts"`, confirmed via binary string/reference inspection |
| E4 | S2 | Doc | [`../../../claude_code/docs/param/034_input_format.md`](../../../claude_code/docs/param/034_input_format.md), [`044_output_format.md`](../../../claude_code/docs/param/044_output_format.md) | Description | `--input-format stream-json` / `--output-format stream-json` already documented as the CLI-level bidirectional NDJSON mode; the SDK's control protocol (`SDKControlRequestMessage`/`SDKControlResponseMessage`) is layered on top of this same CLI surface |
| E5 | S7 | Observation | This session, `ps` ancestry walk | `clr` → `claude --print --output-format json -c "<msg>"` | `clr` (this workspace's own wrapper) is confirmed to manually spawn-and-parse-JSON per turn rather than link the SDK library, yet the resulting process shape is indistinguishable from a real `sdk-cli` launch at the `CLAUDE_CODE_ENTRYPOINT` level — see workspace memory `reference_clr_default_invocation_pattern` |
| E6 | S3 | Doc | `https://code.claude.com/docs/en/agent-sdk/typescript` | `tool()` / `createSdkMcpServer()` code block | Handler signature is `(args, extra) => Promise<CallToolResult>` — a plain in-process async function, not a subprocess dispatch |
| E7 | S6 | Doc | `https://code.claude.com/docs/en/agent-sdk/typescript` | `PermissionMode` type definition | 6-member union documented verbatim: `default`, `acceptEdits`, `bypassPermissions`, `plan`, `dontAsk`, `auto` |
| E8 | S3 | Doc | `https://code.claude.com/docs/en/agent-sdk/overview` | "Agent SDK vs Managed Agents" comparison table | Row "Custom tools": "In-process Python or TypeScript functions" (Agent SDK) vs. "Claude triggers the tool; you execute and return results" (Managed Agents) — Anthropic's own explicit in-process framing |

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| entity | [../001_entity.md](../001_entity.md) | Cross-entity index for this crate |
| api | [../api/readme.md](../api/readme.md) | Function/type signatures referenced by these behaviors |
| param | [../param/readme.md](../param/readme.md) | `Options` field-level reference |
| pattern | [../pattern/readme.md](../pattern/readme.md) | Reusable integration patterns built on these behaviors |
