# Doc Entities

### Master Doc Entities Table

| Type | Purpose | Master File | Instances |
|------|---------|-------------|----------:|
| `api/` | Index of API doc instances covering COMMANDS_YAML, VerbosityLevel, and register_commands contracts | [api/readme.md](api/readme.md) | 1 |
| `cli/` | Index of CLI reference instances covering commands, params, types, groups, dictionary, user stories, env params | [cli/readme.md](cli/readme.md) | 5 |
| `cli/param/` | Index of individual parameter reference instances (36 parameters) | [cli/param/readme.md](cli/param/readme.md) | 36 |
| `cli/user_story/` | Index of user goal and usage pattern instances (26 user stories) | [cli/user_story/readme.md](cli/user_story/readme.md) | 26 |
| `feature/` | Index of feature doc instances covering the clr binary tool design | [feature/readme.md](feature/readme.md) | 1 |
| `invariant/` | Index of invariant doc instances covering default flag injection, dependency constraints, command naming, trace universality, and isolated/refresh subprocess defaults | [invariant/readme.md](invariant/readme.md) | 5 |

### Master Doc Instances Table

| Entity | ID | Name | File |
|--------|----|------|------|
| api | 001 | Public API | [api/001_public_api.md](api/001_public_api.md) |
| cli | — | Commands | [cli/command/](cli/command/readme.md) |
| cli | — | Dictionary | [cli/dictionary.md](cli/dictionary.md) |
| cli | — | Environment Parameters | [cli/env_param.md](cli/env_param.md) |
| cli | — | Parameter Groups | [cli/param_group/](cli/param_group/readme.md) |
| cli | — | Types | [cli/type/](cli/type/readme.md) |
| cli/param | 001 | [MESSAGE] | [cli/param/001_message.md](cli/param/001_message.md) |
| cli/param | 002 | --print | [cli/param/002_print.md](cli/param/002_print.md) |
| cli/param | 003 | --model | [cli/param/003_model.md](cli/param/003_model.md) |
| cli/param | 004 | --verbose | [cli/param/004_verbose.md](cli/param/004_verbose.md) |
| cli/param | 005 | --no-skip-permissions | [cli/param/005_no_skip_permissions.md](cli/param/005_no_skip_permissions.md) |
| cli/param | 006 | --interactive | [cli/param/006_interactive.md](cli/param/006_interactive.md) |
| cli/param | 007 | --new-session | [cli/param/007_new_session.md](cli/param/007_new_session.md) |
| cli/param | 008 | --dir | [cli/param/008_dir.md](cli/param/008_dir.md) |
| cli/param | 009 | --max-tokens | [cli/param/009_max_tokens.md](cli/param/009_max_tokens.md) |
| cli/param | 010 | --session-dir | [cli/param/010_session_dir.md](cli/param/010_session_dir.md) |
| cli/param | 011 | --dry-run | [cli/param/011_dry_run.md](cli/param/011_dry_run.md) |
| cli/param | 012 | --verbosity | [cli/param/012_verbosity.md](cli/param/012_verbosity.md) |
| cli/param | 013 | --trace | [cli/param/013_trace.md](cli/param/013_trace.md) |
| cli/param | 014 | --no-ultrathink | [cli/param/014_no_ultrathink.md](cli/param/014_no_ultrathink.md) |
| cli/param | 015 | --system-prompt | [cli/param/015_system_prompt.md](cli/param/015_system_prompt.md) |
| cli/param | 016 | --append-system-prompt | [cli/param/016_append_system_prompt.md](cli/param/016_append_system_prompt.md) |
| cli/param | 017 | --effort | [cli/param/017_effort.md](cli/param/017_effort.md) |
| cli/param | 018 | --no-effort-max | [cli/param/018_no_effort_max.md](cli/param/018_no_effort_max.md) |
| cli/param | 019 | --creds | [cli/param/019_creds.md](cli/param/019_creds.md) |
| cli/param | 020 | --timeout | [cli/param/020_timeout.md](cli/param/020_timeout.md) |
| cli/param | 021 | --no-chrome | [cli/param/021_no_chrome.md](cli/param/021_no_chrome.md) |
| cli/param | 022 | --no-persist | [cli/param/022_no_persist.md](cli/param/022_no_persist.md) |
| cli/param | 023 | --json-schema | [cli/param/023_json_schema.md](cli/param/023_json_schema.md) |
| cli/param | 024 | --mcp-config | [cli/param/024_mcp_config.md](cli/param/024_mcp_config.md) |
| cli/param | 025 | --file | [cli/param/025_file.md](cli/param/025_file.md) |
| cli/param | 026 | --strip-fences | [cli/param/026_strip_fences.md](cli/param/026_strip_fences.md) |
| cli/param | 027 | --keep-claudecode | [cli/param/027_keep_claudecode.md](cli/param/027_keep_claudecode.md) |
| cli/param | 028 | --subdir | [cli/param/028_subdir.md](cli/param/028_subdir.md) |
| cli/param | 029 | --output-file | [cli/param/029_output_file.md](cli/param/029_output_file.md) |
| cli/param | 030 | --expect | [cli/param/030_expect.md](cli/param/030_expect.md) |
| cli/param | 031 | --expect-strategy | [cli/param/031_expect_strategy.md](cli/param/031_expect_strategy.md) |
| cli/param | 032 | --expect-retries | [cli/param/032_expect_retries.md](cli/param/032_expect_retries.md) |
| cli/param | 033 | --max-sessions | [cli/param/033_max_sessions.md](cli/param/033_max_sessions.md) |
| cli/param | 034 | --retry-on-rate-limit | [cli/param/034_retry_on_rate_limit.md](cli/param/034_retry_on_rate_limit.md) |
| cli/param | 035 | --retry-delay | [cli/param/035_retry_delay.md](cli/param/035_retry_delay.md) |
| cli/param | 036 | --timeout (run/ask) | [cli/param/036_timeout.md](cli/param/036_timeout.md) |
| cli/user_story | 001 | Interactive REPL | [cli/user_story/001_interactive_repl.md](cli/user_story/001_interactive_repl.md) |
| cli/user_story | 002 | Print Mode Capture | [cli/user_story/002_print_mode_capture.md](cli/user_story/002_print_mode_capture.md) |
| cli/user_story | 003 | Interactive With Message | [cli/user_story/003_interactive_with_message.md](cli/user_story/003_interactive_with_message.md) |
| cli/user_story | 004 | Dry-run Preview | [cli/user_story/004_dry_run_preview.md](cli/user_story/004_dry_run_preview.md) |
| cli/user_story | 005 | Project-specific Execution | [cli/user_story/005_project_specific_execution.md](cli/user_story/005_project_specific_execution.md) |
| cli/user_story | 006 | Verbose Debugging | [cli/user_story/006_verbose_debugging.md](cli/user_story/006_verbose_debugging.md) |
| cli/user_story | 007 | Fresh Session | [cli/user_story/007_fresh_session.md](cli/user_story/007_fresh_session.md) |
| cli/user_story | 008 | Trace Execution | [cli/user_story/008_trace_execution.md](cli/user_story/008_trace_execution.md) |
| cli/user_story | 009 | Custom System Prompt | [cli/user_story/009_custom_system_prompt.md](cli/user_story/009_custom_system_prompt.md) |
| cli/user_story | 010 | Credential-isolated Execution | [cli/user_story/010_credential_isolated_execution.md](cli/user_story/010_credential_isolated_execution.md) |
| cli/user_story | 011 | File Input | [cli/user_story/011_file_input.md](cli/user_story/011_file_input.md) |
| cli/user_story | 012 | Code Block Extraction | [cli/user_story/012_code_block_extraction.md](cli/user_story/012_code_block_extraction.md) |
| cli/user_story | 013 | Structured JSON Pipeline | [cli/user_story/013_structured_json_pipeline.md](cli/user_story/013_structured_json_pipeline.md) |
| cli/user_story | 014 | Credential Refresh | [cli/user_story/014_credential_refresh.md](cli/user_story/014_credential_refresh.md) |
| cli/user_story | 015 | Ask Mode | [cli/user_story/015_ask_mode.md](cli/user_story/015_ask_mode.md) |
| cli/user_story | 016 | CLI Discoverability | [cli/user_story/016_cli_discoverability.md](cli/user_story/016_cli_discoverability.md) |
| cli/user_story | 017 | Model Selection | [cli/user_story/017_model_selection.md](cli/user_story/017_model_selection.md) |
| cli/user_story | 018 | Env-var Configuration | [cli/user_story/018_env_var_configuration.md](cli/user_story/018_env_var_configuration.md) |
| cli/user_story | 019 | MCP Config Injection | [cli/user_story/019_mcp_config_injection.md](cli/user_story/019_mcp_config_injection.md) |
| cli/user_story | 020 | Suppress Effort Max | [cli/user_story/020_suppress_effort_max.md](cli/user_story/020_suppress_effort_max.md) |
| cli/user_story | 021 | Keep ClaudeCode Context | [cli/user_story/021_keep_claudecode_context.md](cli/user_story/021_keep_claudecode_context.md) |
| cli/user_story | 022 | Session Isolation via Subdirectory | [cli/user_story/022_session_isolation_subdir.md](cli/user_story/022_session_isolation_subdir.md) |
| cli/user_story | 023 | Output File Capture | [cli/user_story/023_output_file_capture.md](cli/user_story/023_output_file_capture.md) |
| cli/user_story | 024 | Enum Output Validation | [cli/user_story/024_enum_output_validation.md](cli/user_story/024_enum_output_validation.md) |
| cli/user_story | 025 | Session Concurrency Gate | [cli/user_story/025_concurrency_gate.md](cli/user_story/025_concurrency_gate.md) |
| cli/user_story | 026 | Session Listing | [cli/user_story/026_session_listing.md](cli/user_story/026_session_listing.md) |
| feature | 001 | Runner Tool | [feature/001_runner_tool.md](feature/001_runner_tool.md) |
| invariant | 001 | Default Flags | [invariant/001_default_flags.md](invariant/001_default_flags.md) |
| invariant | 002 | Dependency Constraints | [invariant/002_dep_constraints.md](invariant/002_dep_constraints.md) |
| invariant | 003 | Command Naming | [invariant/003_command_naming.md](invariant/003_command_naming.md) |
| invariant | 004 | Trace Universality | [invariant/004_trace_universality.md](invariant/004_trace_universality.md) |
| invariant | 005 | Isolated Subprocess Defaults | [invariant/005_isolated_subprocess_defaults.md](invariant/005_isolated_subprocess_defaults.md) |
