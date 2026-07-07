# claude_params Doc Entity

### Scope

- **Purpose**: Document the builder-API perspective of all ClaudeCommand parameters.
- **Responsibility**: Index of per-parameter reference docs covering all ClaudeCommand with_*() methods and the claude binary flags they wrap.
- **In Scope**: All 72 ClaudeCommand parameters: with_*() methods, types, builder defaults, and underlying binary flag mappings.
- **Out of Scope**: Binary-perspective reference (→ `contract/claude_code/docs/param/`), execution mode design (→ `feature/`).

Builder-API reference for `ClaudeCommand` — documents Rust `with_*()` methods,
builder-specific defaults, and the underlying `claude` binary parameters they wrap.

> **Binary-perspective reference** (actual `claude` flags, env vars, config keys with
> binary defaults): [`contract/claude_code/docs/param/readme.md`](../../../../contract/claude_code/docs/param/readme.md).
> This file is the **builder-API perspective** — defaults here are intentionally tuned
> for automation and may differ from the binary defaults shown there.

### Parameter Summary Table

Quick-reference for all 72 parameters (#1–#72). Type: **CLI** = flag only, **Env** = env var only, **Config** = settings.json key only, **Both** = CLI flag + env var. Builder: typed `with_*()` method for all runtime parameters; only deprecated `mcp_debug` uses `with_arg()` fallback. Parameters #63–#72 are settings config keys with no builder method.

### Core Execution

| # | Parameter | Type | CLI Flag | Env Var | Default | Builder | Description |
|---|-----------|------|----------|---------|---------|---------|-------------|
| 1 | [prompt](043_prompt.md) | CLI | `<prompt>` (positional) | — | — | `with_message()` | Message sent to Claude |
| 2 | [print](042_print.md) | CLI | `-p` / `--print` | — | off | `with_print()` | Print response and exit; skip TTY |
| 3 | [dry_run](022_dry_run.md) | Builder | — | — | false | `with_dry_run()` | Inspect command without spawning process |
| 4 | [continue_conversation](016_continue_conversation.md) | CLI | `-c` / `--continue` | — | off | `with_continue_conversation()` | Continue most recent conversation |
| 5 | [model](037_model.md) | CLI | `--model <model>` | — | `claude-sonnet-5` | `with_model()` | Model alias or full model ID |
| 6 | [verbose](059_verbose.md) | CLI | `--verbose` | — | off | `with_verbose()` | Override verbose mode from config |

### Authentication

| # | Parameter | Type | CLI Flag | Env Var | Default | Builder | Description |
|---|-----------|------|----------|---------|---------|---------|-------------|
| 7 | [api_key](007_api_key.md) | Both | `--api-key <key>` | `ANTHROPIC_API_KEY` | — | `with_api_key()` | Anthropic API key |

### System Prompt

| # | Parameter | Type | CLI Flag | Env Var | Default | Builder | Description |
|---|-----------|------|----------|---------|---------|---------|-------------|
| 8 | [system_prompt](052_system_prompt.md) | CLI | `--system-prompt <prompt>` | — | — | `with_system_prompt()` | Replace default system prompt |
| 9 | [append_system_prompt](008_append_system_prompt.md) | CLI | `--append-system-prompt <prompt>` | — | — | `with_append_system_prompt()` | Append to default system prompt |

### Permissions

| # | Parameter | Type | CLI Flag | Env Var | Default | Builder | Description |
|---|-----------|------|----------|---------|---------|---------|-------------|
| 10 | [dangerously_skip_permissions](017_dangerously_skip_permissions.md) | CLI | `--dangerously-skip-permissions` | — | off | `with_skip_permissions()` | Bypass all permission checks |
| 11 | [allow_dangerously_skip_permissions](005_allow_dangerously_skip_permissions.md) | CLI | `--allow-dangerously-skip-permissions` | — | off | `with_allow_dangerously_skip_permissions()` | Enable skip-permissions as option |
| 12 | [permission_mode](040_permission_mode.md) | CLI | `--permission-mode <mode>` | — | `default` | `with_permission_mode()` | Fine-grained permission mode |

### Session Management

| # | Parameter | Type | CLI Flag | Env Var | Default | Builder | Description |
|---|-----------|------|----------|---------|---------|---------|-------------|
| 13 | [resume](045_resume.md) | CLI | `-r` / `--resume [id]` | — | — | `with_resume()` | Resume conversation by session ID |
| 14 | [session_id](048_session_id.md) | CLI | `--session-id <uuid>` | — | auto | `with_session_id()` | Specify session UUID |
| 15 | [fork_session](026_fork_session.md) | CLI | `--fork-session` | — | off | `with_fork_session()` | Create new session ID on resume |
| 16 | [no_session_persistence](038_no_session_persistence.md) | CLI | `--no-session-persistence` | — | off | `with_no_session_persistence()` | Disable save-to-disk |
| 17 | [from_pr](027_from_pr.md) | CLI | `--from-pr [value]` | — | — | `with_from_pr()` | Resume session linked to PR |

### Tools & Directories

| # | Parameter | Type | CLI Flag | Env Var | Default | Builder | Description |
|---|-----------|------|----------|---------|---------|---------|-------------|
| 18 | [add_dir](002_add_dir.md) | CLI | `--add-dir <dirs...>` | — | — | `with_add_dir()` | Grant tool access to directories |
| 19 | [allowed_tools](006_allowed_tools.md) | CLI | `--allowed-tools <tools...>` | — | all | `with_allowed_tools()` | Allowlist of permitted tools |
| 20 | [disallowed_tools](021_disallowed_tools.md) | CLI | `--disallowed-tools <tools...>` | — | none | `with_disallowed_tools()` | Denylist of forbidden tools |
| 21 | [tools](056_tools.md) | CLI | `--tools <tools...>` | — | `default` | `with_tools()` | Override full available tool set |

### Input / Output

| # | Parameter | Type | CLI Flag | Env Var | Default | Builder | Description |
|---|-----------|------|----------|---------|---------|---------|-------------|
| 22 | [output_format](039_output_format.md) | CLI | `--output-format <fmt>` | — | `text` | `with_output_format()` | Response format (text/json/stream-json) |
| 23 | [input_format](030_input_format.md) | CLI | `--input-format <fmt>` | — | `text` | `with_input_format()` | Input format (text/stream-json) |
| 24 | [include_partial_messages](029_include_partial_messages.md) | CLI | `--include-partial-messages` | — | off | `with_include_partial_messages()` | Stream partial chunks |
| 25 | [replay_user_messages](044_replay_user_messages.md) | CLI | `--replay-user-messages` | — | off | `with_replay_user_messages()` | Re-emit user messages on stdout |
| 26 | [json_schema](031_json_schema.md) | CLI | `--json-schema <schema>` | — | — | `with_json_schema()` | Structured output JSON Schema |

### Model & Budget

| # | Parameter | Type | CLI Flag | Env Var | Default | Builder | Description |
|---|-----------|------|----------|---------|---------|---------|-------------|
| 27 | [effort](023_effort.md) | CLI | `--effort <level>` | — | `medium` | `with_effort()` | Effort level (low/medium/high/max) |
| 28 | [fallback_model](024_fallback_model.md) | CLI | `--fallback-model <model>` | — | — | `with_fallback_model()` | Fallback when primary is overloaded |
| 29 | [max_budget_usd](033_max_budget_usd.md) | CLI | `--max-budget-usd <amount>` | — | — | `with_max_budget_usd()` | Maximum API spend cap in USD |

### MCP & Extensions

| # | Parameter | Type | CLI Flag | Env Var | Default | Builder | Description |
|---|-----------|------|----------|---------|---------|---------|-------------|
| 30 | [mcp_config](035_mcp_config.md) | CLI | `--mcp-config <configs...>` | — | — | `with_mcp_config()` | Load MCP servers from JSON |
| 31 | [strict_mcp_config](051_strict_mcp_config.md) | CLI | `--strict-mcp-config` | — | off | `with_strict_mcp_config()` | Ignore all non-`--mcp-config` MCP |
| 32 | [settings](050_settings.md) | CLI | `--settings <file-or-json>` | — | — | `with_settings()` | Load settings file or JSON |
| 33 | [setting_sources](049_setting_sources.md) | CLI | `--setting-sources <sources>` | — | all | `with_setting_sources()` | Filter setting sources |
| 34 | [agent](003_agent.md) | CLI | `--agent <agent>` | — | — | `with_agent()` | Override agent for session |
| 35 | [agents](004_agents.md) | CLI | `--agents <json>` | — | — | `with_agents()` | Define custom agents as JSON |
| 36 | [plugin_dir](041_plugin_dir.md) | CLI | `--plugin-dir <paths...>` | — | — | `with_plugin_dir()` | Load plugins from directories |

### Terminal & IDE

| # | Parameter | Type | CLI Flag | Env Var | Default | Builder | Description |
|---|-----------|------|----------|---------|---------|---------|-------------|
| 37 | [worktree](060_worktree.md) | CLI | `-w` / `--worktree [name]` | — | — | `with_worktree()` | Create git worktree for session |
| 38 | [tmux](055_tmux.md) | CLI | `--tmux` | — | off | `with_tmux()` | Create tmux session for worktree |
| 39 | [ide](028_ide.md) | CLI | `--ide` | — | off | `with_ide()` | Auto-connect to IDE on startup |
| 40 | [chrome](015_chrome.md) | CLI | `--chrome` / `--no-chrome` | — | **on** | `with_chrome()` | Toggle Claude-in-Chrome integration |

### Debug

| # | Parameter | Type | CLI Flag | Env Var | Default | Builder | Description |
|---|-----------|------|----------|---------|---------|---------|-------------|
| 41 | [debug](018_debug.md) | CLI | `-d` / `--debug [filter]` | — | off | `with_debug()` | Debug mode with optional category filter |
| 42 | [debug_file](019_debug_file.md) | CLI | `--debug-file <path>` | — | — | `with_debug_file()` | Write debug logs to a file |

### Advanced CLI

| # | Parameter | Type | CLI Flag | Env Var | Default | Builder | Description |
|---|-----------|------|----------|---------|---------|---------|-------------|
| 43 | [betas](013_betas.md) | CLI | `--betas <betas...>` | — | — | `with_betas()` | Beta API headers (API key users only) |
| 44 | [brief](014_brief.md) | CLI | `--brief` | — | off | `with_brief()` | Enable SendUserMessage for agents |
| 45 | [disable_slash_commands](020_disable_slash_commands.md) | CLI | `--disable-slash-commands` | — | off | `with_disable_slash_commands()` | Disable all slash command skills |
| 46 | [file](025_file.md) | CLI | `--file <specs...>` | — | — | `with_file()` | Download file resources at startup |
| 47 | [mcp_debug](036_mcp_debug.md) | CLI | `--mcp-debug` | — | off | `with_arg()` | **DEPRECATED** — use `--debug` instead |

### Environment Variables (Builder API)

These parameters are only settable via environment variables. All have dedicated typed builder methods.

| # | Parameter | Type | CLI Flag | Env Var | Builder Default | Builder | Description |
|---|-----------|------|----------|---------|-----------------|---------|-------------|
| 48 | [max_output_tokens](034_max_output_tokens.md) | Env | — | `CLAUDE_CODE_MAX_OUTPUT_TOKENS` | `200000` | `with_max_output_tokens()` | Max tokens per response |
| 49 | [bash_timeout](012_bash_timeout.md) | Env | — | `CLAUDE_CODE_BASH_TIMEOUT` | `3600000` | `with_bash_timeout_ms()` | Default bash timeout (ms) |
| 50 | [bash_max_timeout](011_bash_max_timeout.md) | Env | — | `CLAUDE_CODE_BASH_MAX_TIMEOUT` | `7200000` | `with_bash_max_timeout_ms()` | Max bash timeout (ms) |
| 51 | [auto_continue](010_auto_continue.md) | Env | — | `CLAUDE_CODE_AUTO_CONTINUE` | `true` | `with_auto_continue()` | Auto-continue without prompts |
| 52 | [telemetry](053_telemetry.md) | Env | — | `CLAUDE_CODE_TELEMETRY` | `false` | `with_telemetry()` | Send usage telemetry |
| 53 | [auto_approve_tools](009_auto_approve_tools.md) | Env | — | `CLAUDE_CODE_AUTO_APPROVE_TOOLS` | `false` | `with_auto_approve_tools()` | Auto-approve all tool calls |
| 54 | [action_mode](001_action_mode.md) | Env | — | `CLAUDE_CODE_ACTION_MODE` | `Ask` | `with_action_mode()` | Tool execution action mode |
| 55 | [log_level](032_log_level.md) | Env | — | `CLAUDE_CODE_LOG_LEVEL` | `Info` | `with_log_level()` | Log verbosity level |
| 56 | [temperature](054_temperature.md) | Env | — | `CLAUDE_CODE_TEMPERATURE` | `1.0` | `with_temperature()` | Model temperature (0.0–1.0) |
| 57 | [sandbox_mode](046_sandbox_mode.md) | Env | — | `CLAUDE_CODE_SANDBOX_MODE` | `true` | `with_sandbox_mode()` | Enable sandbox mode |
| 58 | [session_dir](047_session_dir.md) | Env | — | `CLAUDE_CODE_SESSION_DIR` | auto | `with_session_dir()` | Override session directory |
| 59 | [top_p](058_top_p.md) | Env | — | `CLAUDE_CODE_TOP_P` | none | `with_top_p()` | Top-p nucleus sampling (0.0–1.0) |
| 60 | [top_k](057_top_k.md) | Env | — | `CLAUDE_CODE_TOP_K` | none | `with_top_k()` | Top-k sampling cutoff |
| 61 | [compact_window](071_compact_window.md) | Env | — | `CLAUDE_CODE_AUTO_COMPACT_WINDOW` | `300000` | `with_compact_window()` | Auto-compaction context window (tokens) |
| 62 | [print_bg_wait_ceiling_ms](072_print_bg_wait_ceiling_ms.md) | Env | — | `CLAUDE_CODE_PRINT_BG_WAIT_CEILING_MS` | `0` | `with_print_bg_wait_ceiling_ms()` | Print-mode background wait ceiling (ms) |

### Settings Config (`~/.claude/settings.json`)

These parameters are read from the settings file on startup. No builder method — managed by `claude_version`. Full write semantics in [settings/readme.md](../../../../contract/claude_code/docs/settings/readme.md).

| # | Key | Type | Values | Default | Description |
|---|-----|------|--------|---------|-------------|
| 63 | `theme` | Config | `str` | `"dark"` | UI color theme |
| 64 | `autoUpdates` | Config | `bool` | `true` | Auto-update the binary on startup |
| 65 | `preferredVersionSpec` | Config | `str\|null` | `null` | Preferred version alias or semver (e.g. `"stable"`, `"2.1.78"`) |
| 66 | `preferredVersionResolved` | Config | `str\|null` | `null` | Concrete semver resolved at last install; `null` for `latest` |
| 67 | `env` | Config | `object` | `{}` | Persistent env var overrides injected at startup (e.g. `DISABLE_AUTOUPDATER`) |
| 68 | `enabledPlugins` | Config | `object` | `{}` | Active plugin registry |
| 69 | `hooks` | Config | `object` | `{}` | Hooks executed at `PreToolUse` / `PostToolUse` / `UserPromptSubmit` events |
| 70 | `mcpServers` | Config | `object` | `{}` | Inline MCP server definitions (alternative to `--mcp-config`) |
| 71 | `skipDangerousModePermissionPrompt` | Config | `bool` | `false` | Suppress interactive confirmation for dangerous mode |
| 72 | `voiceEnabled` | Config | `bool` | `false` | Enable voice input and audio output |

### Parameters `clr` Sets By Default

`ClaudeCommand::new()` bakes in the following defaults unconditionally — each
field is initialized to `Some(...)` (never `None`) the moment a command is
built, independent of any CLI flag, env var, or `with_*()` override.
`build_command()` emits an env var (or CLI flag, for `chrome`) whenever the
field is `Some`, so every row below is present on the spawned `claude`
subprocess by default. Every other parameter in this doc starts
`None`/off/unset and inherits the `claude` binary's own standard behavior
unless the caller explicitly configures it.

| # | Parameter | Env Var / Flag | Builder Default | `claude` Binary Default | Builder Method |
|---|-----------|-----------------|------------------|--------------------------|-----------------|
| 48 | [max_output_tokens](034_max_output_tokens.md) | `CLAUDE_CODE_MAX_OUTPUT_TOKENS` | `200000` | `32000` | `with_max_output_tokens()` |
| 49 | [bash_timeout](012_bash_timeout.md) | `CLAUDE_CODE_BASH_TIMEOUT` | `3600000` (1 hr) | `120000` (2 min) | `with_bash_timeout_ms()` |
| 50 | [bash_max_timeout](011_bash_max_timeout.md) | `CLAUDE_CODE_BASH_MAX_TIMEOUT` | `7200000` (2 hr) | `600000` (10 min) | `with_bash_max_timeout_ms()` |
| 51 | [auto_continue](010_auto_continue.md) | `CLAUDE_CODE_AUTO_CONTINUE` | `true` | `false` | `with_auto_continue()` |
| 52 | [telemetry](053_telemetry.md) | `CLAUDE_CODE_TELEMETRY` | `false` | `true` | `with_telemetry()` |
| 40 | [chrome](015_chrome.md) | `--chrome` / `--no-chrome` | on | off | `with_chrome()` |
| 61 | [compact_window](071_compact_window.md) | `CLAUDE_CODE_AUTO_COMPACT_WINDOW` | `300000` | unset — model native (`200000` standard / `1000000` extended) | `with_compact_window()` |
| 62 | [print_bg_wait_ceiling_ms](072_print_bg_wait_ceiling_ms.md) | `CLAUDE_CODE_PRINT_BG_WAIT_CEILING_MS` | `0` (exit immediately) | `600000` (10 min) | `with_print_bg_wait_ceiling_ms()` |

Two of these (`compact_window`, `chrome`) expose an explicit disable path
(`with_compact_window(None)`, `with_chrome(None)`) that suppresses the env
var/flag entirely, deferring to the binary's own default. The rest
(`max_output_tokens`, `bash_timeout`, `bash_max_timeout`, `auto_continue`,
`telemetry`, `print_bg_wait_ceiling_ms`) can only be overridden to a
different value — there is no way to fully unset them via the builder API.
Unlike the other five, `print_bg_wait_ceiling_ms`'s default (`0`) is not a
more generous value than the binary's own — it's the opposite: `clr` hands
off all background-task wait behavior to its own wrapper-level
`gate_poll_secs`/`gate_max_attempts` polling instead of the binary's internal
ceiling, so `claude` itself should never wait.

### Cross-Layer Default Consistency Analysis

Comparing the table above against `claude_runner`'s own wrapper-level timeout
and concurrency-admission mechanisms (`module/claude_runner/src/cli/execution.rs`,
`module/claude_runner/src/cli/gate.rs`,
`module/claude_runner/docs/invariant/012_gate_slot_atomicity.md`) surfaces two
open issues not visible from either layer in isolation.

**Finding 1 — `clr`'s own print-mode watchdog is lower than the ceiling it
grants the child.** `execution.rs:578` sets `DEFAULT_PRINT_TIMEOUT_SECS = 3600`
(1 hr) as the default `--timeout` for print-mode `clr` invocations absent an
explicit override — a hard kill of the whole `clr` process (exit code 2). The
same invocation sets `bash_max_timeout` (row 50) to `7200000` ms (2 hr) on the
`claude` subprocess it spawns. A Bash tool call inside `claude` that
legitimately uses the full 2-hour ceiling `clr` granted it will be killed by
`clr`'s own 1-hour outer watchdog an hour early.

**Finding 2 — the `print_bg_wait_ceiling_ms=0` rationale above rests on a
mechanism that does not do what the rationale claims.** Per
`module/claude_runner/docs/invariant/012_gate_slot_atomicity.md`, the
`gate_poll_secs`/`gate_max_attempts` mechanism governs only `--max-sessions`
concurrency admission control — how many concurrent `clr`/`claude` processes
may run at once, host-wide. It has no relationship to waiting for a specific
session's own backgrounded work (Bash `run_in_background`, subagents,
workflows) to finish before that session's `clr` process exits; no
wrapper-level mechanism in `module/claude_runner/src` waits for a session's
own background tasks to complete. Setting `CLAUDE_CODE_PRINT_BG_WAIT_CEILING_MS=0`
may cause `clr` print-mode invocations to exit while the `claude` subprocess
still has genuinely outstanding background work — the failure this default
was intended to avoid.

| # | Parameter | Current Default | Issue | Recommended |
|---|-----------|-----------------|-------|-------------|
| — | `clr`'s own `--timeout` (print mode, `execution.rs:578`) | `3600` s (1 hr) | Lower than `bash_max_timeout` (2 hr) granted to the child | Raise to ≥ 2 hr + margin, or lower `bash_max_timeout_ms` to ≤ 1 hr, so the outer watchdog never fires before the inner ceiling it promises |
| 62 | `print_bg_wait_ceiling_ms` | `0` (exit immediately) | Rationale cites the `--max-sessions` gate/slot mechanism, which does not wait for a session's own background tasks | Implement a genuine wrapper-level background-task-completion wait, or set a bounded non-zero ceiling (e.g. matching the binary's own `600000` ms) as a safety net, or document this as a deliberately accepted risk rather than a mitigated one |

**Status:** analysis only — not yet filed as a bug or applied as a fix.

### Notes

- **Builder defaults vs claude defaults**: see [§ Parameters `clr` Sets By Default](#parameters-clr-sets-by-default) above for the complete table.
- **`--api-key` removed from CLI**: `api_key` (#7) is listed as `Both` (CLI + env) in this doc, but `--api-key` is no longer present in `claude --help` as of current builds — env var `ANTHROPIC_API_KEY` is the only runtime form. The binary-perspective reference in `contract/claude_code/docs/param/007_api_key.md` reflects this correctly; this doc retains the builder method which still passes the value via env var internally.
- **Deprecated**: `mcp_debug` (#47) documents `--mcp-debug` which is deprecated in favor of `--debug` (#41).
- **Builder-only**: `dry_run` (#3) is not a `claude` binary parameter — it controls whether `ClaudeCommand` spawns a process or returns `describe_compact()` as stdout.
- **Config vs runtime**: Settings config parameters (#63–#72) are loaded once at startup from `~/.claude/settings.json`; runtime parameters (#1–#62) are passed per-invocation via CLI flags or env vars.
- **Precedence**: CLI arg > env var > settings config.
- **Dual-form params**: `model` (#5) and `effort` (#27) each have both a CLI flag form (with builder method) and a config key form (`model`, `effortLevel`). Similarly, `allowed_tools` (#19), `disallowed_tools` (#20), and `permission_mode` (#12) accept a `allowedTools`/`disallowedTools`/`permissionMode` config key in project-level `.claude/settings.json`. These dual forms are not listed separately in this table; see `contract/claude_code/docs/param/readme.md` for the complete picture.
- **Source**: CLI flags from `claude --help`; env vars from `src/command/mod.rs` `build_command()`; settings keys from `contract/claude_code/docs/settings/readme.md`.
