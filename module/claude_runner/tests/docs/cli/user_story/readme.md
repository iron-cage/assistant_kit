# User Story Tests

### Scope

- **Purpose**: Test case specs for user story doc instances in `docs/cli/user_story/`.
- **Responsibility**: Per-user-story test spec files covering end-to-end user workflows.
- **In Scope**: All 31 user stories: Interactive REPL, Print Mode Capture, Interactive With Message, Dry-run Preview, Project-specific Execution, Verbose Debugging, Fresh Session, Trace Execution, Custom System Prompt, Credential-isolated Execution, File Input, Code Block Extraction, Structured JSON Pipeline, Credential Refresh, Ask Mode, CLI Discoverability, Model Selection, Env-var Configuration, MCP Config Injection, Suppress Effort Max, Keep ClaudeCode Context, Session Isolation via Topic Directory, Output File Capture, Enum Output Validation, Session Concurrency Gate, Session Listing, Session Termination, Session Cross-Loading, Scope Inspection, Topic Creation, Topic Discovery.
- **Out of Scope**: Parameter-level edge cases (-> `param/`), command-level integration (-> `command/`), type validation (-> `type/`).

### Responsibility Table

| Name | Purpose | Status |
|------|---------|--------|
| procedure.md | Workflow for creating and updating user_story test specs | ✅ |
| [001_interactive_repl.md](001_interactive_repl.md) | `user_story` spec for Interactive REPL | ✅ |
| [002_print_mode_capture.md](002_print_mode_capture.md) | `user_story` spec for Print Mode Capture | ✅ |
| [003_interactive_with_message.md](003_interactive_with_message.md) | `user_story` spec for Interactive With Message | ✅ |
| [004_dry_run_preview.md](004_dry_run_preview.md) | `user_story` spec for Dry-run Preview | ✅ |
| [005_project_specific_execution.md](005_project_specific_execution.md) | `user_story` spec for Project-specific Execution | ✅ |
| [006_verbose_debugging.md](006_verbose_debugging.md) | `user_story` spec for Verbose Debugging | ✅ |
| [007_fresh_session.md](007_fresh_session.md) | `user_story` spec for Fresh Session | ✅ |
| [008_trace_execution.md](008_trace_execution.md) | `user_story` spec for Trace Execution | ✅ |
| [009_custom_system_prompt.md](009_custom_system_prompt.md) | `user_story` spec for Custom System Prompt | ✅ |
| [010_credential_isolated_execution.md](010_credential_isolated_execution.md) | `user_story` spec for Credential-isolated Execution | ✅ |
| [011_file_input.md](011_file_input.md) | `user_story` spec for File Input | ✅ |
| [012_code_block_extraction.md](012_code_block_extraction.md) | `user_story` spec for Code Block Extraction | ✅ |
| [013_structured_json_pipeline.md](013_structured_json_pipeline.md) | `user_story` spec for Structured JSON Pipeline | ✅ |
| [014_credential_refresh.md](014_credential_refresh.md) | `user_story` spec for Credential Refresh | ✅ |
| [015_ask_mode.md](015_ask_mode.md) | `user_story` spec for Ask Mode | ✅ |
| [016_cli_discoverability.md](016_cli_discoverability.md) | `user_story` spec for CLI Discoverability | ✅ |
| [017_model_selection.md](017_model_selection.md) | `user_story` spec for Model Selection | ✅ |
| [018_env_var_configuration.md](018_env_var_configuration.md) | `user_story` spec for Env-var Configuration | ✅ |
| [019_mcp_config_injection.md](019_mcp_config_injection.md) | `user_story` spec for MCP Config Injection | ✅ |
| [020_suppress_effort_max.md](020_suppress_effort_max.md) | `user_story` spec for Suppress Effort Max | ✅ |
| [021_keep_claudecode_context.md](021_keep_claudecode_context.md) | `user_story` spec for Keep ClaudeCode Context | ✅ |
| [022_session_isolation_topic.md](022_session_isolation_topic.md) | `user_story` spec for Session Isolation via Topic Directory | ✅ |
| [023_output_file_capture.md](023_output_file_capture.md) | `user_story` spec for Output File Capture | ✅ |
| [024_enum_output_validation.md](024_enum_output_validation.md) | `user_story` spec for Enum Output Validation | ✅ |
| [025_concurrency_gate.md](025_concurrency_gate.md) | `user_story` spec for Session Concurrency Gate | ✅ |
| [026_session_listing.md](026_session_listing.md) | `user_story` spec for Session Listing | ✅ |
| [027_session_termination.md](027_session_termination.md) | `user_story` spec for Session Termination | ✅ |
| [028_session_transplant.md](028_session_transplant.md) | `user_story` spec for Session Cross-Loading (Transplant) | ✅ |
| [029_scope_inspection.md](029_scope_inspection.md) | `user_story` spec for Scope Inspection | ✅ |
| [030_topic_creation.md](030_topic_creation.md) | `user_story` spec for Topic Creation | ✅ |
| [031_topic_discovery.md](031_topic_discovery.md) | `user_story` spec for Topic Discovery | ✅ |
