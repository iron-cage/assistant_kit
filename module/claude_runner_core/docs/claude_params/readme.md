# claude_params Doc Entity

### Scope

- **Purpose**: Document the builder-API perspective of all ClaudeCommand parameters.
- **Responsibility**: Index of per-parameter reference docs covering all ClaudeCommand with_*() methods and the claude binary flags they wrap.
- **In Scope**: All 70 ClaudeCommand parameters: with_*() methods, types, builder defaults, and underlying binary flag mappings.
- **Out of Scope**: Binary-perspective reference (→ `docs/claude_code/params/`), execution mode design (→ `feature/`).

Builder-API reference for `ClaudeCommand` — documents Rust `with_*()` methods,
builder-specific defaults, and the underlying `claude` binary parameters they wrap.

> **Binary-perspective reference** (actual `claude` flags, env vars, config keys with
> binary defaults): [`docs/claude_code/params/readme.md`](../../../docs/claude_code/params/readme.md).
> This file is the **builder-API perspective** — defaults here are intentionally tuned
> for automation and may differ from the binary defaults shown there.

### Parameter Summary Table

Quick-reference for all 70 parameters (#1–#70). Type: **CLI** = flag only, **Env** = env var only, **Config** = settings.json key only, **Both** = CLI flag + env var. Builder: typed `with_*()` method for all runtime parameters; only deprecated `mcp_debug` uses `with_arg()` fallback. Parameters #61–#70 are settings config keys with no builder method.

### Core Execution

| # | Parameter | Type | CLI Flag | Env Var | Default | Builder | Description |
|---|-----------|------|----------|---------|---------|---------|-------------|
| 1 | [prompt](prompt.md) | CLI | `<prompt>` (positional) | — | — | `with_message()` | Message sent to Claude |
| 2 | [print](print.md) | CLI | `-p` / `--print` | — | off | `with_print()` | Print response and exit; skip TTY |
| 3 | [dry_run](dry_run.md) | Builder | — | — | false | `with_dry_run()` | Inspect command without spawning process |
| 4 | [continue_conversation](continue_conversation.md) | CLI | `-c` / `--continue` | — | off | `with_continue_conversation()` | Continue most recent conversation |
| 5 | [model](model.md) | CLI | `--model <model>` | — | `claude-sonnet-4-6` | `with_model()` | Model alias or full model ID |
| 6 | [verbose](verbose.md) | CLI | `--verbose` | — | off | `with_verbose()` | Override verbose mode from config |

### Authentication

| # | Parameter | Type | CLI Flag | Env Var | Default | Builder | Description |
|---|-----------|------|----------|---------|---------|---------|-------------|
| 7 | [api_key](api_key.md) | Both | `--api-key <key>` | `ANTHROPIC_API_KEY` | — | `with_api_key()` | Anthropic API key |

### System Prompt

| # | Parameter | Type | CLI Flag | Env Var | Default | Builder | Description |
|---|-----------|------|----------|---------|---------|---------|-------------|
| 8 | [system_prompt](system_prompt.md) | CLI | `--system-prompt <prompt>` | — | — | `with_system_prompt()` | Replace default system prompt |
| 9 | [append_system_prompt](append_system_prompt.md) | CLI | `--append-system-prompt <prompt>` | — | — | `with_append_system_prompt()` | Append to default system prompt |

### Permissions

| # | Parameter | Type | CLI Flag | Env Var | Default | Builder | Description |
|---|-----------|------|----------|---------|---------|---------|-------------|
| 10 | [dangerously_skip_permissions](dangerously_skip_permissions.md) | CLI | `--dangerously-skip-permissions` | — | off | `with_skip_permissions()` | Bypass all permission checks |
| 11 | [allow_dangerously_skip_permissions](allow_dangerously_skip_permissions.md) | CLI | `--allow-dangerously-skip-permissions` | — | off | `with_allow_dangerously_skip_permissions()` | Enable skip-permissions as option |
| 12 | [permission_mode](permission_mode.md) | CLI | `--permission-mode <mode>` | — | `default` | `with_permission_mode()` | Fine-grained permission mode |

### Session Management

| # | Parameter | Type | CLI Flag | Env Var | Default | Builder | Description |
|---|-----------|------|----------|---------|---------|---------|-------------|
| 13 | [resume](resume.md) | CLI | `-r` / `--resume [id]` | — | — | `with_resume()` | Resume conversation by session ID |
| 14 | [session_id](session_id.md) | CLI | `--session-id <uuid>` | — | auto | `with_session_id()` | Specify session UUID |
| 15 | [fork_session](fork_session.md) | CLI | `--fork-session` | — | off | `with_fork_session()` | Create new session ID on resume |
| 16 | [no_session_persistence](no_session_persistence.md) | CLI | `--no-session-persistence` | — | off | `with_no_session_persistence()` | Disable save-to-disk |
| 17 | [from_pr](from_pr.md) | CLI | `--from-pr [value]` | — | — | `with_from_pr()` | Resume session linked to PR |

### Tools & Directories

| # | Parameter | Type | CLI Flag | Env Var | Default | Builder | Description |
|---|-----------|------|----------|---------|---------|---------|-------------|
| 18 | [add_dir](add_dir.md) | CLI | `--add-dir <dirs...>` | — | — | `with_add_dir()` | Grant tool access to directories |
| 19 | [allowed_tools](allowed_tools.md) | CLI | `--allowed-tools <tools...>` | — | all | `with_allowed_tools()` | Allowlist of permitted tools |
| 20 | [disallowed_tools](disallowed_tools.md) | CLI | `--disallowed-tools <tools...>` | — | none | `with_disallowed_tools()` | Denylist of forbidden tools |
| 21 | [tools](tools.md) | CLI | `--tools <tools...>` | — | `default` | `with_tools()` | Override full available tool set |

### Input / Output

| # | Parameter | Type | CLI Flag | Env Var | Default | Builder | Description |
|---|-----------|------|----------|---------|---------|---------|-------------|
| 22 | [output_format](output_format.md) | CLI | `--output-format <fmt>` | — | `text` | `with_output_format()` | Response format (text/json/stream-json) |
| 23 | [input_format](input_format.md) | CLI | `--input-format <fmt>` | — | `text` | `with_input_format()` | Input format (text/stream-json) |
| 24 | [include_partial_messages](include_partial_messages.md) | CLI | `--include-partial-messages` | — | off | `with_include_partial_messages()` | Stream partial chunks |
| 25 | [replay_user_messages](replay_user_messages.md) | CLI | `--replay-user-messages` | — | off | `with_replay_user_messages()` | Re-emit user messages on stdout |
| 26 | [json_schema](json_schema.md) | CLI | `--json-schema <schema>` | — | — | `with_json_schema()` | Structured output JSON Schema |

### Model & Budget

| # | Parameter | Type | CLI Flag | Env Var | Default | Builder | Description |
|---|-----------|------|----------|---------|---------|---------|-------------|
| 27 | [effort](effort.md) | CLI | `--effort <level>` | — | `medium` | `with_effort()` | Effort level (low/medium/high/max) |
| 28 | [fallback_model](fallback_model.md) | CLI | `--fallback-model <model>` | — | — | `with_fallback_model()` | Fallback when primary is overloaded |
| 29 | [max_budget_usd](max_budget_usd.md) | CLI | `--max-budget-usd <amount>` | — | — | `with_max_budget_usd()` | Maximum API spend cap in USD |

### MCP & Extensions

| # | Parameter | Type | CLI Flag | Env Var | Default | Builder | Description |
|---|-----------|------|----------|---------|---------|---------|-------------|
| 30 | [mcp_config](mcp_config.md) | CLI | `--mcp-config <configs...>` | — | — | `with_mcp_config()` | Load MCP servers from JSON |
| 31 | [strict_mcp_config](strict_mcp_config.md) | CLI | `--strict-mcp-config` | — | off | `with_strict_mcp_config()` | Ignore all non-`--mcp-config` MCP |
| 32 | [settings](settings.md) | CLI | `--settings <file-or-json>` | — | — | `with_settings()` | Load settings file or JSON |
| 33 | [setting_sources](setting_sources.md) | CLI | `--setting-sources <sources>` | — | all | `with_setting_sources()` | Filter setting sources |
| 34 | [agent](agent.md) | CLI | `--agent <agent>` | — | — | `with_agent()` | Override agent for session |
| 35 | [agents](agents.md) | CLI | `--agents <json>` | — | — | `with_agents()` | Define custom agents as JSON |
| 36 | [plugin_dir](plugin_dir.md) | CLI | `--plugin-dir <paths...>` | — | — | `with_plugin_dir()` | Load plugins from directories |

### Terminal & IDE

| # | Parameter | Type | CLI Flag | Env Var | Default | Builder | Description |
|---|-----------|------|----------|---------|---------|---------|-------------|
| 37 | [worktree](worktree.md) | CLI | `-w` / `--worktree [name]` | — | — | `with_worktree()` | Create git worktree for session |
| 38 | [tmux](tmux.md) | CLI | `--tmux` | — | off | `with_tmux()` | Create tmux session for worktree |
| 39 | [ide](ide.md) | CLI | `--ide` | — | off | `with_ide()` | Auto-connect to IDE on startup |
| 40 | [chrome](chrome.md) | CLI | `--chrome` / `--no-chrome` | — | **on** | `with_chrome()` | Toggle Claude-in-Chrome integration |

### Debug

| # | Parameter | Type | CLI Flag | Env Var | Default | Builder | Description |
|---|-----------|------|----------|---------|---------|---------|-------------|
| 41 | [debug](debug.md) | CLI | `-d` / `--debug [filter]` | — | off | `with_debug()` | Debug mode with optional category filter |
| 42 | [debug_file](debug_file.md) | CLI | `--debug-file <path>` | — | — | `with_debug_file()` | Write debug logs to a file |

### Advanced CLI

| # | Parameter | Type | CLI Flag | Env Var | Default | Builder | Description |
|---|-----------|------|----------|---------|---------|---------|-------------|
| 43 | [betas](betas.md) | CLI | `--betas <betas...>` | — | — | `with_betas()` | Beta API headers (API key users only) |
| 44 | [brief](brief.md) | CLI | `--brief` | — | off | `with_brief()` | Enable SendUserMessage for agents |
| 45 | [disable_slash_commands](disable_slash_commands.md) | CLI | `--disable-slash-commands` | — | off | `with_disable_slash_commands()` | Disable all slash command skills |
| 46 | [file](file.md) | CLI | `--file <specs...>` | — | — | `with_file()` | Download file resources at startup |
| 47 | [mcp_debug](mcp_debug.md) | CLI | `--mcp-debug` | — | off | `with_arg()` | **DEPRECATED** — use `--debug` instead |

### Environment Variables (Builder API)

These parameters are only settable via environment variables. All have dedicated typed builder methods.

| # | Parameter | Type | CLI Flag | Env Var | Builder Default | Builder | Description |
|---|-----------|------|----------|---------|-----------------|---------|-------------|
| 48 | [max_output_tokens](max_output_tokens.md) | Env | — | `CLAUDE_CODE_MAX_OUTPUT_TOKENS` | `200000` | `with_max_output_tokens()` | Max tokens per response |
| 49 | [bash_timeout](bash_timeout.md) | Env | — | `CLAUDE_CODE_BASH_TIMEOUT` | `3600000` | `with_bash_timeout_ms()` | Default bash timeout (ms) |
| 50 | [bash_max_timeout](bash_max_timeout.md) | Env | — | `CLAUDE_CODE_BASH_MAX_TIMEOUT` | `7200000` | `with_bash_max_timeout_ms()` | Max bash timeout (ms) |
| 51 | [auto_continue](auto_continue.md) | Env | — | `CLAUDE_CODE_AUTO_CONTINUE` | `true` | `with_auto_continue()` | Auto-continue without prompts |
| 52 | [telemetry](telemetry.md) | Env | — | `CLAUDE_CODE_TELEMETRY` | `false` | `with_telemetry()` | Send usage telemetry |
| 53 | [auto_approve_tools](auto_approve_tools.md) | Env | — | `CLAUDE_CODE_AUTO_APPROVE_TOOLS` | `false` | `with_auto_approve_tools()` | Auto-approve all tool calls |
| 54 | [action_mode](action_mode.md) | Env | — | `CLAUDE_CODE_ACTION_MODE` | `Ask` | `with_action_mode()` | Tool execution action mode |
| 55 | [log_level](log_level.md) | Env | — | `CLAUDE_CODE_LOG_LEVEL` | `Info` | `with_log_level()` | Log verbosity level |
| 56 | [temperature](temperature.md) | Env | — | `CLAUDE_CODE_TEMPERATURE` | `1.0` | `with_temperature()` | Model temperature (0.0–1.0) |
| 57 | [sandbox_mode](sandbox_mode.md) | Env | — | `CLAUDE_CODE_SANDBOX_MODE` | `true` | `with_sandbox_mode()` | Enable sandbox mode |
| 58 | [session_dir](session_dir.md) | Env | — | `CLAUDE_CODE_SESSION_DIR` | auto | `with_session_dir()` | Override session directory |
| 59 | [top_p](top_p.md) | Env | — | `CLAUDE_CODE_TOP_P` | none | `with_top_p()` | Top-p nucleus sampling (0.0–1.0) |
| 60 | [top_k](top_k.md) | Env | — | `CLAUDE_CODE_TOP_K` | none | `with_top_k()` | Top-k sampling cutoff |

### Settings Config (`~/.claude/settings.json`)

These parameters are read from the settings file on startup. No builder method — managed by `claude_version`. Full write semantics in [005_settings_format.md](../../../docs/claude_code/005_settings_format.md).

| # | Key | Type | Values | Default | Description |
|---|-----|------|--------|---------|-------------|
| 61 | `theme` | Config | `str` | `"dark"` | UI color theme |
| 62 | `autoUpdates` | Config | `bool` | `true` | Auto-update the binary on startup |
| 63 | `preferredVersionSpec` | Config | `str\|null` | `null` | Preferred version alias or semver (e.g. `"stable"`, `"2.1.78"`) |
| 64 | `preferredVersionResolved` | Config | `str\|null` | `null` | Concrete semver resolved at last install; `null` for `latest` |
| 65 | `env` | Config | `object` | `{}` | Persistent env var overrides injected at startup (e.g. `DISABLE_AUTOUPDATER`) |
| 66 | `enabledPlugins` | Config | `object` | `{}` | Active plugin registry |
| 67 | `hooks` | Config | `object` | `{}` | Hooks executed at `PreToolUse` / `PostToolUse` / `UserPromptSubmit` events |
| 68 | `mcpServers` | Config | `object` | `{}` | Inline MCP server definitions (alternative to `--mcp-config`) |
| 69 | `skipDangerousModePermissionPrompt` | Config | `bool` | `false` | Suppress interactive confirmation for dangerous mode |
| 70 | `voiceEnabled` | Config | `bool` | `false` | Enable voice input and audio output |

### Notes

- **Builder defaults vs claude defaults**: `max_output_tokens` (#48), `bash_timeout` (#49), `bash_max_timeout` (#50), `auto_continue` (#51), `telemetry` (#52), and `chrome` (#40) have **different** defaults in `claude_runner_core` than in the `claude` binary. The builder values are tuned for programmatic/automation use. Notably, `chrome` defaults to **on** in the builder (vs off in the raw `claude` binary) so browser context is available by default in automation.
- **`--api-key` removed from CLI**: `api_key` (#7) is listed as `Both` (CLI + env) in this doc, but `--api-key` is no longer present in `claude --help` as of current builds — env var `ANTHROPIC_API_KEY` is the only runtime form. The binary-perspective reference in `docs/claude_code/params/api_key.md` reflects this correctly; this doc retains the builder method which still passes the value via env var internally.
- **Deprecated**: `mcp_debug` (#47) documents `--mcp-debug` which is deprecated in favor of `--debug` (#41).
- **Builder-only**: `dry_run` (#3) is not a `claude` binary parameter — it controls whether `ClaudeCommand` spawns a process or returns `describe_compact()` as stdout.
- **Config vs runtime**: Settings config parameters (#61–#70) are loaded once at startup from `~/.claude/settings.json`; runtime parameters (#1–#60) are passed per-invocation via CLI flags or env vars.
- **Precedence**: CLI arg > env var > settings config.
- **Dual-form params**: `model` (#5) and `effort` (#27) each have both a CLI flag form (with builder method) and a config key form (`model`, `effortLevel`). Similarly, `allowed_tools` (#19), `disallowed_tools` (#20), and `permission_mode` (#12) accept a `allowedTools`/`disallowedTools`/`permissionMode` config key in project-level `.claude/settings.json`. These dual forms are not listed separately in this table; see `docs/claude_code/params/readme.md` for the complete picture.
- **Source**: CLI flags from `claude --help`; env vars from `src/command.rs` `build_command()`; settings keys from `docs/claude_code/005_settings_format.md`.
