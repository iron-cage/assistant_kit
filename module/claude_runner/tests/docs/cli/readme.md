# Testing

### Scope

- **Purpose**: Document integration and edge case test plans for all clr commands, parameters, and types.
- **Responsibility**: Index of per-command, per-parameter, per-type, per-group, and per-env-param test case planning files.
- **In Scope**: All 8 clr commands, all 69 parameters, all 14 types, all 5 parameter groups, 2 env parameter specs, 27 user story specs, 1 dictionary vocabulary check, and test surface for feature/invariant/api doc instances.
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
| user_story/ | Per-user-story end-to-end workflow test specs |
| dictionary.md | Dictionary vocabulary completeness and accuracy checks |

### Coverage Summary

| Scope | Files | Min Tests |
|-------|-------|-----------|
| Commands | 8 | ≥8 IT each |
| Parameters | 69 | ≥6 EC each |
| Parameter groups | 5 | ≥4 CC each |
| Types | 14 | ≥4 TC each |
| Env params | 2 | ≥6 EC each |
| User stories | 27 | ≥4 US each |
| Dictionary | 1 | ≥4 DT total |

### Navigation

#### Commands
- [`run`](command/01_run.md)
- [`help`](command/02_help.md)
- [`isolated`](command/03_isolated.md)
- [`refresh`](command/04_refresh.md)
- [`ask`](command/05_ask.md)
- [`ps`](command/06_ps.md)
- [`kill`](command/07_kill.md)
- [`tools`](command/08_tools.md)

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
- [`--timeout` (isolated/refresh)](param/20_timeout.md)
- [`--no-chrome`](param/21_no_chrome.md)
- [`--no-persist`](param/22_no_persist.md)
- [`--json-schema`](param/23_json_schema.md)
- [`--mcp-config`](param/24_mcp_config.md)
- [`--file`](param/25_file.md)
- [`--strip-fences`](param/26_strip_fences.md)
- [`--keep-claudecode`](param/27_keep_claudecode.md)
- [`--subdir`](param/28_subdir.md)
- [`--output-file`](param/29_output_file.md)
- [`--expect`](param/30_expect.md)
- [`--expect-strategy`](param/31_expect_strategy.md)
- [`--max-sessions`](param/33_max_sessions.md)
- [`--retry-on-transient`](param/34_retry_on_transient.md)
- [`--transient-delay`](param/35_transient_delay.md)
- [`--timeout` (run/ask)](param/36_timeout.md)
- [`--retry-on-account`](param/040_retry_on_account.md)
- [`--account-delay`](param/041_account_delay.md)
- [`--retry-on-auth`](param/042_retry_on_auth.md)
- [`--auth-delay`](param/043_auth_delay.md)
- [`--retry-on-service`](param/044_retry_on_service.md)
- [`--service-delay`](param/045_service_delay.md)
- [`--retry-on-process`](param/046_retry_on_process.md)
- [`--process-delay`](param/047_process_delay.md)
- [`--retry-on-validation`](param/048_retry_on_validation.md)
- [`--validation-delay`](param/049_validation_delay.md)
- [`--retry-on-runner`](param/050_retry_on_runner.md)
- [`--runner-delay`](param/051_runner_delay.md)
- [`--retry-on-unknown`](param/052_retry_on_unknown.md)
- [`--unknown-delay`](param/053_unknown_delay.md)
- [`--retry-override`](param/054_retry_override.md)
- [`--retry-override-delay`](param/055_retry_override_delay.md)
- [`--retry-default`](param/056_retry_default.md)
- [`--retry-default-delay`](param/057_retry_default_delay.md)
- [`--mode`](param/058_mode.md)
- [`--columns`](param/059_columns.md)
- [`--wide`](param/060_wide.md)
- [`--pid`](param/068_pid.md)
- [`--inspect`](param/069_inspect.md)
- [`--output-format`](param/061_output_format.md)
- [`--max-turns`](param/062_max_turns.md)
- [`--allowed-tools`](param/063_allowed_tools.md)
- [`--disallowed-tools`](param/064_disallowed_tools.md)
- [`--max-budget-usd`](param/065_max_budget_usd.md)
- [`--add-dir`](param/066_add_dir.md)
- [`--fallback-model`](param/067_fallback_model.md)
- [`--output-style`](param/070_output_style.md)
- [`--summary-fields`](param/071_summary_fields.md)
- [`--journal`](param/072_journal.md)
- [`--journal-dir`](param/073_journal_dir.md)

#### Parameter Groups
- [Claude-Native Flags](param_group/01_claude_native_flags.md)
- [Runner Control](param_group/02_runner_control.md)
- [System Prompt](param_group/03_system_prompt.md)
- [Credential Operations](param_group/04_credential_operations.md)
- [Session Listing](param_group/05_session_listing.md)

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
- [`FilePath`](type/12_file_path.md)
- [`ErrorKind`](type/13_error_kind.md)
- [`ErrorClass`](type/14_error_class.md)

#### User Stories
- [Interactive REPL](user_story/01_interactive_repl.md)
- [Print Mode Capture](user_story/02_print_mode_capture.md)
- [Interactive With Message](user_story/03_interactive_with_message.md)
- [Dry-run Preview](user_story/04_dry_run_preview.md)
- [Project-specific Execution](user_story/05_project_specific_execution.md)
- [Verbose Debugging](user_story/06_verbose_debugging.md)
- [Fresh Session](user_story/07_fresh_session.md)
- [Trace Execution](user_story/08_trace_execution.md)
- [Custom System Prompt](user_story/09_custom_system_prompt.md)
- [Credential-isolated Execution](user_story/10_credential_isolated_execution.md)
- [File Input](user_story/11_file_input.md)
- [Code Block Extraction](user_story/12_code_block_extraction.md)
- [Structured JSON Pipeline](user_story/13_structured_json_pipeline.md)
- [Credential Refresh](user_story/14_credential_refresh.md)
- [Ask Mode](user_story/15_ask_mode.md)
- [CLI Discoverability](user_story/16_cli_discoverability.md)
- [Model Selection](user_story/17_model_selection.md)
- [Env-var Configuration](user_story/18_env_var_configuration.md)
- [MCP Config Injection](user_story/19_mcp_config_injection.md)
- [Suppress Effort Max](user_story/20_suppress_effort_max.md)
- [Keep ClaudeCode Context](user_story/21_keep_claudecode_context.md)
- [Session Isolation via Subdirectory](user_story/22_session_isolation_subdir.md)
- [Output File Capture](user_story/23_output_file_capture.md)
- [Enum Output Validation](user_story/24_enum_output_validation.md)
- [Session Concurrency Gate](user_story/25_concurrency_gate.md)
- [Session Listing](user_story/26_session_listing.md)
- [Session Termination](user_story/27_session_termination.md)

#### Env Params
- [`CLAUDE_CODE_MAX_OUTPUT_TOKENS`](env_param/01_max_output_tokens.md)
- [`CLR_* (69 vars)`](env_param/02_clr_input_vars.md)
