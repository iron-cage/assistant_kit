# Doc Entities

## Master Doc Entities Table

| Type | Purpose | Master File | Instances |
|------|---------|-------------|----------:|
| `behavior` | Observed behaviors of the Agent SDK relevant to a Rust integration (S1–S8) | [behavior/readme.md](behavior/readme.md) | 8 |
| `api` | SDK function/type signatures — canonical `api/` doc entity per `doc_des.rulebook.md` | [api/readme.md](api/readme.md) | 6 |
| `param` | Curated `Options`/`ClaudeAgentOptions` field reference (13 of 61 fields) | [param/readme.md](param/readme.md) | 13 |
| `pattern` | Reusable SDK integration patterns, incl. Rust bridge strategy options | [pattern/readme.md](pattern/readme.md) | 2 |

**Total doc instances**: 29 (8 + 6 + 13 + 2)

**Type provenance note**: `behavior`/`param`/`pattern` are local extensions inherited from the `contract/claude_code` precedent this crate mirrors (per user instruction) — not canonical `doc_des.rulebook.md` types; `api` is a canonical type used as-is. See `behavior/readme.md` Scope for the full in/out-of-scope boundary against the sibling `contract/claude_code` crate.

## Master Doc Instances Table

### behavior/ (8 instances)

| ID | Name | File |
|----|------|------|
| S1 | SDK Wraps Same Binary | [behavior/001_s1_sdk_wraps_same_binary.md](behavior/001_s1_sdk_wraps_same_binary.md) |
| S2 | Stream-JSON Control Protocol | [behavior/002_s2_stream_json_control_protocol.md](behavior/002_s2_stream_json_control_protocol.md) |
| S3 | Custom Tools In-Process | [behavior/003_s3_custom_tools_in_process.md](behavior/003_s3_custom_tools_in_process.md) |
| S4 | No Rust Binding | [behavior/004_s4_no_rust_binding.md](behavior/004_s4_no_rust_binding.md) |
| S5 | MCP Tool Naming | [behavior/005_s5_mcp_tool_naming.md](behavior/005_s5_mcp_tool_naming.md) |
| S6 | Permission Modes Richer Than CLI | [behavior/006_s6_permission_modes_richer_than_cli.md](behavior/006_s6_permission_modes_richer_than_cli.md) |
| S7 | Entrypoint Self-Reports SDK | [behavior/007_s7_entrypoint_self_reports_sdk.md](behavior/007_s7_entrypoint_self_reports_sdk.md) |
| S8 | Session Identity: Options vs. Flags | [behavior/008_s8_session_identity_options_vs_flags.md](behavior/008_s8_session_identity_options_vs_flags.md) |

### api/ (6 instances)

| ID | Name | File |
|----|------|------|
| 001 | `query()` | [api/001_query_function.md](api/001_query_function.md) |
| 002 | `Options` / `ClaudeAgentOptions` | [api/002_options_type.md](api/002_options_type.md) |
| 003 | `tool()` + `createSdkMcpServer()` | [api/003_custom_tool_definition.md](api/003_custom_tool_definition.md) |
| 004 | `Query` Control Object | [api/004_query_control_object.md](api/004_query_control_object.md) |
| 005 | `SDKMessage` Stream | [api/005_sdk_message_stream.md](api/005_sdk_message_stream.md) |
| 006 | `CanUseTool` Permission Callback | [api/006_permission_callback.md](api/006_permission_callback.md) |

### param/ (13 instances)

| ID | Name | File |
|----|------|------|
| 001 | cwd | [param/001_cwd.md](param/001_cwd.md) |
| 002 | model | [param/002_model.md](param/002_model.md) |
| 003 | permissionMode | [param/003_permission_mode.md](param/003_permission_mode.md) |
| 004 | allowedTools | [param/004_allowed_tools.md](param/004_allowed_tools.md) |
| 005 | disallowedTools | [param/005_disallowed_tools.md](param/005_disallowed_tools.md) |
| 006 | mcpServers | [param/006_mcp_servers.md](param/006_mcp_servers.md) |
| 007 | resume | [param/007_resume.md](param/007_resume.md) |
| 008 | sessionId | [param/008_session_id.md](param/008_session_id.md) |
| 009 | continue | [param/009_continue.md](param/009_continue.md) |
| 010 | forkSession | [param/010_fork_session.md](param/010_fork_session.md) |
| 011 | systemPrompt | [param/011_system_prompt.md](param/011_system_prompt.md) |
| 012 | pathToClaudeCodeExecutable | [param/012_path_to_claude_code_executable.md](param/012_path_to_claude_code_executable.md) |
| 013 | canUseTool | [param/013_can_use_tool.md](param/013_can_use_tool.md) |

### pattern/ (2 instances)

| ID | Name | File |
|----|------|------|
| 001 | In-Process Custom Tool | [pattern/001_in_process_custom_tool.md](pattern/001_in_process_custom_tool.md) |
| 002 | Rust Bridge Strategies | [pattern/002_rust_bridge_strategies.md](pattern/002_rust_bridge_strategies.md) |
