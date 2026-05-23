# Claude Code: Parameters

All runtime parameters accepted by the `claude` binary — CLI flags, environment
variables, and settings config keys — unified in one flat table. One file per
parameter in this directory.

### Scope

- **Purpose**: Authoritative flat reference for every parameter the `claude` binary accepts at runtime.
- **Responsibility**: Master table and per-parameter detail files for CLI flags, env vars, and settings config keys.
- **In Scope**: All 73 parameters — positional args, long/short flags, `CLAUDE_CODE_*` env vars, `ANTHROPIC_*` env vars, `~/.claude/settings.json` config keys, project-level `.claude/settings.json` config keys.
- **Out of Scope**: Builder-API defaults and Rust `with_*()` methods (→ `module/claude_runner_core/docs/claude_params/`); Claude API protocol (→ Anthropic docs).

### Responsibility Table

| File | Responsibility |
|------|----------------|
| readme.md | Master flat parameter table (this file) |
| 001_action_mode.md | `CLAUDE_CODE_ACTION_MODE` — tool execution mode |
| 002_add_dir.md | `--add-dir` — grant tool access to directories |
| 003_agent.md | `--agent` — override agent for session |
| 004_agents.md | `--agents` — define custom agents as JSON |
| 005_allow_dangerously_skip_permissions.md | `--allow-dangerously-skip-permissions` — enable skip-perms as option |
| 006_allowed_tools.md | `--allowed-tools` — allowlist of permitted tools |
| 007_api_key.md | `ANTHROPIC_API_KEY` — Anthropic API key |
| 008_append_system_prompt.md | `--append-system-prompt` — append to default system prompt |
| 009_auto_approve_tools.md | `CLAUDE_CODE_AUTO_APPROVE_TOOLS` — auto-approve tool calls |
| 010_auto_continue.md | `CLAUDE_CODE_AUTO_CONTINUE` — auto-continue without prompts |
| 011_auto_updates.md | `autoUpdates` config key — auto-update binary |
| 012_bash_max_timeout.md | `CLAUDE_CODE_BASH_MAX_TIMEOUT` — max bash command timeout |
| 013_bash_timeout.md | `CLAUDE_CODE_BASH_TIMEOUT` — default bash command timeout |
| 014_betas.md | `--betas` — beta API headers |
| 015_brief.md | `--brief` — enable SendUserMessage for agents |
| 016_chrome.md | `--chrome` / `--no-chrome` — Claude-in-Chrome integration |
| 017_continue.md | `-c` / `--continue` — continue most recent session |
| 018_dangerously_skip_permissions.md | `--dangerously-skip-permissions` — bypass all permissions |
| 019_debug.md | `-d` / `--debug` — debug mode with category filter |
| 020_debug_file.md | `--debug-file` — write debug logs to file |
| 021_disable_slash_commands.md | `--disable-slash-commands` — disable slash command skills |
| 022_disallowed_tools.md | `--disallowed-tools` — denylist of forbidden tools |
| 023_effort.md | `--effort` — effort level (low/medium/high/max) |
| 024_enabled_plugins.md | `enabledPlugins` config key — active plugin registry |
| 025_env_overrides.md | `env` config key — persistent env var overrides |
| 026_fallback_model.md | `--fallback-model` — fallback when primary model overloaded |
| 027_file.md | `--file` — download file resources at startup |
| 029_fork_session.md | `--fork-session` — new session ID on resume |
| 030_from_pr.md | `--from-pr` — resume session linked to PR |
| 031_hooks.md | `hooks` config key — hooks for tool-use lifecycle events |
| 032_ide.md | `--ide` — auto-connect to IDE on startup |
| 033_include_partial_messages.md | `--include-partial-messages` — stream partial chunks |
| 034_input_format.md | `--input-format` — input format (text/stream-json) |
| 035_json_schema.md | `--json-schema` — JSON Schema for structured output |
| 036_log_level.md | `CLAUDE_CODE_LOG_LEVEL` — log verbosity level |
| 037_max_budget_usd.md | `--max-budget-usd` — max API spend cap in USD |
| 038_max_output_tokens.md | `CLAUDE_CODE_MAX_OUTPUT_TOKENS` — max tokens per response |
| 039_mcp_config.md | `--mcp-config` — load MCP servers from JSON |
| 040_mcp_debug.md | `--mcp-debug` — deprecated; use --debug |
| 041_mcp_servers.md | `mcpServers` config key — inline MCP server definitions |
| 042_model.md | `--model` / `model` config key — model alias or full ID |
| 043_no_session_persistence.md | `--no-session-persistence` — disable save-to-disk |
| 044_output_format.md | `--output-format` — response format (text/json/stream-json) |
| 046_permission_mode.md | `--permission-mode` — fine-grained permission mode |
| 048_plugin_dir.md | `--plugin-dir` — load plugins from directories |
| 049_preferred_version_resolved.md | `preferredVersionResolved` config key — resolved semver |
| 050_preferred_version_spec.md | `preferredVersionSpec` config key — preferred version |
| 051_print.md | `-p` / `--print` — print response and exit |
| 052_prompt.md | `<message>` positional — message sent to Claude |
| 054_replay_user_messages.md | `--replay-user-messages` — re-emit user messages on stdout |
| 055_resume.md | `-r` / `--resume` — resume session by ID |
| 056_sandbox_mode.md | `CLAUDE_CODE_SANDBOX_MODE` — enable sandbox mode |
| 057_session_dir.md | `CLAUDE_CODE_SESSION_DIR` — override session directory |
| 058_session_id.md | `--session-id` — specify session UUID |
| 059_setting_sources.md | `--setting-sources` — filter which config sources load |
| 060_settings.md | `--settings` — load additional settings from file or JSON |
| 061_skip_dangerous_mode_permission_prompt.md | `skipDangerousModePermissionPrompt` — auto-accept dangerous mode prompt |
| 062_strict_mcp_config.md | `--strict-mcp-config` — ignore non-`--mcp-config` MCP sources |
| 063_system_prompt.md | `--system-prompt` — replace default system prompt |
| 064_telemetry.md | `CLAUDE_CODE_TELEMETRY` — send usage telemetry |
| 065_temperature.md | `CLAUDE_CODE_TEMPERATURE` — model temperature |
| 066_theme.md | `theme` config key — UI color theme |
| 067_tmux.md | `--tmux` — create tmux session for worktree |
| 068_tools.md | `--tools` — override full available tool set |
| 069_top_k.md | `CLAUDE_CODE_TOP_K` — top-k sampling cutoff |
| 070_top_p.md | `CLAUDE_CODE_TOP_P` — top-p nucleus sampling |
| 071_verbose.md | `--verbose` — override verbose mode from config |
| 072_voice_enabled.md | `voiceEnabled` config key — voice input/output features |
| 073_worktree.md | `-w` / `--worktree` — create git worktree for session |
| 047_permissions.md | `permissions` config key — per-project tool allow/deny/ask rules |
| 045_output_style.md | `outputStyle` config key — terminal output visual style |
| 028_file_checkpointing_enabled.md | `fileCheckpointingEnabled` config key — file checkpointing before edits |
| 053_remote_control_at_startup.md | `remoteControlAtStartup` config key — remote-control channel on startup |

### Parameter Table

Precedence: CLI arg > env var > settings config.
`—` = that form does not exist for this parameter.

| # | Name | CLI Flag | Env Var | Config Key | Type | Binary Default | Description |
|---|------|----------|---------|------------|------|----------------|-------------|
| 1 | [prompt](052_prompt.md) | `<message>` (positional) | — | — | string | — | Message sent to Claude |
| 2 | [print](051_print.md) | `-p` / `--print` | — | — | bool | off | Print response and exit; skip TTY |
| 3 | [continue](017_continue.md) | `-c` / `--continue` | — | — | bool | off | Continue most recent session |
| 4 | [model](042_model.md) | `--model <model>` | — | `model` | string | `claude-sonnet-4-6` | Model alias or full ID |
| 5 | [verbose](071_verbose.md) | `--verbose` | — | — | bool | off | Override verbose mode from config |
| 6 | [api_key](007_api_key.md) | — | `ANTHROPIC_API_KEY` | — | string | — | Anthropic API key (`--api-key` removed from CLI) |
| 7 | [system_prompt](063_system_prompt.md) | `--system-prompt <prompt>` | — | — | string | — | Replace default system prompt entirely |
| 8 | [append_system_prompt](008_append_system_prompt.md) | `--append-system-prompt <prompt>` | — | — | string | — | Append text to default system prompt |
| 9 | [dangerously_skip_permissions](018_dangerously_skip_permissions.md) | `--dangerously-skip-permissions` | — | — | bool | off | Bypass all permission checks |
| 10 | [allow_dangerously_skip_permissions](005_allow_dangerously_skip_permissions.md) | `--allow-dangerously-skip-permissions` | — | — | bool | off | Enable skip-permissions as option (not default) |
| 11 | [permission_mode](046_permission_mode.md) | `--permission-mode <mode>` | — | `permissionMode` | enum | `default` | `default` `acceptEdits` `bypassPermissions` `dontAsk` `plan` `auto` |
| 12 | [resume](055_resume.md) | `-r` / `--resume [id]` | — | — | string? | — | Resume session by ID; interactive picker if no ID |
| 13 | [session_id](058_session_id.md) | `--session-id <uuid>` | — | — | uuid | auto | Specify session UUID for this run |
| 14 | [fork_session](029_fork_session.md) | `--fork-session` | — | — | bool | off | Create new session ID on resume |
| 15 | [no_session_persistence](043_no_session_persistence.md) | `--no-session-persistence` | — | — | bool | off | Disable save-to-disk; cannot be resumed |
| 16 | [from_pr](030_from_pr.md) | `--from-pr [value]` | — | — | string? | — | Resume session linked to PR by number or URL |
| 17 | [session_dir](057_session_dir.md) | — | `CLAUDE_CODE_SESSION_DIR` | — | path | auto | Override session storage directory |
| 18 | [auto_continue](010_auto_continue.md) | — | `CLAUDE_CODE_AUTO_CONTINUE` | — | bool | false | Auto-continue without prompts |
| 19 | [add_dir](002_add_dir.md) | `--add-dir <dirs...>` | — | — | path[] | — | Grant tool access to additional directories |
| 20 | [allowed_tools](006_allowed_tools.md) | `--allowed-tools <tools...>` | — | `allowedTools` | string[] | all | Allowlist of permitted tools |
| 21 | [disallowed_tools](022_disallowed_tools.md) | `--disallowed-tools <tools...>` | — | `disallowedTools` | string[] | none | Denylist of forbidden tools |
| 22 | [tools](068_tools.md) | `--tools <tools...>` | — | — | string[] | `default` | Override full tool set; `""` disables all |
| 23 | [auto_approve_tools](009_auto_approve_tools.md) | — | `CLAUDE_CODE_AUTO_APPROVE_TOOLS` | — | bool | false | Auto-approve all tool calls without prompting |
| 24 | [action_mode](001_action_mode.md) | — | `CLAUDE_CODE_ACTION_MODE` | — | enum | `Ask` | Tool execution action mode |
| 25 | [output_format](044_output_format.md) | `--output-format <fmt>` | — | — | enum | `text` | `text` `json` `stream-json` |
| 26 | [input_format](034_input_format.md) | `--input-format <fmt>` | — | — | enum | `text` | `text` `stream-json` |
| 27 | [include_partial_messages](033_include_partial_messages.md) | `--include-partial-messages` | — | — | bool | off | Stream partial chunks (requires stream-json) |
| 28 | [replay_user_messages](054_replay_user_messages.md) | `--replay-user-messages` | — | — | bool | off | Re-emit user messages on stdout |
| 29 | [json_schema](035_json_schema.md) | `--json-schema <schema>` | — | — | json | — | JSON Schema for structured output validation |
| 30 | [max_output_tokens](038_max_output_tokens.md) | — | `CLAUDE_CODE_MAX_OUTPUT_TOKENS` | — | integer | 32 000 | Max tokens per response |
| 31 | [effort](023_effort.md) | `--effort <level>` | — | `effortLevel` | enum | `medium` | `low` `medium` `high` `max` |
| 32 | [fallback_model](026_fallback_model.md) | `--fallback-model <model>` | — | — | string | — | Fallback model when primary is overloaded |
| 33 | [max_budget_usd](037_max_budget_usd.md) | `--max-budget-usd <amount>` | — | — | float | — | Max API spend cap in USD (print mode only) |
| 34 | [temperature](065_temperature.md) | — | `CLAUDE_CODE_TEMPERATURE` | — | float | 1.0 | Model temperature (0.0–1.0) |
| 35 | [top_p](070_top_p.md) | — | `CLAUDE_CODE_TOP_P` | — | float | none | Top-p nucleus sampling (0.0–1.0) |
| 36 | [top_k](069_top_k.md) | — | `CLAUDE_CODE_TOP_K` | — | integer | none | Top-k sampling cutoff |
| 37 | [bash_timeout](013_bash_timeout.md) | — | `CLAUDE_CODE_BASH_TIMEOUT` | — | integer ms | 120 000 | Default bash command timeout (2 min) |
| 38 | [bash_max_timeout](012_bash_max_timeout.md) | — | `CLAUDE_CODE_BASH_MAX_TIMEOUT` | — | integer ms | 600 000 | Max allowed bash command timeout (10 min) |
| 39 | [mcp_config](039_mcp_config.md) | `--mcp-config <configs...>` | — | — | json[] | — | Load MCP servers from JSON files or strings |
| 40 | [strict_mcp_config](062_strict_mcp_config.md) | `--strict-mcp-config` | — | — | bool | off | Ignore all non-`--mcp-config` MCP sources |
| 41 | [settings](060_settings.md) | `--settings <file-or-json>` | — | — | path/json | — | Load additional settings from file or JSON string |
| 42 | [setting_sources](059_setting_sources.md) | `--setting-sources <sources>` | — | — | string | all | Filter which sources load: `user` `project` `local` |
| 43 | [agent](003_agent.md) | `--agent <agent>` | — | — | string | — | Override agent for this session |
| 44 | [agents](004_agents.md) | `--agents <json>` | — | — | json | — | Define custom agents as JSON object |
| 45 | [plugin_dir](048_plugin_dir.md) | `--plugin-dir <paths...>` | — | — | path[] | — | Load plugins from directories (session only) |
| 46 | [worktree](073_worktree.md) | `-w` / `--worktree [name]` | — | — | string? | — | Create git worktree for this session |
| 47 | [tmux](067_tmux.md) | `--tmux` | — | — | bool | off | Create tmux session for the worktree |
| 48 | [ide](032_ide.md) | `--ide` | — | — | bool | off | Auto-connect to IDE on startup |
| 49 | [chrome](016_chrome.md) | `--chrome` / `--no-chrome` | — | — | bool | off | Claude-in-Chrome integration |
| 50 | [debug](019_debug.md) | `-d` / `--debug [filter]` | — | — | string? | off | Debug mode; optional category filter e.g. `"api,hooks"` |
| 51 | [debug_file](020_debug_file.md) | `--debug-file <path>` | — | — | path | — | Write debug logs to file (implicitly enables debug) |
| 52 | [log_level](036_log_level.md) | — | `CLAUDE_CODE_LOG_LEVEL` | — | enum | `Info` | `Error` `Warn` `Info` `Debug` `Trace` |
| 53 | [sandbox_mode](056_sandbox_mode.md) | — | `CLAUDE_CODE_SANDBOX_MODE` | — | bool | true | Enable sandbox mode |
| 54 | [telemetry](064_telemetry.md) | — | `CLAUDE_CODE_TELEMETRY` | — | bool | true | Send usage telemetry to Anthropic |
| 55 | [betas](014_betas.md) | `--betas <betas...>` | — | — | string[] | — | Beta API headers (API key auth only) |
| 56 | [brief](015_brief.md) | `--brief` | — | — | bool | off | Enable `SendUserMessage` tool for agents |
| 57 | [disable_slash_commands](021_disable_slash_commands.md) | `--disable-slash-commands` | — | — | bool | off | Disable all slash command skills |
| 58 | [file](027_file.md) | `--file <specs...>` | — | — | string[] | — | Download file resources at startup (`file_id:path`) |
| 59 | [mcp_debug](040_mcp_debug.md) | `--mcp-debug` ⚠️ | — | — | bool | off | **DEPRECATED** — use `--debug` instead |
| 60 | [theme](066_theme.md) | — | — | `theme` | string | `"dark"` | UI color theme |
| 61 | [auto_updates](011_auto_updates.md) | — | — | `autoUpdates` | bool | true | Auto-update binary on startup |
| 62 | [preferred_version_spec](050_preferred_version_spec.md) | — | — | `preferredVersionSpec` | string/null | null | Preferred version alias or semver |
| 63 | [preferred_version_resolved](049_preferred_version_resolved.md) | — | — | `preferredVersionResolved` | string/null | null | Concrete semver resolved at last install |
| 64 | [env_overrides](025_env_overrides.md) | — | — | `env` | object | `{}` | Persistent env vars injected at every startup |
| 65 | [enabled_plugins](024_enabled_plugins.md) | — | — | `enabledPlugins` | object | `{}` | Active plugin registry |
| 66 | [hooks](031_hooks.md) | — | — | `hooks` | object | `{}` | Hooks executed at `PreToolUse` / `PostToolUse` / `UserPromptSubmit` events |
| 67 | [mcp_servers](041_mcp_servers.md) | — | — | `mcpServers` | object | `{}` | Inline MCP server definitions (alternative to `--mcp-config`) |
| 68 | [skip_dangerous_mode_permission_prompt](061_skip_dangerous_mode_permission_prompt.md) | — | — | `skipDangerousModePermissionPrompt` | bool | `false` | Suppress interactive confirmation for dangerous mode |
| 69 | [voice_enabled](072_voice_enabled.md) | — | — | `voiceEnabled` | bool | `false` | Enable voice input and audio output features |
| 70 | [permissions](047_permissions.md) | — | — | `permissions` | object | `{}` | Per-project tool allow/deny/ask rules; auto-managed by Claude Code |
| 71 | [output_style](045_output_style.md) | — | — | `outputStyle` | string | `"default"` | Terminal output visual rendering style |
| 72 | [file_checkpointing_enabled](028_file_checkpointing_enabled.md) | — | — | `fileCheckpointingEnabled` | bool | `false` | Save file checkpoint before each edit |
| 73 | [remote_control_at_startup](053_remote_control_at_startup.md) | — | — | `remoteControlAtStartup` | bool | `false` | Open remote-control channel on startup |

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [`../behavior/readme.md`](../behavior/readme.md) | Claude Code behavior doc entity master file |
| doc | [`../behavior/001_session_behaviors.md`](../behavior/001_session_behaviors.md) | Observed flag behavior in practice |
| doc | [`../behavior/005_settings_format.md`](../behavior/005_settings_format.md) | settings.json write semantics and structure |
| doc | [`../../../../../module/claude_runner_core/docs/claude_params/readme.md`](../../../../../module/claude_runner_core/docs/claude_params/readme.md) | Builder-API perspective with Rust `with_*()` methods |
