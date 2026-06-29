# Types

### Scope

- **Purpose**: Define semantic types that constrain parameter values and classify output results.
- **Responsibility**: Specify fundamental type, constants, constraints, parsing rules, and cross-references to parameters and commands for each type.
- **In Scope**: All 13 active types — 11 parameter value types and 2 output/result types (`ErrorKind`, `ErrorClass`). `VerbosityLevel` deprecated and removed.
- **Out of Scope**: Parameter behavior (-> `../param/`), group membership (-> `../param_group/`).

### Responsibility Table

| File | Responsibility |
|------|----------------|
| 01_message_text.md | Type spec: free-form prompt text for Claude |
| 02_directory_path.md | Type spec: filesystem path to a directory |
| 03_token_limit.md | Type spec: maximum output token count |
| 04_model_name.md | Type spec: Claude model identifier string |
| 05_verbosity_level.md | Type spec: runner diagnostic output gate (DEPRECATED — removed) |
| 06_system_prompt_text.md | Type spec: free-form system prompt text |
| 07_effort_level.md | Type spec: reasoning effort level enumeration |
| 08_credentials_file_path.md | Type spec: path to credentials JSON file |
| 09_timeout_secs.md | Type spec: subprocess wait limit in seconds |
| 10_json_schema_text.md | Type spec: JSON Schema object string |
| 11_mcp_config_path.md | Type spec: MCP configuration JSON file path |
| 12_file_path.md | Type spec: readable file path for subprocess stdin |
| 13_error_kind.md | Type spec: subprocess failure classification enum |
| 14_error_class.md | Type spec: caller-facing error class taxonomy |

### All Types (13 active total; 1 deprecated)

| # | Type | Fundamental Type | Parameters / Source | Purpose |
|---|------|-----------------|---------------------|---------|
| 1 | `MessageText` | String | [`[MESSAGE]`](../param/001_message.md) | Free-form prompt text sent to the `claude` subprocess |
| 2 | `DirectoryPath` | String | [`--dir`](../param/008_dir.md), [`--session-dir`](../param/010_session_dir.md) | Filesystem path to a directory |
| 3 | `TokenLimit` | unsigned 32-bit integer | [`--max-tokens`](../param/009_max_tokens.md) | Maximum output token count |
| 4 | `ModelName` | String | [`--model`](../param/003_model.md) | Claude model identifier string |
| 5 | ~~`VerbosityLevel`~~ | ~~unsigned 8-bit integer~~ | ~~[`--verbosity`](../param/012_verbosity.md)~~ | **DEPRECATED** — removed; replaced by `--quiet` bool |
| 6 | `SystemPromptText` | String | [`--system-prompt`](../param/015_system_prompt.md), [`--append-system-prompt`](../param/016_append_system_prompt.md) | Free-form system prompt text (system turn, not user turn) |
| 7 | `EffortLevel` | enumeration | [`--effort`](../param/017_effort.md) | Reasoning effort level forwarded to the `claude` subprocess |
| 8 | `CredentialsFilePath` | String | [`--creds`](../param/019_creds.md) | Path to an existing credentials JSON file |
| 9 | `TimeoutSecs` | unsigned 64-bit integer | [`--timeout`](../param/020_timeout.md) | Subprocess wait limit in seconds |
| 10 | `JsonSchemaText` | String | [`--json-schema`](../param/023_json_schema.md) | JSON Schema object string for structured output |
| 11 | `McpConfigPath` | String | [`--mcp-config`](../param/024_mcp_config.md) | Filesystem path to an MCP configuration JSON file |
| 12 | `FilePath` | String | [`--file`](../param/025_file.md) | Filesystem path to a readable file piped as subprocess stdin |
| 13 | `ErrorKind` | enumeration (6 variants) | `classify_error()` return type | Subprocess failure classification: `RateLimit`, `QuotaExhausted`, `AuthError`, `ApiError`, `Signal`, `Unknown` |
| 14 | `ErrorClass` | taxonomy (8 classes) | documentation type | Caller-facing grouping of all CLI error conditions into semantic response classes |

**Total:** 13 active types (11 parameter types + 2 output/result types); `VerbosityLevel` (5) deprecated
