# Param Doc Entity

A curated subset of the `Options`/`ClaudeAgentOptions` fields — the ones most relevant to designing
a Rust-native SDK-protocol integration for `claude_runner`. For the full, exhaustive 60-field listing,
see [`../api/002_options_type.md`](../api/002_options_type.md).

### Scope

- **Purpose**: Detailed per-field reference for the `Options` fields that most directly parallel or extend `claude_runner`'s existing CLI-flag-based invocation model.
- **Responsibility**: Master table and per-field detail files for the 13 curated fields.
- **In Scope**: `cwd`, `model`, `permissionMode`, `allowedTools`, `disallowedTools`, `mcpServers`, `resume`, `sessionId`, `continue`, `forkSession`, `systemPrompt`, `pathToClaudeCodeExecutable`, `canUseTool`.
- **Out of Scope**: The remaining ~47 `Options` fields (hooks, sandboxing, budget/thinking controls, plugin/skill loading, session-store persistence, etc.) — listed verbatim but not individually detailed in [`../api/002_options_type.md`](../api/002_options_type.md); `claude_code`'s own CLI-flag parameter reference (→ [`../../../claude_code/docs/param/readme.md`](../../../claude_code/docs/param/readme.md)).

### Responsibility Table

| File | Responsibility |
|------|-----------------|
| readme.md | Master curated parameter table (this file) |
| 001_cwd.md | `cwd` — working directory the spawned process operates in |
| 002_model.md | `model` — model alias/ID override |
| 003_permission_mode.md | `permissionMode` — session-wide permission posture |
| 004_allowed_tools.md | `allowedTools` — tool allowlist |
| 005_disallowed_tools.md | `disallowedTools` — tool denylist, incl. MCP wildcard forms |
| 006_mcp_servers.md | `mcpServers` — external + in-process SDK server registration |
| 007_resume.md | `resume` — resume a specific prior session by ID |
| 008_session_id.md | `sessionId` — assign a deterministic session ID |
| 009_continue.md | `continue` — resume the most recently modified session |
| 010_fork_session.md | `forkSession` — branch a new session ID on resume |
| 011_system_prompt.md | `systemPrompt` — replace/append to the default system prompt |
| 012_path_to_claude_code_executable.md | `pathToClaudeCodeExecutable` — override which `claude` binary is spawned |
| 013_can_use_tool.md | `canUseTool` — per-call permission override callback |
