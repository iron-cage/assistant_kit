# Testing

### Scope

- **Purpose**: Document integration and edge case test plans for all clr commands, parameters, and types.
- **Responsibility**: Index of per-command, per-parameter, per-type, per-group, and per-env-param test case planning files.
- **In Scope**: All 3 clr commands, all 24 parameters, all 11 types, all 4 parameter groups, 1 env parameter, and test surface for feature/invariant/api doc instances.
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
| Commands | 3 | ≥8 IT each |
| Parameters | 24 | ≥6 EC each |
| Parameter groups | 4 | ≥4 CC each |
| Types | 11 | ≥4 TC each |
| Env params | 1 | ≥6 EC each |

### Navigation

#### Commands
- [`run`](command/01_run.md)
- [`help`](command/02_help.md)
- [`isolated`](command/03_isolated.md)

#### Parameters
- [`[MESSAGE]`](param/01_message.md)
- [`--print`](param/02_print.md)
- [`--model`](param/03_model.md)
- [`--verbose`](param/04_verbose.md)
- [`--no-skip-permissions`](param/05_no_skip_permissions.md)
- [`--interactive`](param/06_interactive.md)
- [`--new-session`](param/07_new_session.md)
- [`--dir`](param/08_dir.md)
- [`--max-tokens`](param/09_max_tokens.md)
- [`--session-dir`](param/10_session_dir.md)
- [`--dry-run`](param/11_dry_run.md)
- [`--verbosity`](param/12_verbosity.md)
- [`--trace`](param/13_trace.md)
- [`--no-ultrathink`](param/14_no_ultrathink.md)
- [`--system-prompt`](param/15_system_prompt.md)
- [`--append-system-prompt`](param/16_append_system_prompt.md)
- [`--effort`](param/17_effort.md)
- [`--no-effort-max`](param/18_no_effort_max.md)
- [`--creds`](param/19_creds.md)
- [`--timeout`](param/20_timeout.md)
- [`--no-chrome`](param/21_no_chrome.md)
- [`--no-persist`](param/22_no_persist.md)
- [`--json-schema`](param/23_json_schema.md)
- [`--mcp-config`](param/24_mcp_config.md)

#### Parameter Groups
- [Claude-Native Flags](param_group/01_claude_native_flags.md)
- [Runner Control](param_group/02_runner_control.md)
- [System Prompt](param_group/03_system_prompt.md)
- [Isolated Subcommand](param_group/04_isolated_subcommand.md)

#### Types
- [`MessageText`](type/01_message_text.md)
- [`DirectoryPath`](type/02_directory_path.md)
- [`TokenLimit`](type/03_token_limit.md)
- [`ModelName`](type/04_model_name.md)
- [`VerbosityLevel`](type/05_verbosity_level.md)
- [`SystemPromptText`](type/06_system_prompt_text.md)
- [`EffortLevel`](type/07_effort_level.md)
- [`CredentialsFilePath`](type/08_credentials_file_path.md)
- [`TimeoutSecs`](type/09_timeout_secs.md)
- [`JsonSchemaText`](type/10_json_schema_text.md)
- [`McpConfigPath`](type/11_mcp_config_path.md)

#### Env Params
- [`CLAUDE_CODE_MAX_OUTPUT_TOKENS`](env_param/01_max_output_tokens.md)
- [`CLR_* (24 vars)`](env_param/02_clr_input_vars.md)
