# User Stories

### Scope

- **Purpose**: Document canonical user goals for the `clr` CLI.
- **Responsibility**: Enumerate the user intents that drive `clr` design: what users want to accomplish and when the feature is considered done.
- **In Scope**: All meaningful usage patterns addressable by a single `clr` invocation or mode.
- **Out of Scope**: Implementation internals (→ `feature/001_runner_tool.md`), parameter semantics (→ `cli/param/`), type constraints (→ `cli/type/`).

### Responsibility Table

| File | Responsibility |
|------|----------------|
| 001_interactive_repl.md | User story: open interactive REPL with session continuation |
| 002_print_mode_capture.md | User story: capture Claude output for scripting or piping |
| 003_interactive_with_message.md | User story: TTY passthrough with an initial prompt |
| 004_dry_run_preview.md | User story: inspect assembled command without executing |
| 005_project_specific_execution.md | User story: run Claude in a specific project directory |
| 006_verbose_debugging.md | User story: increase diagnostic output to troubleshoot runner |
| 007_fresh_session.md | User story: start a new conversation without prior context |
| 008_trace_execution.md | User story: print the command to stderr then execute |
| 009_custom_system_prompt.md | User story: replace or extend the default system prompt |
| 010_credential_isolated_execution.md | User story: run Claude with a separate credentials file |
| 011_file_input.md | User story: pipe a file's content as subprocess stdin |
| 012_code_block_extraction.md | User story: strip code fence from captured output |
| 013_structured_json_pipeline.md | User story: generate schema-constrained JSON for downstream tools |
| 014_credential_refresh.md | User story: refresh OAuth credentials without running a task |
| 015_ask_mode.md | User story: quick Q&A with lightweight defaults |
| 016_cli_discoverability.md | User story: discover CLI commands and usage via help |
| 017_model_selection.md | User story: select the Claude model for an invocation |
| 018_env_var_configuration.md | User story: configure clr defaults via CLR_* env vars |
| 019_mcp_config_injection.md | User story: forward MCP server configs to subprocess |
| 020_suppress_effort_max.md | User story: suppress automatic --effort max injection |
| 021_keep_claudecode_context.md | User story: preserve CLAUDECODE in subprocess env |

### Index

| # | Title | Primary Flags | Command |
|---|-------|---------------|---------|
| 001 | Interactive REPL | (none) | `run` |
| 002 | Print Mode Capture | `[MESSAGE]`, `--print` | `run` |
| 003 | Interactive With Message | `--interactive` | `run` |
| 004 | Dry-run Preview | `--dry-run` | `run` |
| 005 | Project-specific Execution | `--dir`, `--session-dir` | `run` |
| 006 | Verbose Debugging | `--verbosity` | `run` |
| 007 | Fresh Session | `--new-session` | `run` |
| 008 | Trace Execution | `--trace` | `run` |
| 009 | Custom System Prompt | `--system-prompt`, `--append-system-prompt` | `run` |
| 010 | Credential-isolated Execution | `--creds`, `--timeout` | `isolated` |
| 011 | File Input | `--file` | `run` |
| 012 | Code Block Extraction | `--strip-fences` | `run` |
| 013 | Structured JSON Pipeline | `--json-schema`, `--strip-fences` | `run` |
| 014 | Credential Refresh | `--creds`, `--timeout`, `--trace` | `refresh` |
| 015 | Ask Mode | `[MESSAGE]`, `--effort`, `--max-tokens` | `ask` |
| 016 | CLI Discoverability | (none) | `help` |
| 017 | Model Selection | `--model` | `run`, `ask` |
| 018 | Env-var Configuration | (env vars) | `run` |
| 019 | MCP Config Injection | `--mcp-config` | `run` |
| 020 | Suppress Effort Max | `--no-effort-max` | `run` |
| 021 | Keep ClaudeCode Context | `--keep-claudecode` | `run` |
