# Testing

### Scope

- **Purpose**: Document integration and edge case test plans for all clr commands, parameters, and types.
- **Responsibility**: Index of per-command, per-parameter, per-type, per-group, and per-env-param test case planning files.
- **In Scope**: All 4 clr commands, all 27 parameters, all 12 types, all 4 parameter groups, 1 env parameter, and test surface for feature/invariant/api doc instances.
- **Out of Scope**: Automated test implementations (→ `tests/` in crate), spec documentation (→ `docs/feature/`).

Test case planning for `clr` CLI. Each file contains a Test Case Index with coverage summary. Detailed test sections (executable specs) are added at L5.

### Responsibility Table

| Directory | Responsibility |
|-----------|----------------|
| command/ | Per-command integration test case indices |
| param/ | Per-parameter edge case indices |
| param_group/ | Per-parameter-group interaction test indices |
| type/ | Per-type validation test indices |
| env_param/ | Per-env-parameter edge case indices |

### Coverage Summary

| Scope | Files | Min Tests |
|-------|-------|-----------|
| Commands | 4 | ≥8 IT each |
| Parameters | 27 | ≥6 EC each |
| Parameter groups | 4 | ≥4 CC each |
| Types | 12 | ≥4 TC each |
| Env params | 1 | ≥6 EC each |

### Navigation

#### Commands
- [`run`](command/001_run.md)
- [`help`](command/002_help.md)
- [`isolated`](command/003_isolated.md)
- [`refresh`](command/004_refresh.md)

#### Parameters
- [`[MESSAGE]`](param/001_message.md)
- [`--print`](param/002_print.md)
- [`--model`](param/003_model.md)
- [`--verbose`](param/004_verbose.md)
- [`--no-skip-permissions`](param/005_no_skip_permissions.md)
- [`--interactive`](param/006_interactive.md)
- [`--new-session`](param/007_new_session.md)
- [`--dir`](param/008_dir.md)
- [`--max-tokens`](param/009_max_tokens.md)
- [`--session-dir`](param/010_session_dir.md)
- [`--dry-run`](param/011_dry_run.md)
- [`--verbosity`](param/012_verbosity.md)
- [`--trace`](param/013_trace.md)
- [`--no-ultrathink`](param/014_no_ultrathink.md)
- [`--system-prompt`](param/015_system_prompt.md)
- [`--append-system-prompt`](param/016_append_system_prompt.md)
- [`--effort`](param/017_effort.md)
- [`--no-effort-max`](param/018_no_effort_max.md)
- [`--creds`](param/019_creds.md)
- [`--timeout`](param/020_timeout.md)
- [`--no-chrome`](param/021_no_chrome.md)
- [`--no-persist`](param/022_no_persist.md)
- [`--json-schema`](param/023_json_schema.md)
- [`--mcp-config`](param/024_mcp_config.md)
- [`--file`](param/025_file.md)
- [`--strip-fences`](param/026_strip_fences.md)
- [`--keep-claudecode`](param/027_keep_claudecode.md)

#### Parameter Groups
- [Claude-Native Flags](param_group/001_claude_native_flags.md)
- [Runner Control](param_group/002_runner_control.md)
- [System Prompt](param_group/003_system_prompt.md)
- [Credential Operations](param_group/004_credential_operations.md)

#### Types
- [`MessageText`](type/001_message_text.md)
- [`DirectoryPath`](type/002_directory_path.md)
- [`TokenLimit`](type/003_token_limit.md)
- [`ModelName`](type/004_model_name.md)
- [`VerbosityLevel`](type/005_verbosity_level.md)
- [`SystemPromptText`](type/006_system_prompt_text.md)
- [`EffortLevel`](type/007_effort_level.md)
- [`CredentialsFilePath`](type/008_credentials_file_path.md)
- [`TimeoutSecs`](type/009_timeout_secs.md)
- [`JsonSchemaText`](type/010_json_schema_text.md)
- [`McpConfigPath`](type/011_mcp_config_path.md)
- [`FilePath`](type/012_file_path.md)

#### Env Params
- [`CLAUDE_CODE_MAX_OUTPUT_TOKENS`](env_param/001_max_output_tokens.md)
- [`CLR_* (28 vars)`](env_param/002_clr_input_vars.md)
