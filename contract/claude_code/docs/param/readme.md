# Param Doc Entity

All runtime parameters accepted by the `claude` binary — CLI flags, environment
variables, and settings config keys — unified in one flat table. One file per
parameter in this directory.

### Scope

- **Purpose**: Authoritative flat reference for every parameter the `claude` binary accepts at runtime.
- **Responsibility**: Master table and per-parameter detail files for CLI flags, env vars, and settings config keys.
- **In Scope**: All 140 parameters — positional args, long/short flags, `CLAUDE_CODE_*` env vars, `ANTHROPIC_*` env vars, `MCP_*` env vars, `API_*` env vars, `CLAUDE_CLIENT_*` env vars, `BASH_*` env vars, `DISABLE_*` env vars, `~/.claude/settings.json` config keys, project-level `.claude/settings.json` config keys, `managed-settings.json` config keys.
- **Out of Scope**: Builder-API defaults and Rust `with_*()` methods (→ `module/claude_runner_core/docs/claude_param/`); Claude API protocol (→ Anthropic docs).

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
| 074_auto_compact_window.md | `CLAUDE_CODE_AUTO_COMPACT_WINDOW` — context window size for compaction calculations |
| 075_autocompact_pct_override.md | `CLAUDE_AUTOCOMPACT_PCT_OVERRIDE` — compaction trigger as percentage of window |
| 076_max_turns.md | `--max-turns` — max agentic turns per session |
| 077_safe_mode.md | `--safe-mode` / `CLAUDE_CODE_SAFE_MODE` — disable bundled skills and experimental features |
| 078_disable_bundled_skills.md | `CLAUDE_CODE_DISABLE_BUNDLED_SKILLS` / `disableBundledSkills` — disable bundled slash command skills |
| 079_subagent_model.md | `CLAUDE_CODE_SUBAGENT_MODEL` — model override for subagent sessions |
| 080_experimental_agent_teams.md | `CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS` — experimental multi-agent teams |
| 081_enable_auto_mode.md | `CLAUDE_CODE_ENABLE_AUTO_MODE` — enable auto-mode permission classifier |
| 082_tmpdir.md | `CLAUDE_CODE_TMPDIR` — custom temporary directory |
| 083_stop_hook_block_cap.md | `CLAUDE_CODE_STOP_HOOK_BLOCK_CAP` — max consecutive hook blocks |
| 084_ps_execution_policy.md | `CLAUDE_CODE_POWERSHELL_RESPECT_EXECUTION_POLICY` — PowerShell execution policy |
| 085_default_sonnet_model.md | `ANTHROPIC_DEFAULT_SONNET_MODEL` — override sonnet alias target |
| 086_client_presence_file.md | `CLAUDE_CLIENT_PRESENCE_FILE` — IDE client presence file path |
| 087_workspace_id.md | `ANTHROPIC_WORKSPACE_ID` — Anthropic workspace ID |
| 088_plugin_prefer_https.md | `CLAUDE_CODE_PLUGIN_PREFER_HTTPS` — prefer HTTPS plugin transport |
| 089_mcp_tool_timeout.md | `MCP_TOOL_TIMEOUT` — MCP tool invocation timeout |
| 090_api_force_idle_timeout.md | `API_FORCE_IDLE_TIMEOUT` — force API connection idle timeout |
| 091_language.md | `language` config key — UI language |
| 092_worktree_bg_isolation.md | `worktree.bgIsolation` config key — worktree background isolation |
| 093_sandbox_allow_apple_events.md | `sandbox.allowAppleEvents` config key — allow Apple Events in sandbox |
| 094_thinking_disabled_display.md | `thinking.disabled.display` config key — thinking indicator when disabled |
| 095_wheel_scroll_accel.md | `wheelScrollAccelerationEnabled` config key — mouse wheel scroll acceleration |
| 096_bash_default_timeout_ms.md | `BASH_DEFAULT_TIMEOUT_MS` — default Bash tool timeout |
| 097_bash_max_output_length.md | `BASH_MAX_OUTPUT_LENGTH` — max chars before file save |
| 098_bash_max_timeout_ms.md | `BASH_MAX_TIMEOUT_MS` — max model-requested timeout |
| 099_disable_autoupdater.md | `DISABLE_AUTOUPDATER` — disable background auto-updates |
| 100_disable_auto_compact.md | `DISABLE_AUTO_COMPACT` — disable auto-compaction |
| 101_disable_compact.md | `DISABLE_COMPACT` — disable all compaction |
| 102_disable_cost_warnings.md | `DISABLE_COST_WARNINGS` — disable cost warnings |
| 103_disable_doctor_command.md | `DISABLE_DOCTOR_COMMAND` — hide /doctor command |
| 104_disable_error_reporting.md | `DISABLE_ERROR_REPORTING` — opt out of Sentry |
| 105_disable_extra_usage_command.md | `DISABLE_EXTRA_USAGE_COMMAND` — hide /usage-credits |
| 106_disable_feedback_command.md | `DISABLE_FEEDBACK_COMMAND` — disable /feedback |
| 107_disable_growthbook.md | `DISABLE_GROWTHBOOK` — disable feature flags |
| 108_disable_installation_checks.md | `DISABLE_INSTALLATION_CHECKS` — disable install warnings |
| 109_disable_install_github_app_command.md | `DISABLE_INSTALL_GITHUB_APP_COMMAND` — hide /install-github-app |
| 110_disable_interleaved_thinking.md | `DISABLE_INTERLEAVED_THINKING` — disable interleaved thinking |
| 111_disable_login_command.md | `DISABLE_LOGIN_COMMAND` — hide /login |
| 112_disable_logout_command.md | `DISABLE_LOGOUT_COMMAND` — hide /logout |
| 113_disable_prompt_caching.md | `DISABLE_PROMPT_CACHING` — disable all prompt caching |
| 114_disable_prompt_caching_fable.md | `DISABLE_PROMPT_CACHING_FABLE` — disable Fable caching |
| 115_disable_prompt_caching_haiku.md | `DISABLE_PROMPT_CACHING_HAIKU` — disable Haiku caching |
| 116_disable_prompt_caching_opus.md | `DISABLE_PROMPT_CACHING_OPUS` — disable Opus caching |
| 117_disable_prompt_caching_sonnet.md | `DISABLE_PROMPT_CACHING_SONNET` — disable Sonnet caching |
| 118_disable_telemetry.md | `DISABLE_TELEMETRY` — opt out of telemetry |
| 119_disable_updates.md | `DISABLE_UPDATES` — block all updates |
| 120_disable_upgrade_command.md | `DISABLE_UPGRADE_COMMAND` — hide /upgrade |
| 121_auto_updates_channel.md | `autoUpdatesChannel` config key — release channel (latest/stable) |
| 122_minimum_version.md | `minimumVersion` config key — update floor version |
| 123_required_minimum_version.md | `requiredMinimumVersion` managed config key — startup floor |
| 124_required_maximum_version.md | `requiredMaximumVersion` managed config key — startup ceiling |
| 125_package_manager_auto_update.md | `CLAUDE_CODE_PACKAGE_MANAGER_AUTO_UPDATE` — auto-run brew/winget upgrade |
| 126_disable_nonessential_traffic.md | `CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC` — combined disable of 4 vars |
| 127_bg_classifier_model.md | `CLAUDE_CODE_BG_CLASSIFIER_MODEL` — model override for background-task classifier calls |
| 128_bg_tasks_report_running.md | `CLAUDE_CODE_BG_TASKS_REPORT_RUNNING` — keep session reported "running" while background tasks are outstanding |
| 129_disable_bg_exit_handoff.md | `CLAUDE_CODE_DISABLE_BG_EXIT_HANDOFF` — disable handing off in-flight background work across a process exit |
| 130_disable_bg_shell_pressure_reap.md | `CLAUDE_CODE_DISABLE_BG_SHELL_PRESSURE_REAP` — disable memory-pressure reaping of idle background shells |
| 131_print_bg_wait_ceiling_ms.md | `CLAUDE_CODE_PRINT_BG_WAIT_CEILING_MS` — ceiling on print-mode wait for outstanding background tasks |
| 132_claudecode.md | `CLAUDECODE` — broadest marker set in any Claude-Code-spawned subprocess |
| 133_child_session.md | `CLAUDE_CODE_CHILD_SESSION` — precise nested-claude-process marker |
| 134_entrypoint.md | `CLAUDE_CODE_ENTRYPOINT` — launching-wrapper classifier (vscode/remote/sdk/teams) |
| 135_force_session_persistence.md | `CLAUDE_CODE_FORCE_SESSION_PERSISTENCE` — override child-session exclusion false positives |
| 136_disable_background_tasks.md | `CLAUDE_CODE_DISABLE_BACKGROUND_TASKS` — disable all background task functionality |
| 137_job_dir.md | `CLAUDE_JOB_DIR` — internal supervisor/agent-view job directory marker |
| 138_disable_adopt.md | `CLAUDE_DISABLE_ADOPT` — stop in-flight work carrying over when backgrounding a session |
| 139_async_agent_stall_timeout_ms.md | `CLAUDE_ASYNC_AGENT_STALL_TIMEOUT_MS` — stall timeout for background subagents |
| 140_auto_background_tasks.md | `CLAUDE_AUTO_BACKGROUND_TASKS` — force-enable automatic backgrounding heuristic |

### Parameter Table

Precedence: CLI arg > env var > settings config.
`—` = that form does not exist for this parameter.

| # | Name | CLI Flag | Env Var | Config Key | Type | Binary Default | Since | Description |
|---|------|----------|---------|------------|------|----------------|-------|-------------|
| 1 | [prompt](052_prompt.md) | `<message>` (positional) | — | — | string | — | pre-v1.0 | Message sent to Claude |
| 2 | [print](051_print.md) | `-p` / `--print` | — | — | bool | off | pre-v1.0 | Print response and exit; skip TTY |
| 3 | [continue](017_continue.md) | `-c` / `--continue` | — | — | bool | off | pre-v1.0 | Continue most recent session |
| 4 | [model](042_model.md) | `--model <model>` | — | `model` | string | `claude-sonnet-4-6` | pre-v1.0 | Model alias or full ID |
| 5 | [verbose](071_verbose.md) | `--verbose` | — | — | bool | off | pre-v1.0 | Override verbose mode from config |
| 6 | [api_key](007_api_key.md) | — | `ANTHROPIC_API_KEY` | — | string | — | pre-v1.0 | Anthropic API key (`--api-key` removed from CLI) |
| 7 | [system_prompt](063_system_prompt.md) | `--system-prompt <prompt>` | — | — | string | — | pre-v1.0 | Replace default system prompt entirely |
| 8 | [append_system_prompt](008_append_system_prompt.md) | `--append-system-prompt <prompt>` | — | — | string | — | pre-v1.0 | Append text to default system prompt |
| 9 | [dangerously_skip_permissions](018_dangerously_skip_permissions.md) | `--dangerously-skip-permissions` | — | — | bool | off | pre-v1.0 | Bypass all permission checks |
| 10 | [allow_dangerously_skip_permissions](005_allow_dangerously_skip_permissions.md) | `--allow-dangerously-skip-permissions` | — | — | bool | off | pre-v1.0 | Enable skip-permissions as option (not default) |
| 11 | [permission_mode](046_permission_mode.md) | `--permission-mode <mode>` | — | `permissionMode` | enum | `default` | pre-v1.0 | `default` `acceptEdits` `bypassPermissions` `dontAsk` `plan` `auto` |
| 12 | [resume](055_resume.md) | `-r` / `--resume [id]` | — | — | string? | — | pre-v1.0 | Resume session by ID; interactive picker if no ID |
| 13 | [session_id](058_session_id.md) | `--session-id <uuid>` | — | — | uuid | auto | pre-v1.0 | Specify session UUID for this run |
| 14 | [fork_session](029_fork_session.md) | `--fork-session` | — | — | bool | off | pre-v1.0 | Create new session ID on resume |
| 15 | [no_session_persistence](043_no_session_persistence.md) | `--no-session-persistence` | — | — | bool | off | pre-v1.0 | Disable save-to-disk; cannot be resumed |
| 16 | [from_pr](030_from_pr.md) | `--from-pr [value]` | — | — | string? | — | pre-v1.0 | Resume session linked to PR by number or URL |
| 17 | [session_dir](057_session_dir.md) | — | `CLAUDE_CODE_SESSION_DIR` | — | path | auto | pre-v1.0 | Override session storage directory |
| 18 | [auto_continue](010_auto_continue.md) | — | `CLAUDE_CODE_AUTO_CONTINUE` | — | bool | false | pre-v1.0 | Auto-continue without prompts |
| 19 | [add_dir](002_add_dir.md) | `--add-dir <dirs...>` | — | — | path[] | — | pre-v1.0 | Grant tool access to additional directories |
| 20 | [allowed_tools](006_allowed_tools.md) | `--allowed-tools <tools...>` | — | `allowedTools` | string[] | all | pre-v1.0 | Allowlist of permitted tools |
| 21 | [disallowed_tools](022_disallowed_tools.md) | `--disallowed-tools <tools...>` | — | `disallowedTools` | string[] | none | pre-v1.0 | Denylist of forbidden tools |
| 22 | [tools](068_tools.md) | `--tools <tools...>` | — | — | string[] | `default` | pre-v1.0 | Override full tool set; `""` disables all |
| 23 | [auto_approve_tools](009_auto_approve_tools.md) | — | `CLAUDE_CODE_AUTO_APPROVE_TOOLS` | — | bool | false | pre-v1.0 | Auto-approve all tool calls without prompting |
| 24 | [action_mode](001_action_mode.md) | — | `CLAUDE_CODE_ACTION_MODE` | — | enum | `Ask` | pre-v1.0 | Tool execution action mode |
| 25 | [output_format](044_output_format.md) | `--output-format <fmt>` | — | — | enum | `text` | pre-v1.0 | `text` `json` `stream-json` |
| 26 | [input_format](034_input_format.md) | `--input-format <fmt>` | — | — | enum | `text` | pre-v1.0 | `text` `stream-json` |
| 27 | [include_partial_messages](033_include_partial_messages.md) | `--include-partial-messages` | — | — | bool | off | pre-v1.0 | Stream partial chunks (requires stream-json) |
| 28 | [replay_user_messages](054_replay_user_messages.md) | `--replay-user-messages` | — | — | bool | off | pre-v1.0 | Re-emit user messages on stdout |
| 29 | [json_schema](035_json_schema.md) | `--json-schema <schema>` | — | — | json | — | pre-v1.0 | JSON Schema for structured output validation |
| 30 | [max_output_tokens](038_max_output_tokens.md) | — | `CLAUDE_CODE_MAX_OUTPUT_TOKENS` | — | integer | 32 000 | pre-v1.0 | Max tokens per response |
| 31 | [effort](023_effort.md) | `--effort <level>` | — | `effortLevel` | enum | `medium` | pre-v1.0 | `low` `medium` `high` `max` |
| 32 | [fallback_model](026_fallback_model.md) | `--fallback-model <model>` | — | — | string | — | pre-v1.0 | Fallback model when primary is overloaded |
| 33 | [max_budget_usd](037_max_budget_usd.md) | `--max-budget-usd <amount>` | — | — | float | — | pre-v1.0 | Max API spend cap in USD (print mode only) |
| 34 | [temperature](065_temperature.md) | — | `CLAUDE_CODE_TEMPERATURE` | — | float | 1.0 | pre-v1.0 | Model temperature (0.0–1.0) |
| 35 | [top_p](070_top_p.md) | — | `CLAUDE_CODE_TOP_P` | — | float | none | pre-v1.0 | Top-p nucleus sampling (0.0–1.0) |
| 36 | [top_k](069_top_k.md) | — | `CLAUDE_CODE_TOP_K` | — | integer | none | pre-v1.0 | Top-k sampling cutoff |
| 37 | [bash_timeout](013_bash_timeout.md) | — | `CLAUDE_CODE_BASH_TIMEOUT` | — | integer ms | 120 000 | pre-v1.0 | Default bash command timeout (2 min) |
| 38 | [bash_max_timeout](012_bash_max_timeout.md) | — | `CLAUDE_CODE_BASH_MAX_TIMEOUT` | — | integer ms | 600 000 | pre-v1.0 | Max allowed bash command timeout (10 min) |
| 39 | [mcp_config](039_mcp_config.md) | `--mcp-config <configs...>` | — | — | json[] | — | pre-v1.0 | Load MCP servers from JSON files or strings |
| 40 | [strict_mcp_config](062_strict_mcp_config.md) | `--strict-mcp-config` | — | — | bool | off | pre-v1.0 | Ignore all non-`--mcp-config` MCP sources |
| 41 | [settings](060_settings.md) | `--settings <file-or-json>` | — | — | path/json | — | pre-v1.0 | Load additional settings from file or JSON string |
| 42 | [setting_sources](059_setting_sources.md) | `--setting-sources <sources>` | — | — | string | all | pre-v1.0 | Filter which sources load: `user` `project` `local` |
| 43 | [agent](003_agent.md) | `--agent <agent>` | — | — | string | — | pre-v1.0 | Override agent for this session |
| 44 | [agents](004_agents.md) | `--agents <json>` | — | — | json | — | pre-v1.0 | Define custom agents as JSON object |
| 45 | [plugin_dir](048_plugin_dir.md) | `--plugin-dir <paths...>` | — | — | path[] | — | pre-v1.0 | Load plugins from directories (session only) |
| 46 | [worktree](073_worktree.md) | `-w` / `--worktree [name]` | — | — | string? | — | pre-v1.0 | Create git worktree for this session |
| 47 | [tmux](067_tmux.md) | `--tmux` | — | — | bool | off | pre-v1.0 | Create tmux session for the worktree |
| 48 | [ide](032_ide.md) | `--ide` | — | — | bool | off | pre-v1.0 | Auto-connect to IDE on startup |
| 49 | [chrome](016_chrome.md) | `--chrome` / `--no-chrome` | — | — | bool | off | pre-v1.0 | Claude-in-Chrome integration |
| 50 | [debug](019_debug.md) | `-d` / `--debug [filter]` | — | — | string? | off | pre-v1.0 | Debug mode; optional category filter e.g. `"api,hooks"` |
| 51 | [debug_file](020_debug_file.md) | `--debug-file <path>` | — | — | path | — | pre-v1.0 | Write debug logs to file (implicitly enables debug) |
| 52 | [log_level](036_log_level.md) | — | `CLAUDE_CODE_LOG_LEVEL` | — | enum | `Info` | pre-v1.0 | `Error` `Warn` `Info` `Debug` `Trace` |
| 53 | [sandbox_mode](056_sandbox_mode.md) | — | `CLAUDE_CODE_SANDBOX_MODE` | — | bool | true | pre-v1.0 | Enable sandbox mode |
| 54 | [telemetry](064_telemetry.md) | — | `CLAUDE_CODE_TELEMETRY` | — | bool | true | pre-v1.0 | Send usage telemetry to Anthropic |
| 55 | [betas](014_betas.md) | `--betas <betas...>` | — | — | string[] | — | pre-v1.0 | Beta API headers (API key auth only) |
| 56 | [brief](015_brief.md) | `--brief` | — | — | bool | off | pre-v1.0 | Enable `SendUserMessage` tool for agents |
| 57 | [disable_slash_commands](021_disable_slash_commands.md) | `--disable-slash-commands` | — | — | bool | off | pre-v1.0 | Disable all slash command skills |
| 58 | [file](027_file.md) | `--file <specs...>` | — | — | string[] | — | pre-v1.0 | Download file resources at startup (`file_id:path`) |
| 59 | [mcp_debug](040_mcp_debug.md) | `--mcp-debug` ⚠️ | — | — | bool | off | pre-v1.0 | **DEPRECATED** — use `--debug` instead |
| 60 | [theme](066_theme.md) | — | — | `theme` | string | `"dark"` | pre-v1.0 | UI color theme |
| 61 | [auto_updates](011_auto_updates.md) | — | — | `autoUpdates` | bool | true | pre-v1.0 | Auto-update binary on startup |
| 62 | [preferred_version_spec](050_preferred_version_spec.md) | — | — | `preferredVersionSpec` | string/null | null | pre-v1.0 | Preferred version alias or semver |
| 63 | [preferred_version_resolved](049_preferred_version_resolved.md) | — | — | `preferredVersionResolved` | string/null | null | pre-v1.0 | Concrete semver resolved at last install |
| 64 | [env_overrides](025_env_overrides.md) | — | — | `env` | object | `{}` | pre-v1.0 | Persistent env vars injected at every startup |
| 65 | [enabled_plugins](024_enabled_plugins.md) | — | — | `enabledPlugins` | object | `{}` | pre-v1.0 | Active plugin registry |
| 66 | [hooks](031_hooks.md) | — | — | `hooks` | object | `{}` | pre-v1.0 | Hooks executed at `PreToolUse` / `PostToolUse` / `UserPromptSubmit` events |
| 67 | [mcp_servers](041_mcp_servers.md) | — | — | `mcpServers` | object | `{}` | pre-v1.0 | Inline MCP server definitions (alternative to `--mcp-config`) |
| 68 | [skip_dangerous_mode_permission_prompt](061_skip_dangerous_mode_permission_prompt.md) | — | — | `skipDangerousModePermissionPrompt` | bool | `false` | pre-v1.0 | Suppress interactive confirmation for dangerous mode |
| 69 | [voice_enabled](072_voice_enabled.md) | — | — | `voiceEnabled` | bool | `false` | pre-v1.0 | Enable voice input and audio output features |
| 70 | [permissions](047_permissions.md) | — | — | `permissions` | object | `{}` | pre-v1.0 | Per-project tool allow/deny/ask rules; auto-managed by Claude Code |
| 71 | [output_style](045_output_style.md) | — | — | `outputStyle` | string | `"default"` | pre-v1.0 | Terminal output visual rendering style |
| 72 | [file_checkpointing_enabled](028_file_checkpointing_enabled.md) | — | — | `fileCheckpointingEnabled` | bool | `false` | pre-v1.0 | Save file checkpoint before each edit |
| 73 | [remote_control_at_startup](053_remote_control_at_startup.md) | — | — | `remoteControlAtStartup` | bool | `false` | pre-v1.0 | Open remote-control channel on startup |
| 74 | [auto_compact_window](074_auto_compact_window.md) | — | `CLAUDE_CODE_AUTO_COMPACT_WINDOW` | — | integer (tokens) | `200 000` / `1 000 000` | v2.1.75 | Context window in tokens for auto-compaction threshold; capped at model limit |
| 75 | [autocompact_pct_override](075_autocompact_pct_override.md) | — | `CLAUDE_AUTOCOMPACT_PCT_OVERRIDE` | — | integer (1–100) | auto | v2.1.75 (no `_CODE_` infix) | Compaction trigger as percentage of `CLAUDE_CODE_AUTO_COMPACT_WINDOW` |
| 76 | [max_turns](076_max_turns.md) | `--max-turns <n>` | — | — | integer | — | pre-v1.0 | Max agentic turns per session; unset = unlimited |
| 77 | [safe_mode](077_safe_mode.md) | `--safe-mode` | `CLAUDE_CODE_SAFE_MODE` | — | bool | off | v2.1.169 | Disable bundled skills and experimental features |
| 78 | [disable_bundled_skills](078_disable_bundled_skills.md) | — | `CLAUDE_CODE_DISABLE_BUNDLED_SKILLS` | `disableBundledSkills` | bool | false | v2.1.169 | Disable all built-in slash command skills |
| 79 | [subagent_model](079_subagent_model.md) | — | `CLAUDE_CODE_SUBAGENT_MODEL` | — | string | — | v2.1.153 | Model override for Agent-tool subagent sessions |
| 80 | [experimental_agent_teams](080_experimental_agent_teams.md) | — | `CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS` | — | bool | false | v2.1.178 | Enable experimental multi-agent team coordination |
| 81 | [enable_auto_mode](081_enable_auto_mode.md) | — | `CLAUDE_CODE_ENABLE_AUTO_MODE` | — | bool | false | v2.1.158 | Enable auto-mode permission classifier |
| 82 | [tmpdir](082_tmpdir.md) | — | `CLAUDE_CODE_TMPDIR` | — | path | system temp | v2.1.161 | Override temporary directory |
| 83 | [stop_hook_block_cap](083_stop_hook_block_cap.md) | — | `CLAUDE_CODE_STOP_HOOK_BLOCK_CAP` | — | integer | — | v2.1.147 | Max consecutive hook blocks before bypass |
| 84 | [ps_execution_policy](084_ps_execution_policy.md) | — | `CLAUDE_CODE_POWERSHELL_RESPECT_EXECUTION_POLICY` | — | bool | false | v2.1.143 | Respect PowerShell execution policy (Windows) |
| 85 | [default_sonnet_model](085_default_sonnet_model.md) | — | `ANTHROPIC_DEFAULT_SONNET_MODEL` | — | string | latest Sonnet | v2.1.174 | Override sonnet alias model target |
| 86 | [client_presence_file](086_client_presence_file.md) | — | `CLAUDE_CLIENT_PRESENCE_FILE` | — | path | — | v2.1.181 | IDE client presence detection file path |
| 87 | [workspace_id](087_workspace_id.md) | — | `ANTHROPIC_WORKSPACE_ID` | — | string | — | v2.1.141 | Anthropic workspace ID for API routing |
| 88 | [plugin_prefer_https](088_plugin_prefer_https.md) | — | `CLAUDE_CODE_PLUGIN_PREFER_HTTPS` | — | bool | false | v2.1.141 | Prefer HTTPS transport for plugins |
| 89 | [mcp_tool_timeout](089_mcp_tool_timeout.md) | — | `MCP_TOOL_TIMEOUT` | — | integer ms | — | v2.1.142 | MCP tool invocation timeout |
| 90 | [api_force_idle_timeout](090_api_force_idle_timeout.md) | — | `API_FORCE_IDLE_TIMEOUT` | — | integer ms | — | v2.1.169 | Force API connection idle timeout |
| 91 | [language](091_language.md) | — | — | `language` | string | system locale | v2.1.176 | UI language |
| 92 | [worktree_bg_isolation](092_worktree_bg_isolation.md) | — | — | `worktree.bgIsolation` | bool | false | v2.1.143 | Worktree background isolation |
| 93 | [sandbox_allow_apple_events](093_sandbox_allow_apple_events.md) | — | — | `sandbox.allowAppleEvents` | bool | false | v2.1.181 | Allow Apple Events in sandbox (macOS) |
| 94 | [thinking_disabled_display](094_thinking_disabled_display.md) | — | — | `thinking.disabled.display` | string | — | v2.1.181 | Thinking indicator display when disabled |
| 95 | [wheel_scroll_accel](095_wheel_scroll_accel.md) | — | — | `wheelScrollAccelerationEnabled` | bool | false | v2.1.174 | Mouse wheel scroll acceleration |
| 96 | [bash_default_timeout_ms](096_bash_default_timeout_ms.md) | — | `BASH_DEFAULT_TIMEOUT_MS` | — | integer ms | `120000` | pre-v1.0 | Default Bash tool timeout (2 min) |
| 97 | [bash_max_output_length](097_bash_max_output_length.md) | — | `BASH_MAX_OUTPUT_LENGTH` | — | integer | — | pre-v1.0 | Max chars in Bash output before file save |
| 98 | [bash_max_timeout_ms](098_bash_max_timeout_ms.md) | — | `BASH_MAX_TIMEOUT_MS` | — | integer ms | `600000` | v0.2.108 | Max model-requested Bash timeout (10 min) |
| 99 | [disable_autoupdater](099_disable_autoupdater.md) | — | `DISABLE_AUTOUPDATER` | — | bool | off | pre-v1.0 | Disable background auto-updates |
| 100 | [disable_auto_compact](100_disable_auto_compact.md) | — | `DISABLE_AUTO_COMPACT` | — | bool | off | pre-v1.0 | Disable auto-compaction only |
| 101 | [disable_compact](101_disable_compact.md) | — | `DISABLE_COMPACT` | — | bool | off | pre-v1.0 | Disable all compaction (auto + manual) |
| 102 | [disable_cost_warnings](102_disable_cost_warnings.md) | — | `DISABLE_COST_WARNINGS` | — | bool | off | pre-v1.0 | Disable cost warning messages |
| 103 | [disable_doctor_command](103_disable_doctor_command.md) | — | `DISABLE_DOCTOR_COMMAND` | — | bool | off | pre-v1.0 | Hide /doctor slash command |
| 104 | [disable_error_reporting](104_disable_error_reporting.md) | — | `DISABLE_ERROR_REPORTING` | — | bool | off | pre-v1.0 | Opt out of Sentry error reporting |
| 105 | [disable_extra_usage_command](105_disable_extra_usage_command.md) | — | `DISABLE_EXTRA_USAGE_COMMAND` | — | bool | off | pre-v1.0 | Hide /usage-credits slash command |
| 106 | [disable_feedback_command](106_disable_feedback_command.md) | — | `DISABLE_FEEDBACK_COMMAND` | — | bool | off | pre-v1.0 | Disable /feedback slash command |
| 107 | [disable_growthbook](107_disable_growthbook.md) | — | `DISABLE_GROWTHBOOK` | — | bool | off | pre-v1.0 | Disable GrowthBook feature flags |
| 108 | [disable_installation_checks](108_disable_installation_checks.md) | — | `DISABLE_INSTALLATION_CHECKS` | — | bool | off | pre-v1.0 | Disable installation warnings |
| 109 | [disable_install_github_app_command](109_disable_install_github_app_command.md) | — | `DISABLE_INSTALL_GITHUB_APP_COMMAND` | — | bool | off | pre-v1.0 | Hide /install-github-app command |
| 110 | [disable_interleaved_thinking](110_disable_interleaved_thinking.md) | — | `DISABLE_INTERLEAVED_THINKING` | — | bool | off | v1.0.1 | Disable interleaved thinking beta |
| 111 | [disable_login_command](111_disable_login_command.md) | — | `DISABLE_LOGIN_COMMAND` | — | bool | off | pre-v1.0 | Hide /login slash command |
| 112 | [disable_logout_command](112_disable_logout_command.md) | — | `DISABLE_LOGOUT_COMMAND` | — | bool | off | pre-v1.0 | Hide /logout slash command |
| 113 | [disable_prompt_caching](113_disable_prompt_caching.md) | — | `DISABLE_PROMPT_CACHING` | — | bool | off | pre-v1.0 | Disable prompt caching for all models |
| 114 | [disable_prompt_caching_fable](114_disable_prompt_caching_fable.md) | — | `DISABLE_PROMPT_CACHING_FABLE` | — | bool | off | v2.1.170 | Disable prompt caching for Fable |
| 115 | [disable_prompt_caching_haiku](115_disable_prompt_caching_haiku.md) | — | `DISABLE_PROMPT_CACHING_HAIKU` | — | bool | off | pre-v1.0 | Disable prompt caching for Haiku |
| 116 | [disable_prompt_caching_opus](116_disable_prompt_caching_opus.md) | — | `DISABLE_PROMPT_CACHING_OPUS` | — | bool | off | pre-v1.0 | Disable prompt caching for Opus |
| 117 | [disable_prompt_caching_sonnet](117_disable_prompt_caching_sonnet.md) | — | `DISABLE_PROMPT_CACHING_SONNET` | — | bool | off | pre-v1.0 | Disable prompt caching for Sonnet |
| 118 | [disable_telemetry](118_disable_telemetry.md) | — | `DISABLE_TELEMETRY` | — | bool | off | pre-v1.0 | Opt out of telemetry |
| 119 | [disable_updates](119_disable_updates.md) | — | `DISABLE_UPDATES` | — | bool | off | pre-v1.0 | Block all updates (auto + manual) |
| 120 | [disable_upgrade_command](120_disable_upgrade_command.md) | — | `DISABLE_UPGRADE_COMMAND` | — | bool | off | pre-v1.0 | Hide /upgrade slash command |
| 121 | [auto_updates_channel](121_auto_updates_channel.md) | — | — | `autoUpdatesChannel` | string | `"latest"` | pre-v1.0 | Release channel for auto-updates: latest or stable |
| 122 | [minimum_version](122_minimum_version.md) | — | — | `minimumVersion` | string (semver) | — | pre-v1.0 | Update floor; blocks auto-update/claude update below this version |
| 123 | [required_minimum_version](123_required_minimum_version.md) | — | — | `requiredMinimumVersion` | string (semver) | — | v2.1.163 | Managed-only startup floor; exits at launch if older |
| 124 | [required_maximum_version](124_required_maximum_version.md) | — | — | `requiredMaximumVersion` | string (semver) | — | v2.1.163 | Managed-only startup ceiling; exits at launch if newer |
| 125 | [package_manager_auto_update](125_package_manager_auto_update.md) | — | `CLAUDE_CODE_PACKAGE_MANAGER_AUTO_UPDATE` | — | bool | off | v2.1.129 | Auto-run brew/winget upgrade in background |
| 126 | [disable_nonessential_traffic](126_disable_nonessential_traffic.md) | — | `CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC` | — | bool | off | pre-v1.0 | Equivalent to 4 DISABLE_* vars combined |
| 127 | [bg_classifier_model](127_bg_classifier_model.md) | — | `CLAUDE_CODE_BG_CLASSIFIER_MODEL` | — | string | binary default | ≤v2.1.197 (undocumented) | Model override for background-task classifier calls |
| 128 | [bg_tasks_report_running](128_bg_tasks_report_running.md) | — | `CLAUDE_CODE_BG_TASKS_REPORT_RUNNING` | — | bool | false | ≤v2.1.197 (undocumented) | Keep session reported "running" while background tasks are outstanding |
| 129 | [disable_bg_exit_handoff](129_disable_bg_exit_handoff.md) | — | `CLAUDE_CODE_DISABLE_BG_EXIT_HANDOFF` | — | bool | false | ≤v2.1.197 (undocumented) | Disable handoff of in-flight background shells/workflows across a process exit |
| 130 | [disable_bg_shell_pressure_reap](130_disable_bg_shell_pressure_reap.md) | — | `CLAUDE_CODE_DISABLE_BG_SHELL_PRESSURE_REAP` | — | bool | false | v2.1.193 | Disable memory-pressure reaping of idle background shells |
| 131 | [print_bg_wait_ceiling_ms](131_print_bg_wait_ceiling_ms.md) | — | `CLAUDE_CODE_PRINT_BG_WAIT_CEILING_MS` | — | integer ms | `600000` | ≤v2.1.197 (undocumented) | Ceiling on print-mode wait for outstanding background tasks |
| 132 | [claudecode](132_claudecode.md) | — | `CLAUDECODE` | — | bool | false | pre-v1.0 (documented) | Broadest marker: set in any subprocess Claude Code spawns |
| 133 | [child_session](133_child_session.md) | — | `CLAUDE_CODE_CHILD_SESSION` | — | bool | false | v2.1.172+ | Precise marker for a nested `claude` process Claude Code itself launched |
| 134 | [entrypoint](134_entrypoint.md) | — | `CLAUDE_CODE_ENTRYPOINT` | — | enum (string) | unset | ≤v2.1.197 (undocumented) | Classifies the launching wrapper (vscode/remote/sdk/teams variants) |
| 135 | [force_session_persistence](135_force_session_persistence.md) | — | `CLAUDE_CODE_FORCE_SESSION_PERSISTENCE` | — | bool | false | ≤v2.1.197 (documented) | Override `CLAUDE_CODE_CHILD_SESSION` exclusion for false positives (e.g. `screen`) |
| 136 | [disable_background_tasks](136_disable_background_tasks.md) | — | `CLAUDE_CODE_DISABLE_BACKGROUND_TASKS` | — | bool | false | ≤v2.1.197 (documented) | Disable all background task functionality (`run_in_background`, auto-background, Ctrl+B) |
| 137 | [job_dir](137_job_dir.md) | — | `CLAUDE_JOB_DIR` | — | string (dir path) | unset | ≤v2.1.197 (undocumented; no `_CODE_` infix) | Internal marker for supervisor/agent-view architecture; precondition for exit handoff |
| 138 | [disable_adopt](138_disable_adopt.md) | — | `CLAUDE_DISABLE_ADOPT` | — | bool | false | v2.1.195+ (no `_CODE_` infix) | Stop in-flight work from carrying over when a session is backgrounded (`←`/`/background`) |
| 139 | [async_agent_stall_timeout_ms](139_async_agent_stall_timeout_ms.md) | — | `CLAUDE_ASYNC_AGENT_STALL_TIMEOUT_MS` | — | integer ms | `600000` | ≤v2.1.197 (documented; no `_CODE_` infix) | Stall/no-progress timeout for background subagents, resets on progress |
| 140 | [auto_background_tasks](140_auto_background_tasks.md) | — | `CLAUDE_AUTO_BACKGROUND_TASKS` | — | bool | false | ≤v2.1.197 (documented; no `_CODE_` infix) | Force-enable automatic backgrounding of long-running agent tasks |

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [`../behavior/readme.md`](../behavior/readme.md) | Claude Code behavior collection master file |
| doc | [`../behavior/readme.md`](../behavior/readme.md) | Observed flag behavior in practice |
| doc | [`../settings/readme.md`](../settings/readme.md) | settings.json write semantics and structure |
| doc | [`../../../../module/claude_runner_core/docs/claude_params/readme.md`](../../../../module/claude_runner_core/docs/claude_params/readme.md) | Builder-API perspective with Rust `with_*()` methods |
