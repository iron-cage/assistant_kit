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
| 015_ask_mode.md | User story: `ask` as pure semantic alias for `run` |
| 016_cli_discoverability.md | User story: discover CLI commands and usage via help |
| 017_model_selection.md | User story: select the Claude model for an invocation |
| 018_env_var_configuration.md | User story: configure clr defaults via CLR_* env vars |
| 019_mcp_config_injection.md | User story: forward MCP server configs to subprocess |
| 020_suppress_effort_max.md | User story: suppress automatic --effort max injection |
| 021_keep_claudecode_context.md | User story: preserve CLAUDECODE in subprocess env |
| 022_session_isolation_subdir.md | User story: named workspace session isolation via --subdir |
| 023_output_file_capture.md | User story: tee captured stdout to a file with --output-file |
| 024_enum_output_validation.md | User story: validate print-mode output against a fixed enum with --expect |
| 025_concurrency_gate.md | User story: limit concurrent Claude Code sessions with --max-sessions |
| 026_session_listing.md | User story: list running Claude Code sessions with clr ps |
| 027_session_termination.md | User story: terminate a Claude Code session by PID with clr kill |

### Index

| # | Title | Primary Flags | Command |
|---|-------|---------------|---------|
| 001 | Interactive REPL | (none) | `run` |
| 002 | Print Mode Capture | `[MESSAGE]`, `--print` | `run` |
| 003 | Interactive With Message | `--interactive` | `run` |
| 004 | Dry-run Preview | `--dry-run` | `run` |
| 005 | Project-specific Execution | `--dir`, `--session-dir` | `run` |
| 006 | Quiet Mode and Diagnostic Control | `--quiet` | `run` |
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
| 022 | Session Isolation via Subdirectory | `--subdir` | `run`, `ask` |
| 023 | Output File Capture | `--output-file` | `run`, `ask` |
| 024 | Enum Output Validation | `--expect`, `--expect-strategy`, `--retry-on-validation` | `run`, `ask` |
| 025 | Session Concurrency Gate | `--max-sessions` | `run`, `ask` |
| 026 | Session Listing | (none) | `ps` |
| 027 | Session Termination | `<PID>` | `kill` |

### Adding User Stories

When adding new user stories, update these files in order:

1. `docs/cli/user_story/NNN_*.md` — feature doc (Scope, Persona, Goal, ACs, cross-refs)
2. `tests/docs/cli/user_story/NNN_*.md` — test spec (4 US cases)
3. Appropriate test file (US01–09: `tests/user_story_test.rs`; US10–18: `tests/user_story_creds_isolated_test.rs`; US19–25: `tests/user_story_output_test.rs`) — 4 test functions + matrix row in module doc
4. `docs/cli/user_story/readme.md` — Responsibility Table row + Index row *(this file)*
5. `tests/docs/cli/user_story/readme.md` — Responsibility Table row + In Scope count
6. `docs/cli/param/NNN_*.md` — Referenced User Stories for each param involved
7. `docs/entity.md` — count in header row + new instance row
8. `docs/cli/readme.md` — count in 4 places (Responsibility Table, Completion Matrix ×2, Navigation)
9. `tests/readme.md` — count in 2 places (Domain Map, Responsibility Table)
10. `tests/docs/cli/readme.md` — count in Scope, Coverage Summary, nav links
11. `docs/cli/param_group/01_claude_native_flags.md` — Referenced User Stories (if story uses Claude-Native flags: `--model`, `--mcp-config`, etc.)
12. `docs/cli/param_group/02_runner_control.md` — Referenced User Stories (if story uses Runner Control flags: `--dry-run`, `--no-effort-max`, etc.)
13. Existing related story files — Related User Stories back-references

**Note:** The phrase "YES for all 42" in `docs/cli/param_group/02_runner_control.md` refers to the 42 Runner Control *parameters* in that group — do not update this count when fixing user story totals.
