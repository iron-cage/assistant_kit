# Parameter Groups

### All Groups (4 total)

| # | Group | Parameters | Purpose |
|---|-------|------------|---------|
| 1 | Claude-Native Flags | 7 | Flags passed through to the claude subprocess |
| 2 | Runner Control | 12 | Flags consumed by the runner itself |
| 3 | System Prompt | 2 | Flags that inject or extend the system prompt sent to claude |
| 4 | Isolated Subcommand | 2 | Flags exclusive to the `isolated` subcommand |

**Total:** 4 groups

---

### Group :: 1. Claude-Native Flags

Flags forwarded directly to the `claude` subprocess. The runner does not
interpret these — it passes them through via `ClaudeCommand` builder calls.

**Note:** Session continuation (`-c`) is applied automatically and is not
exposed as a user flag. Use `--new-session` (Runner Control) to disable it.

**Note:** `--dangerously-skip-permissions` is injected automatically by `clr` (default-on).
Use `--no-skip-permissions` (Runner Control) to disable the automatic bypass.

**Note:** `--effort max` is injected automatically by `clr` (default-on).
Use `--effort <level>` to override or `--no-effort-max` (Runner Control) to suppress entirely.

**Coherence test:** "Is this flag consumed by the claude subprocess?" — YES for all 7.

**Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| [`-p`/`--print`](param/02_print.md) | bool | Print mode (default when message given) |
| [`--model`](param/03_model.md) | [`ModelName`](type.md#type--4-modelname) | Model selection |
| [`--verbose`](param/04_verbose.md) | bool | Claude verbose output |
| [`--effort`](param/17_effort.md) | [`EffortLevel`](type.md#type--7-effortlevel) | Reasoning effort level (default: max) |
| [`--no-persist`](param/22_no_persist.md) | bool | Disable session persistence (`--no-session-persistence`) |
| [`--json-schema`](param/23_json_schema.md) | [`JsonSchemaText`](type.md#type--10-jsonschematext) | JSON Schema for structured output |
| [`--mcp-config`](param/24_mcp_config.md) | [`McpConfigPath`](type.md#type--11-mcpconfigpath) | MCP server config file (repeatable) |

**Used by:** [`run`](command.md#command--1-run)

**Why NOT in this group:**
- `--dir`: sets runner working directory, not a claude flag
- `--max-tokens`: set via env var by runner, not a claude CLI flag
- `--dry-run`: prevents execution entirely, runner-only concern
- `--new-session`: controls runner session behavior, not forwarded to claude
- `--no-skip-permissions`: controls whether runner injects `--dangerously-skip-permissions`; consumed by runner, not forwarded to claude
- `--no-effort-max`: controls whether runner injects `--effort max`; consumed by runner, not forwarded to claude
- `--no-chrome`: controls whether runner injects `--chrome`; consumed by runner, not forwarded to claude

**Typical usage:**

```sh
clr -p "Fix bug" --model sonnet --verbose
```

---

### Group :: 2. Runner Control

Flags consumed by the runner itself before or instead of invoking the
claude subprocess. These control execution behavior, not Claude Code
behavior.

**Coherence test:** "Is this flag consumed by the runner, not Claude?" — YES for all 12.

**Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| [`--no-skip-permissions`](param/05_no_skip_permissions.md) | bool | Disable automatic permission bypass |
| [`--interactive`](param/06_interactive.md) | bool | Interactive TTY passthrough when message given |
| [`--new-session`](param/07_new_session.md) | bool | Start fresh session (disable default continuation) |
| [`--dir`](param/08_dir.md) | [`DirectoryPath`](type.md#type--2-directorypath) | Working directory |
| [`--max-tokens`](param/09_max_tokens.md) | [`TokenLimit`](type.md#type--3-tokenlimit) | Max output tokens |
| [`--session-dir`](param/10_session_dir.md) | [`DirectoryPath`](type.md#type--2-directorypath) | Session storage |
| [`--dry-run`](param/11_dry_run.md) | bool | Preview without executing |
| [`--verbosity`](param/12_verbosity.md) | [`VerbosityLevel`](type.md#type--5-verbositylevel) | Runner output level |
| [`--trace`](param/13_trace.md) | bool | Print env+command to stderr then execute |
| [`--no-ultrathink`](param/14_no_ultrathink.md) | bool | Disable default ultrathink message suffix |
| [`--no-effort-max`](param/18_no_effort_max.md) | bool | Suppress default `--effort max` injection |
| [`--no-chrome`](param/21_no_chrome.md) | bool | Suppress default `--chrome` injection |

**Used by:** [`run`](command.md#command--1-run)

**Why NOT in this group:**
- `--print`: forwarded to claude subprocess as `--print`
- `--model`: forwarded to claude subprocess as `--model`
- `--verbose`: forwarded to claude subprocess as `--verbose`
- `--effort`: forwarded to claude subprocess as `--effort <level>`
- `--dangerously-skip-permissions`: not a user flag — injected automatically by the runner (default-on)

**Typical usage:**

```sh
clr --dir /project --max-tokens 50000 --verbosity 4 "test"
clr --interactive "Continue this work" --dir /project
clr --new-session --dry-run "check command"
clr --trace "Fix bug" --dir /project
```

**Note:** `[MESSAGE]` is not in any group — it is a positional argument
serving as input content, not a control flag. `--help` is handled
separately as a universal override.

---

### Group :: 3. System Prompt

Flags that inject or extend the system prompt sent to the `claude` subprocess.
Although forwarded to claude (like Claude-Native Flags), they form a dedicated
group to keep parameter ranges contiguous: params 14–15 cannot join Group 1's
params 2–4 without introducing a gap in the range.

**Coherence test:** "Is this flag used to inject or extend the system prompt
sent to claude?" — YES for both.

**Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| [`--system-prompt`](param/15_system_prompt.md) | [`SystemPromptText`](type.md#type--6-systemprompttext) | Set system prompt (replaces the default) |
| [`--append-system-prompt`](param/16_append_system_prompt.md) | [`SystemPromptText`](type.md#type--6-systemprompttext) | Append text to the default system prompt |

**Used by:** [`run`](command.md#command--1-run)

**Why NOT in this group:**
- `--model`, `--print`, `--verbose`: Claude-native but not system-prompt related
- All Runner Control flags: consumed by the runner, not forwarded to claude
- `[MESSAGE]`: user-turn content, not system-turn context

**Typical usage:**

```sh
clr --system-prompt "You are a Rust expert." "Review this PR"
clr --append-system-prompt "Always respond in JSON." "List failing tests"
```

---

### Group :: 4. Isolated Subcommand

Parameters exclusive to the `isolated` subcommand. They configure the
credential-isolated execution environment and are not available on the
`run` command.

**Coherence test:** "Is this parameter exclusive to `clr isolated`?" — YES for both.

**Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| [`--creds`](param/19_creds.md) | [`CredentialsFilePath`](type.md#type--8-credentialsfilepath) | Credentials JSON file for isolation (required) |
| [`--timeout`](param/20_timeout.md) | [`TimeoutSecs`](type.md#type--9-timeoutsecs) | Max seconds to wait for isolated subprocess |

**Used by:** [`isolated`](command.md#command--2-isolated)

**Why NOT in other groups:**
- `--creds`: exclusive to `isolated`; sets credentials file — not applicable to `run`
- `--timeout`: exclusive to `isolated`; controls subprocess wait time — not applicable to `run`

**Typical usage:**

```sh
clr isolated --creds ~/.claude/.credentials.json "Fix bug"
clr isolated --creds /path/to/creds.json --timeout 120 "Refactor this"
```
