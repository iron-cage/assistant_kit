# claude_params/

Comprehensive reference for every parameter the `claude` binary accepts — CLI flags, environment variables, and combined options.

## Parameter Summary Table

Quick-reference for all 59 parameters (+1 builder-only). Type: **CLI** = flag only, **Env** = env var only, **Both** = CLI flag + env var. Builder: typed `with_*()` method for all parameters; only deprecated `mcp_debug` uses `with_arg()` fallback.

### Core Execution

| Parameter | Type | CLI Flag | Env Var | Default | Builder | Description |
|-----------|------|----------|---------|---------|---------|-------------|
| [prompt](prompt.md) | CLI | `<prompt>` (positional) | — | — | `with_message()` | Message sent to Claude |
| [print](print.md) | CLI | `-p` / `--print` | — | off | `with_print()` | Print response and exit; skip TTY |
| [dry_run](dry_run.md) | Builder | — | — | false | `with_dry_run()` | Inspect command without spawning process |
| [continue_conversation](continue_conversation.md) | CLI | `-c` / `--continue` | — | off | `with_continue_conversation()` | Continue most recent conversation |
| [model](model.md) | CLI | `--model <model>` | — | `claude-sonnet-4-6` | `with_model()` | Model alias or full model ID |
| [verbose](verbose.md) | CLI | `--verbose` | — | off | `with_verbose()` | Override verbose mode from config |

### Authentication

| Parameter | Type | CLI Flag | Env Var | Default | Builder | Description |
|-----------|------|----------|---------|---------|---------|-------------|
| [api_key](api_key.md) | Both | `--api-key <key>` | `ANTHROPIC_API_KEY` | — | `with_api_key()` | Anthropic API key |

### System Prompt

| Parameter | Type | CLI Flag | Env Var | Default | Builder | Description |
|-----------|------|----------|---------|---------|---------|-------------|
| [system_prompt](system_prompt.md) | CLI | `--system-prompt <prompt>` | — | — | `with_system_prompt()` | Replace default system prompt |
| [append_system_prompt](append_system_prompt.md) | CLI | `--append-system-prompt <prompt>` | — | — | `with_append_system_prompt()` | Append to default system prompt |

### Permissions

| Parameter | Type | CLI Flag | Env Var | Default | Builder | Description |
|-----------|------|----------|---------|---------|---------|-------------|
| [dangerously_skip_permissions](dangerously_skip_permissions.md) | CLI | `--dangerously-skip-permissions` | — | off | `with_skip_permissions()` | Bypass all permission checks |
| [allow_dangerously_skip_permissions](allow_dangerously_skip_permissions.md) | CLI | `--allow-dangerously-skip-permissions` | — | off | `with_allow_dangerously_skip_permissions()` | Enable skip-permissions as option |
| [permission_mode](permission_mode.md) | CLI | `--permission-mode <mode>` | — | `default` | `with_permission_mode()` | Fine-grained permission mode |

### Session Management

| Parameter | Type | CLI Flag | Env Var | Default | Builder | Description |
|-----------|------|----------|---------|---------|---------|-------------|
| [resume](resume.md) | CLI | `-r` / `--resume [id]` | — | — | `with_resume()` | Resume conversation by session ID |
| [session_id](session_id.md) | CLI | `--session-id <uuid>` | — | auto | `with_session_id()` | Specify session UUID |
| [fork_session](fork_session.md) | CLI | `--fork-session` | — | off | `with_fork_session()` | Create new session ID on resume |
| [no_session_persistence](no_session_persistence.md) | CLI | `--no-session-persistence` | — | off | `with_no_session_persistence()` | Disable save-to-disk |
| [from_pr](from_pr.md) | CLI | `--from-pr [value]` | — | — | `with_from_pr()` | Resume session linked to PR |

### Tools & Directories

| Parameter | Type | CLI Flag | Env Var | Default | Builder | Description |
|-----------|------|----------|---------|---------|---------|-------------|
| [add_dir](add_dir.md) | CLI | `--add-dir <dirs...>` | — | — | `with_add_dir()` | Grant tool access to directories |
| [allowed_tools](allowed_tools.md) | CLI | `--allowed-tools <tools...>` | — | all | `with_allowed_tools()` | Allowlist of permitted tools |
| [disallowed_tools](disallowed_tools.md) | CLI | `--disallowed-tools <tools...>` | — | none | `with_disallowed_tools()` | Denylist of forbidden tools |
| [tools](tools.md) | CLI | `--tools <tools...>` | — | `default` | `with_tools()` | Override full available tool set |

### Input / Output

| Parameter | Type | CLI Flag | Env Var | Default | Builder | Description |
|-----------|------|----------|---------|---------|---------|-------------|
| [output_format](output_format.md) | CLI | `--output-format <fmt>` | — | `text` | `with_output_format()` | Response format (text/json/stream-json) |
| [input_format](input_format.md) | CLI | `--input-format <fmt>` | — | `text` | `with_input_format()` | Input format (text/stream-json) |
| [include_partial_messages](include_partial_messages.md) | CLI | `--include-partial-messages` | — | off | `with_include_partial_messages()` | Stream partial chunks |
| [replay_user_messages](replay_user_messages.md) | CLI | `--replay-user-messages` | — | off | `with_replay_user_messages()` | Re-emit user messages on stdout |
| [json_schema](json_schema.md) | CLI | `--json-schema <schema>` | — | — | `with_json_schema()` | Structured output JSON Schema |

### Model & Budget

| Parameter | Type | CLI Flag | Env Var | Default | Builder | Description |
|-----------|------|----------|---------|---------|---------|-------------|
| [effort](effort.md) | CLI | `--effort <level>` | — | `medium` | `with_effort()` | Effort level (low/medium/high/max) |
| [fallback_model](fallback_model.md) | CLI | `--fallback-model <model>` | — | — | `with_fallback_model()` | Fallback when primary is overloaded |
| [max_budget_usd](max_budget_usd.md) | CLI | `--max-budget-usd <amount>` | — | — | `with_max_budget_usd()` | Maximum API spend cap in USD |

### MCP & Extensions

| Parameter | Type | CLI Flag | Env Var | Default | Builder | Description |
|-----------|------|----------|---------|---------|---------|-------------|
| [mcp_config](mcp_config.md) | CLI | `--mcp-config <configs...>` | — | — | `with_mcp_config()` | Load MCP servers from JSON |
| [strict_mcp_config](strict_mcp_config.md) | CLI | `--strict-mcp-config` | — | off | `with_strict_mcp_config()` | Ignore all non-`--mcp-config` MCP |
| [settings](settings.md) | CLI | `--settings <file-or-json>` | — | — | `with_settings()` | Load settings file or JSON |
| [setting_sources](setting_sources.md) | CLI | `--setting-sources <sources>` | — | all | `with_setting_sources()` | Filter setting sources |
| [agent](agent.md) | CLI | `--agent <agent>` | — | — | `with_agent()` | Override agent for session |
| [agents](agents.md) | CLI | `--agents <json>` | — | — | `with_agents()` | Define custom agents as JSON |
| [plugin_dir](plugin_dir.md) | CLI | `--plugin-dir <paths...>` | — | — | `with_plugin_dir()` | Load plugins from directories |

### Terminal & IDE

| Parameter | Type | CLI Flag | Env Var | Default | Builder | Description |
|-----------|------|----------|---------|---------|---------|-------------|
| [worktree](worktree.md) | CLI | `-w` / `--worktree [name]` | — | — | `with_worktree()` | Create git worktree for session |
| [tmux](tmux.md) | CLI | `--tmux` | — | off | `with_tmux()` | Create tmux session for worktree |
| [ide](ide.md) | CLI | `--ide` | — | off | `with_ide()` | Auto-connect to IDE on startup |
| [chrome](chrome.md) | CLI | `--chrome` / `--no-chrome` | — | **on** | `with_chrome()` | Toggle Claude-in-Chrome integration |

### Debug

| Parameter | Type | CLI Flag | Env Var | Default | Builder | Description |
|-----------|------|----------|---------|---------|---------|-------------|
| [debug](debug.md) | CLI | `-d` / `--debug [filter]` | — | off | `with_debug()` | Debug mode with optional category filter |
| [debug_file](debug_file.md) | CLI | `--debug-file <path>` | — | — | `with_debug_file()` | Write debug logs to a file |

### Advanced CLI

| Parameter | Type | CLI Flag | Env Var | Default | Builder | Description |
|-----------|------|----------|---------|---------|---------|-------------|
| [betas](betas.md) | CLI | `--betas <betas...>` | — | — | `with_betas()` | Beta API headers (API key users only) |
| [brief](brief.md) | CLI | `--brief` | — | off | `with_brief()` | Enable SendUserMessage for agents |
| [disable_slash_commands](disable_slash_commands.md) | CLI | `--disable-slash-commands` | — | off | `with_disable_slash_commands()` | Disable all slash command skills |
| [file](file.md) | CLI | `--file <specs...>` | — | — | `with_file()` | Download file resources at startup |
| [mcp_debug](mcp_debug.md) | CLI | `--mcp-debug` | — | off | `with_arg()` | **DEPRECATED** — use `--debug` instead |

### Environment Variables (Builder API)

These parameters are only settable via environment variables. All have dedicated typed builder methods.

| Parameter | Type | CLI Flag | Env Var | Builder Default | Builder | Description |
|-----------|------|----------|---------|-----------------|---------|-------------|
| [max_output_tokens](max_output_tokens.md) | Env | — | `CLAUDE_CODE_MAX_OUTPUT_TOKENS` | `200000` | `with_max_output_tokens()` | Max tokens per response |
| [bash_timeout](bash_timeout.md) | Env | — | `CLAUDE_CODE_BASH_TIMEOUT` | `3600000` | `with_bash_timeout_ms()` | Default bash timeout (ms) |
| [bash_max_timeout](bash_max_timeout.md) | Env | — | `CLAUDE_CODE_BASH_MAX_TIMEOUT` | `7200000` | `with_bash_max_timeout_ms()` | Max bash timeout (ms) |
| [auto_continue](auto_continue.md) | Env | — | `CLAUDE_CODE_AUTO_CONTINUE` | `true` | `with_auto_continue()` | Auto-continue without prompts |
| [telemetry](telemetry.md) | Env | — | `CLAUDE_CODE_TELEMETRY` | `false` | `with_telemetry()` | Send usage telemetry |
| [auto_approve_tools](auto_approve_tools.md) | Env | — | `CLAUDE_CODE_AUTO_APPROVE_TOOLS` | `false` | `with_auto_approve_tools()` | Auto-approve all tool calls |
| [action_mode](action_mode.md) | Env | — | `CLAUDE_CODE_ACTION_MODE` | `Ask` | `with_action_mode()` | Tool execution action mode |
| [log_level](log_level.md) | Env | — | `CLAUDE_CODE_LOG_LEVEL` | `Info` | `with_log_level()` | Log verbosity level |
| [temperature](temperature.md) | Env | — | `CLAUDE_CODE_TEMPERATURE` | `1.0` | `with_temperature()` | Model temperature (0.0–1.0) |
| [sandbox_mode](sandbox_mode.md) | Env | — | `CLAUDE_CODE_SANDBOX_MODE` | `true` | `with_sandbox_mode()` | Enable sandbox mode |
| [session_dir](session_dir.md) | Env | — | `CLAUDE_CODE_SESSION_DIR` | auto | `with_session_dir()` | Override session directory |
| [top_p](top_p.md) | Env | — | `CLAUDE_CODE_TOP_P` | none | `with_top_p()` | Top-p nucleus sampling (0.0–1.0) |
| [top_k](top_k.md) | Env | — | `CLAUDE_CODE_TOP_K` | none | `with_top_k()` | Top-k sampling cutoff |

## Notes

- **Builder defaults vs claude defaults**: `max_output_tokens`, `bash_timeout`, `bash_max_timeout`, `auto_continue`, `telemetry`, and `chrome` have **different** defaults in `claude_runner_core` than in the `claude` binary. The builder values are tuned for programmatic/automation use. Notably, `chrome` defaults to **on** in the builder (vs off in the raw `claude` binary) so browser context is available by default in automation.
- **Combined parameters**: [`api_key`](api_key.md) documents both `--api-key` CLI flag and `ANTHROPIC_API_KEY` env var in one place since they configure the same thing.
- **Deprecated**: [`mcp_debug`](mcp_debug.md) documents `--mcp-debug` which is deprecated in favor of `--debug`.
- **Builder-only**: `dry_run` is not a `claude` binary parameter — it controls whether `ClaudeCommand` spawns a process or returns `describe_compact()` as stdout.
- **Source**: CLI flags from `claude --help`; env vars from `src/command.rs` `build_command()`.
