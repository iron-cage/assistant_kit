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
| action_mode.md | `CLAUDE_CODE_ACTION_MODE` — tool execution mode |
| add_dir.md | `--add-dir` — grant tool access to directories |
| agent.md | `--agent` — override agent for session |
| agents.md | `--agents` — define custom agents as JSON |
| allow_dangerously_skip_permissions.md | `--allow-dangerously-skip-permissions` — enable skip-perms as option |
| allowed_tools.md | `--allowed-tools` — allowlist of permitted tools |
| api_key.md | `ANTHROPIC_API_KEY` — Anthropic API key |
| append_system_prompt.md | `--append-system-prompt` — append to default system prompt |
| auto_approve_tools.md | `CLAUDE_CODE_AUTO_APPROVE_TOOLS` — auto-approve tool calls |
| auto_continue.md | `CLAUDE_CODE_AUTO_CONTINUE` — auto-continue without prompts |
| auto_updates.md | `autoUpdates` config key — auto-update binary |
| bash_max_timeout.md | `CLAUDE_CODE_BASH_MAX_TIMEOUT` — max bash command timeout |
| bash_timeout.md | `CLAUDE_CODE_BASH_TIMEOUT` — default bash command timeout |
| betas.md | `--betas` — beta API headers |
| brief.md | `--brief` — enable SendUserMessage for agents |
| chrome.md | `--chrome` / `--no-chrome` — Claude-in-Chrome integration |
| continue.md | `-c` / `--continue` — continue most recent session |
| dangerously_skip_permissions.md | `--dangerously-skip-permissions` — bypass all permissions |
| debug.md | `-d` / `--debug` — debug mode with category filter |
| debug_file.md | `--debug-file` — write debug logs to file |
| disable_slash_commands.md | `--disable-slash-commands` — disable slash command skills |
| disallowed_tools.md | `--disallowed-tools` — denylist of forbidden tools |
| effort.md | `--effort` — effort level (low/medium/high/max) |
| enabled_plugins.md | `enabledPlugins` config key — active plugin registry |
| env_overrides.md | `env` config key — persistent env var overrides |
| fallback_model.md | `--fallback-model` — fallback when primary model overloaded |
| file.md | `--file` — download file resources at startup |
| fork_session.md | `--fork-session` — new session ID on resume |
| from_pr.md | `--from-pr` — resume session linked to PR |
| hooks.md | `hooks` config key — hooks for tool-use lifecycle events |
| ide.md | `--ide` — auto-connect to IDE on startup |
| include_partial_messages.md | `--include-partial-messages` — stream partial chunks |
| input_format.md | `--input-format` — input format (text/stream-json) |
| json_schema.md | `--json-schema` — JSON Schema for structured output |
| log_level.md | `CLAUDE_CODE_LOG_LEVEL` — log verbosity level |
| max_budget_usd.md | `--max-budget-usd` — max API spend cap in USD |
| max_output_tokens.md | `CLAUDE_CODE_MAX_OUTPUT_TOKENS` — max tokens per response |
| mcp_config.md | `--mcp-config` — load MCP servers from JSON |
| mcp_debug.md | `--mcp-debug` — deprecated; use --debug |
| mcp_servers.md | `mcpServers` config key — inline MCP server definitions |
| model.md | `--model` / `model` config key — model alias or full ID |
| no_session_persistence.md | `--no-session-persistence` — disable save-to-disk |
| output_format.md | `--output-format` — response format (text/json/stream-json) |
| permission_mode.md | `--permission-mode` — fine-grained permission mode |
| plugin_dir.md | `--plugin-dir` — load plugins from directories |
| preferred_version_resolved.md | `preferredVersionResolved` config key — resolved semver |
| preferred_version_spec.md | `preferredVersionSpec` config key — preferred version |
| print.md | `-p` / `--print` — print response and exit |
| prompt.md | `<message>` positional — message sent to Claude |
| replay_user_messages.md | `--replay-user-messages` — re-emit user messages on stdout |
| resume.md | `-r` / `--resume` — resume session by ID |
| sandbox_mode.md | `CLAUDE_CODE_SANDBOX_MODE` — enable sandbox mode |
| session_dir.md | `CLAUDE_CODE_SESSION_DIR` — override session directory |
| session_id.md | `--session-id` — specify session UUID |
| setting_sources.md | `--setting-sources` — filter which config sources load |
| settings.md | `--settings` — load additional settings from file or JSON |
| skip_dangerous_mode_permission_prompt.md | `skipDangerousModePermissionPrompt` — auto-accept dangerous mode prompt |
| strict_mcp_config.md | `--strict-mcp-config` — ignore non-`--mcp-config` MCP sources |
| system_prompt.md | `--system-prompt` — replace default system prompt |
| telemetry.md | `CLAUDE_CODE_TELEMETRY` — send usage telemetry |
| temperature.md | `CLAUDE_CODE_TEMPERATURE` — model temperature |
| theme.md | `theme` config key — UI color theme |
| tmux.md | `--tmux` — create tmux session for worktree |
| tools.md | `--tools` — override full available tool set |
| top_k.md | `CLAUDE_CODE_TOP_K` — top-k sampling cutoff |
| top_p.md | `CLAUDE_CODE_TOP_P` — top-p nucleus sampling |
| verbose.md | `--verbose` — override verbose mode from config |
| voice_enabled.md | `voiceEnabled` config key — voice input/output features |
| worktree.md | `-w` / `--worktree` — create git worktree for session |
| permissions.md | `permissions` config key — per-project tool allow/deny/ask rules |
| output_style.md | `outputStyle` config key — terminal output visual style |
| file_checkpointing_enabled.md | `fileCheckpointingEnabled` config key — file checkpointing before edits |
| remote_control_at_startup.md | `remoteControlAtStartup` config key — remote-control channel on startup |

### Parameter Table

Precedence: CLI arg > env var > settings config.
`—` = that form does not exist for this parameter.

| # | Name | CLI Flag | Env Var | Config Key | Type | Binary Default | Description |
|---|------|----------|---------|------------|------|----------------|-------------|
| 1 | [prompt](prompt.md) | `<message>` (positional) | — | — | string | — | Message sent to Claude |
| 2 | [print](print.md) | `-p` / `--print` | — | — | bool | off | Print response and exit; skip TTY |
| 3 | [continue](continue.md) | `-c` / `--continue` | — | — | bool | off | Continue most recent session |
| 4 | [model](model.md) | `--model <model>` | — | `model` | string | `claude-sonnet-4-6` | Model alias or full ID |
| 5 | [verbose](verbose.md) | `--verbose` | — | — | bool | off | Override verbose mode from config |
| 6 | [api_key](api_key.md) | — | `ANTHROPIC_API_KEY` | — | string | — | Anthropic API key (`--api-key` removed from CLI) |
| 7 | [system_prompt](system_prompt.md) | `--system-prompt <prompt>` | — | — | string | — | Replace default system prompt entirely |
| 8 | [append_system_prompt](append_system_prompt.md) | `--append-system-prompt <prompt>` | — | — | string | — | Append text to default system prompt |
| 9 | [dangerously_skip_permissions](dangerously_skip_permissions.md) | `--dangerously-skip-permissions` | — | — | bool | off | Bypass all permission checks |
| 10 | [allow_dangerously_skip_permissions](allow_dangerously_skip_permissions.md) | `--allow-dangerously-skip-permissions` | — | — | bool | off | Enable skip-permissions as option (not default) |
| 11 | [permission_mode](permission_mode.md) | `--permission-mode <mode>` | — | `permissionMode` | enum | `default` | `default` `acceptEdits` `bypassPermissions` `dontAsk` `plan` `auto` |
| 12 | [resume](resume.md) | `-r` / `--resume [id]` | — | — | string? | — | Resume session by ID; interactive picker if no ID |
| 13 | [session_id](session_id.md) | `--session-id <uuid>` | — | — | uuid | auto | Specify session UUID for this run |
| 14 | [fork_session](fork_session.md) | `--fork-session` | — | — | bool | off | Create new session ID on resume |
| 15 | [no_session_persistence](no_session_persistence.md) | `--no-session-persistence` | — | — | bool | off | Disable save-to-disk; cannot be resumed |
| 16 | [from_pr](from_pr.md) | `--from-pr [value]` | — | — | string? | — | Resume session linked to PR by number or URL |
| 17 | [session_dir](session_dir.md) | — | `CLAUDE_CODE_SESSION_DIR` | — | path | auto | Override session storage directory |
| 18 | [auto_continue](auto_continue.md) | — | `CLAUDE_CODE_AUTO_CONTINUE` | — | bool | false | Auto-continue without prompts |
| 19 | [add_dir](add_dir.md) | `--add-dir <dirs...>` | — | — | path[] | — | Grant tool access to additional directories |
| 20 | [allowed_tools](allowed_tools.md) | `--allowed-tools <tools...>` | — | `allowedTools` | string[] | all | Allowlist of permitted tools |
| 21 | [disallowed_tools](disallowed_tools.md) | `--disallowed-tools <tools...>` | — | `disallowedTools` | string[] | none | Denylist of forbidden tools |
| 22 | [tools](tools.md) | `--tools <tools...>` | — | — | string[] | `default` | Override full tool set; `""` disables all |
| 23 | [auto_approve_tools](auto_approve_tools.md) | — | `CLAUDE_CODE_AUTO_APPROVE_TOOLS` | — | bool | false | Auto-approve all tool calls without prompting |
| 24 | [action_mode](action_mode.md) | — | `CLAUDE_CODE_ACTION_MODE` | — | enum | `Ask` | Tool execution action mode |
| 25 | [output_format](output_format.md) | `--output-format <fmt>` | — | — | enum | `text` | `text` `json` `stream-json` |
| 26 | [input_format](input_format.md) | `--input-format <fmt>` | — | — | enum | `text` | `text` `stream-json` |
| 27 | [include_partial_messages](include_partial_messages.md) | `--include-partial-messages` | — | — | bool | off | Stream partial chunks (requires stream-json) |
| 28 | [replay_user_messages](replay_user_messages.md) | `--replay-user-messages` | — | — | bool | off | Re-emit user messages on stdout |
| 29 | [json_schema](json_schema.md) | `--json-schema <schema>` | — | — | json | — | JSON Schema for structured output validation |
| 30 | [max_output_tokens](max_output_tokens.md) | — | `CLAUDE_CODE_MAX_OUTPUT_TOKENS` | — | integer | 32 000 | Max tokens per response |
| 31 | [effort](effort.md) | `--effort <level>` | — | `effortLevel` | enum | `medium` | `low` `medium` `high` `max` |
| 32 | [fallback_model](fallback_model.md) | `--fallback-model <model>` | — | — | string | — | Fallback model when primary is overloaded |
| 33 | [max_budget_usd](max_budget_usd.md) | `--max-budget-usd <amount>` | — | — | float | — | Max API spend cap in USD (print mode only) |
| 34 | [temperature](temperature.md) | — | `CLAUDE_CODE_TEMPERATURE` | — | float | 1.0 | Model temperature (0.0–1.0) |
| 35 | [top_p](top_p.md) | — | `CLAUDE_CODE_TOP_P` | — | float | none | Top-p nucleus sampling (0.0–1.0) |
| 36 | [top_k](top_k.md) | — | `CLAUDE_CODE_TOP_K` | — | integer | none | Top-k sampling cutoff |
| 37 | [bash_timeout](bash_timeout.md) | — | `CLAUDE_CODE_BASH_TIMEOUT` | — | integer ms | 120 000 | Default bash command timeout (2 min) |
| 38 | [bash_max_timeout](bash_max_timeout.md) | — | `CLAUDE_CODE_BASH_MAX_TIMEOUT` | — | integer ms | 600 000 | Max allowed bash command timeout (10 min) |
| 39 | [mcp_config](mcp_config.md) | `--mcp-config <configs...>` | — | — | json[] | — | Load MCP servers from JSON files or strings |
| 40 | [strict_mcp_config](strict_mcp_config.md) | `--strict-mcp-config` | — | — | bool | off | Ignore all non-`--mcp-config` MCP sources |
| 41 | [settings](settings.md) | `--settings <file-or-json>` | — | — | path/json | — | Load additional settings from file or JSON string |
| 42 | [setting_sources](setting_sources.md) | `--setting-sources <sources>` | — | — | string | all | Filter which sources load: `user` `project` `local` |
| 43 | [agent](agent.md) | `--agent <agent>` | — | — | string | — | Override agent for this session |
| 44 | [agents](agents.md) | `--agents <json>` | — | — | json | — | Define custom agents as JSON object |
| 45 | [plugin_dir](plugin_dir.md) | `--plugin-dir <paths...>` | — | — | path[] | — | Load plugins from directories (session only) |
| 46 | [worktree](worktree.md) | `-w` / `--worktree [name]` | — | — | string? | — | Create git worktree for this session |
| 47 | [tmux](tmux.md) | `--tmux` | — | — | bool | off | Create tmux session for the worktree |
| 48 | [ide](ide.md) | `--ide` | — | — | bool | off | Auto-connect to IDE on startup |
| 49 | [chrome](chrome.md) | `--chrome` / `--no-chrome` | — | — | bool | off | Claude-in-Chrome integration |
| 50 | [debug](debug.md) | `-d` / `--debug [filter]` | — | — | string? | off | Debug mode; optional category filter e.g. `"api,hooks"` |
| 51 | [debug_file](debug_file.md) | `--debug-file <path>` | — | — | path | — | Write debug logs to file (implicitly enables debug) |
| 52 | [log_level](log_level.md) | — | `CLAUDE_CODE_LOG_LEVEL` | — | enum | `Info` | `Error` `Warn` `Info` `Debug` `Trace` |
| 53 | [sandbox_mode](sandbox_mode.md) | — | `CLAUDE_CODE_SANDBOX_MODE` | — | bool | true | Enable sandbox mode |
| 54 | [telemetry](telemetry.md) | — | `CLAUDE_CODE_TELEMETRY` | — | bool | true | Send usage telemetry to Anthropic |
| 55 | [betas](betas.md) | `--betas <betas...>` | — | — | string[] | — | Beta API headers (API key auth only) |
| 56 | [brief](brief.md) | `--brief` | — | — | bool | off | Enable `SendUserMessage` tool for agents |
| 57 | [disable_slash_commands](disable_slash_commands.md) | `--disable-slash-commands` | — | — | bool | off | Disable all slash command skills |
| 58 | [file](file.md) | `--file <specs...>` | — | — | string[] | — | Download file resources at startup (`file_id:path`) |
| 59 | [mcp_debug](mcp_debug.md) | `--mcp-debug` ⚠️ | — | — | bool | off | **DEPRECATED** — use `--debug` instead |
| 60 | [theme](theme.md) | — | — | `theme` | string | `"dark"` | UI color theme |
| 61 | [auto_updates](auto_updates.md) | — | — | `autoUpdates` | bool | true | Auto-update binary on startup |
| 62 | [preferred_version_spec](preferred_version_spec.md) | — | — | `preferredVersionSpec` | string/null | null | Preferred version alias or semver |
| 63 | [preferred_version_resolved](preferred_version_resolved.md) | — | — | `preferredVersionResolved` | string/null | null | Concrete semver resolved at last install |
| 64 | [env_overrides](env_overrides.md) | — | — | `env` | object | `{}` | Persistent env vars injected at every startup |
| 65 | [enabled_plugins](enabled_plugins.md) | — | — | `enabledPlugins` | object | `{}` | Active plugin registry |
| 66 | [hooks](hooks.md) | — | — | `hooks` | object | `{}` | Hooks executed at `PreToolUse` / `PostToolUse` / `UserPromptSubmit` events |
| 67 | [mcp_servers](mcp_servers.md) | — | — | `mcpServers` | object | `{}` | Inline MCP server definitions (alternative to `--mcp-config`) |
| 68 | [skip_dangerous_mode_permission_prompt](skip_dangerous_mode_permission_prompt.md) | — | — | `skipDangerousModePermissionPrompt` | bool | `false` | Suppress interactive confirmation for dangerous mode |
| 69 | [voice_enabled](voice_enabled.md) | — | — | `voiceEnabled` | bool | `false` | Enable voice input and audio output features |
| 70 | [permissions](permissions.md) | — | — | `permissions` | object | `{}` | Per-project tool allow/deny/ask rules; auto-managed by Claude Code |
| 71 | [output_style](output_style.md) | — | — | `outputStyle` | string | `"default"` | Terminal output visual rendering style |
| 72 | [file_checkpointing_enabled](file_checkpointing_enabled.md) | — | — | `fileCheckpointingEnabled` | bool | `false` | Save file checkpoint before each edit |
| 73 | [remote_control_at_startup](remote_control_at_startup.md) | — | — | `remoteControlAtStartup` | bool | `false` | Open remote-control channel on startup |

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [`../readme.md`](../readme.md) | Claude Code doc entity master file |
| doc | [`../001_session_behaviors.md`](../001_session_behaviors.md) | Observed flag behavior in practice |
| doc | [`../005_settings_format.md`](../005_settings_format.md) | settings.json write semantics and structure |
| doc | [`../../../../module/claude_runner_core/docs/claude_params/readme.md`](../../../../module/claude_runner_core/docs/claude_params/readme.md) | Builder-API perspective with Rust `with_*()` methods |
