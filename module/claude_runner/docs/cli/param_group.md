# Parameter Groups

### All Groups (4 total)

| # | Group | Parameters | Purpose |
|---|-------|------------|---------|
| 1 | Claude-Native Flags | 6 | Pass selected `claude` binary flags through without runner modification |
| 2 | Runner Control | 16 | Control runner execution behavior — before, during, or instead of subprocess invocation |
| 3 | System Prompt | 2 | Inject or extend the behavioral system context sent to the `claude` subprocess |
| 4 | Credential Operations | 3 | Configure credential-isolated execution for `clr isolated` and `clr refresh` |

**Total:** 4 groups

---

### Group :: 1. Claude-Native Flags

**Pattern:** Forwarded as-is to the `claude` subprocess via `ClaudeCommand` builder calls; runner does not interpret or transform these values.

**Purpose:** Pass selected `claude` binary flags through without runner modification.

Session continuation (`-c`) is applied automatically and is not exposed as a user flag.
Use `--new-session` (Runner Control) to disable it.

`--dangerously-skip-permissions` is injected automatically by `clr` (default-on).
Use `--no-skip-permissions` (Runner Control) to disable the automatic bypass.

`--effort max` is injected automatically by `clr` (default-on). Use `--effort <level>`
to override or `--no-effort-max` (Runner Control) to suppress entirely.

### Semantic Coherence Test

"Is this flag consumed by the claude subprocess?" — YES for all 6.

### Parameters

| Parameter | Type | Default | Role in Group | Description |
|-----------|------|---------|---------------|-------------|
| [`-p`/`--print`](param/02_print.md) | bool | auto | Print mode selector | Print mode (default when message given) |
| [`--model`](param/03_model.md) | [`ModelName`](type.md#type--4-modelname) | — | Model selection | Model to use |
| [`--verbose`](param/04_verbose.md) | bool | false | Verbosity toggle | Enable Claude verbose output |
| [`--effort`](param/17_effort.md) | [`EffortLevel`](type.md#type--7-effortlevel) | max | Effort override | Reasoning effort level (default: max) |
| [`--json-schema`](param/23_json_schema.md) | [`JsonSchemaText`](type.md#type--10-jsonschematext) | — | Output structure constraint | JSON Schema for structured output |
| [`--mcp-config`](param/24_mcp_config.md) | [`McpConfigPath`](type.md#type--11-mcpconfigpath) | — | Tool server config | MCP server config (repeatable) |

### Why NOT X

- `--dir`: sets runner working directory, not a claude flag
- `--max-tokens`: set via env var by runner, not a claude CLI flag
- `--dry-run`: prevents execution entirely, runner-only concern
- `--new-session`: controls runner session behavior, not forwarded to claude
- `--no-skip-permissions`: controls whether runner injects `--dangerously-skip-permissions`; consumed by runner, not forwarded to claude
- `--no-effort-max`: controls whether runner injects `--effort max`; consumed by runner, not forwarded to claude
- `--no-chrome`: controls whether runner injects `--chrome`; consumed by runner, not forwarded to claude
- `--no-persist`: controls whether runner injects `--no-session-persistence`; consumed by runner, not forwarded to claude

### Invariants

All parameters are forwarded to the subprocess as-is. The runner applies no transformation to their values.

### Notes

—

### Referenced Commands

| # | Command | Membership | Excluded Params | Notes |
|---|---------|-----------|-----------------|-------|
| 1 | `run` | all | — | |

### Referenced Tests

- [`tests/docs/cli/param_group/01_claude_native_flags.md`](../../tests/docs/cli/param_group/01_claude_native_flags.md)

**Typical usage:**

```sh
clr -p "Fix bug" --model sonnet --verbose
```

### Referenced User Stories

| # | User Story | Notes |
|---|-----------|-------|
| 1 | [002 Print Mode Capture](user_story/002_print_mode_capture.md) | `--print` is a Claude-native flag |
| 2 | [012 Code Block Extraction](user_story/012_code_block_extraction.md) | `--print` used with `--strip-fences` |
| 3 | [013 Structured JSON Pipeline](user_story/013_structured_json_pipeline.md) | `--json-schema` drives structured output |

---

### Group :: 2. Runner Control

**Pattern:** Consumed by the runner before subprocess launch; may suppress, override, or replace automatic flag injections; never forwarded directly.

**Purpose:** Control runner execution behavior — before, during, or instead of subprocess invocation.

### Semantic Coherence Test

"Is this flag consumed by the runner, not Claude?" — YES for all 16.

### Parameters

| Parameter | Type | Default | Role in Group | Description |
|-----------|------|---------|---------------|-------------|
| [`--no-skip-permissions`](param/05_no_skip_permissions.md) | bool | false | Injection suppressor | Disable automatic permission bypass |
| [`--interactive`](param/06_interactive.md) | bool | false | Mode selector | Interactive TTY passthrough when message given |
| [`--new-session`](param/07_new_session.md) | bool | false | Session mode | Start fresh session (disable default continuation) |
| [`--dir`](param/08_dir.md) | [`DirectoryPath`](type.md#type--2-directorypath) | cwd | Working directory | Working directory for subprocess |
| [`--max-tokens`](param/09_max_tokens.md) | [`TokenLimit`](type.md#type--3-tokenlimit) | 200000 | Token cap | Max output tokens |
| [`--session-dir`](param/10_session_dir.md) | [`DirectoryPath`](type.md#type--2-directorypath) | — | Session storage | Session storage directory |
| [`--dry-run`](param/11_dry_run.md) | bool | false | Execution gate | Preview without executing |
| [`--verbosity`](param/12_verbosity.md) | [`VerbosityLevel`](type.md#type--5-verbositylevel) | 3 | Diagnostic level | Runner output gate level |
| [`--trace`](param/13_trace.md) | bool | false | Trace mode | Print env+command to stderr then execute |
| [`--no-ultrathink`](param/14_no_ultrathink.md) | bool | false | Injection suppressor | Disable default ultrathink message suffix |
| [`--no-effort-max`](param/18_no_effort_max.md) | bool | false | Injection suppressor | Suppress default `--effort max` injection |
| [`--no-chrome`](param/21_no_chrome.md) | bool | false | Injection suppressor | Suppress default `--chrome` injection |
| [`--no-persist`](param/22_no_persist.md) | bool | false | Injection suppressor | Disable session persistence injection |
| [`--file`](param/25_file.md) | [`FilePath`](type.md#type--12-filepath) | — | Stdin source | File content piped as subprocess stdin |
| [`--strip-fences`](param/26_strip_fences.md) | bool | false | Output processor | Strip outermost markdown code fences from stdout |
| [`--keep-claudecode`](param/27_keep_claudecode.md) | bool | false | Env filter | Preserve `CLAUDECODE` env var in subprocess (default: removed) |

### Why NOT X

- `--print`: forwarded to claude subprocess as `--print`
- `--model`: forwarded to claude subprocess as `--model`
- `--verbose`: forwarded to claude subprocess as `--verbose`
- `--effort`: forwarded to claude subprocess as `--effort <level>`
- `--dangerously-skip-permissions`: not a user flag — injected automatically by the runner (default-on)

### Invariants

No parameter is forwarded to the subprocess unchanged. Each is fully consumed by runner logic before subprocess construction.

### Notes

`[MESSAGE]` is not in any group — it is a positional argument serving as input content,
not a control flag. `--help` is handled separately as a universal override.

### Referenced Commands

| # | Command | Membership | Excluded Params | Notes |
|---|---------|-----------|-----------------|-------|
| 1 | `run` | all | — | |

### Referenced Tests

- [`tests/docs/cli/param_group/02_runner_control.md`](../../tests/docs/cli/param_group/02_runner_control.md)

**Typical usage:**

```sh
clr --dir /project --max-tokens 50000 --verbosity 4 "test"
clr --interactive "Continue this work" --dir /project
clr --new-session --dry-run "check command"
clr --trace "Fix bug" --dir /project
```

### Referenced User Stories

| # | User Story | Notes |
|---|-----------|-------|
| 1 | [001 Interactive REPL](user_story/001_interactive_repl.md) | `--dir` scopes REPL to a project |
| 2 | [003 Interactive With Message](user_story/003_interactive_with_message.md) | `--interactive` opts out of print mode |
| 3 | [004 Dry-run Preview](user_story/004_dry_run_preview.md) | `--dry-run` inspects assembled command |
| 4 | [005 Project-specific Execution](user_story/005_project_specific_execution.md) | `--dir` + `--session-dir` isolate projects |
| 5 | [006 Verbose Debugging](user_story/006_verbose_debugging.md) | `--verbosity` gates diagnostic output |
| 6 | [007 Fresh Session](user_story/007_fresh_session.md) | `--new-session` suppresses `-c` |
| 7 | [008 Trace Execution](user_story/008_trace_execution.md) | `--trace` prints env+cmd then executes |
| 8 | [011 File Input](user_story/011_file_input.md) | `--file` pipes file content as stdin |
| 9 | [012 Code Block Extraction](user_story/012_code_block_extraction.md) | `--strip-fences` removes output fences |

---

### Group :: 3. System Prompt

**Pattern:** Forwarded to the `claude` subprocess via `--system-prompt` / `--append-system-prompt` flags; distinct from Claude-Native (content-injection, not control).

**Purpose:** Inject or extend the behavioral system context sent to the `claude` subprocess.

Although forwarded to claude (like Claude-Native Flags), they form a dedicated
group to keep parameter ranges contiguous: params 15–16 cannot join Group 1's
params 2–4 without introducing a gap in the range.

### Semantic Coherence Test

"Is this flag used to inject or extend the system prompt sent to claude?" — YES for both.

### Parameters

| Parameter | Type | Default | Role in Group | Description |
|-----------|------|---------|---------------|-------------|
| [`--system-prompt`](param/15_system_prompt.md) | [`SystemPromptText`](type.md#type--6-systemprompttext) | — | Full replacement | Set system prompt (replaces the default) |
| [`--append-system-prompt`](param/16_append_system_prompt.md) | [`SystemPromptText`](type.md#type--6-systemprompttext) | — | Additive extension | Append text to the default system prompt |

### Why NOT X

- `--model`, `--print`, `--verbose`: Claude-native but not system-prompt related
- All Runner Control flags: consumed by the runner, not forwarded to claude
- `[MESSAGE]`: user-turn content, not system-turn context

### Invariants

Both parameters produce system-prompt-related subprocess flags (`--system-prompt` or
`--append-system-prompt`). Neither is runner-only.

### Notes

—

### Referenced Commands

| # | Command | Membership | Excluded Params | Notes |
|---|---------|-----------|-----------------|-------|
| 1 | `run` | all | — | |

### Referenced Tests

- [`tests/docs/cli/param_group/03_system_prompt.md`](../../tests/docs/cli/param_group/03_system_prompt.md)

**Typical usage:**

```sh
clr --system-prompt "You are a Rust expert." "Review this PR"
clr --append-system-prompt "Always respond in JSON." "List failing tests"
```

### Referenced User Stories

| # | User Story | Notes |
|---|-----------|-------|
| 1 | [009 Custom System Prompt](user_story/009_custom_system_prompt.md) | Primary user story for this group |

---

### Group :: 4. Credential Operations

**Pattern:** Shared by `clr isolated` and `clr refresh`; configure the credential-isolated execution environment; not accepted by `clr run`.

**Purpose:** Configure credential-isolated execution for `clr isolated` and `clr refresh`.

### Semantic Coherence Test

"Is this parameter used by credential-operating commands (`isolated`/`refresh`) and not by `run`?" — YES for all 3.

### Parameters

| Parameter | Type | Default | Role in Group | Description |
|-----------|------|---------|---------------|-------------|
| [`--creds`](param/19_creds.md) | [`CredentialsFilePath`](type.md#type--8-credentialsfilepath) | — | Credentials source | Credentials JSON file (required) |
| [`--timeout`](param/20_timeout.md) | [`TimeoutSecs`](type.md#type--9-timeoutsecs) | 30/45 | Duration limit | Max seconds to wait (30 isolated, 45 refresh) |
| [`--trace`](param/13_trace.md) | bool | false | Trace mode | Print underlying call details to stderr then execute |

### Why NOT X

- `--creds`: exclusive to credential ops; sets credentials file — not applicable to `run`
- `--timeout`: exclusive to credential ops; controls subprocess wait time — not applicable to `run`
- `--trace`: also in Runner Control (Group 2) for `run`; listed here because it applies to credential ops too

### Invariants

`--creds` and `--timeout` are exclusive to `clr isolated` and `clr refresh`. Neither is accepted by `clr run`. `--trace` is cross-command (also in Group 2).

### Notes

`--timeout` has different defaults per command: 30s for `isolated` (general task execution), 45s for `refresh` (allows headroom for slow OAuth token exchange).

### Referenced Commands

| # | Command | Membership | Excluded Params | Notes |
|---|---------|-----------|-----------------|-------|
| 1 | `isolated` | all | — | |
| 2 | `refresh` | all | — | |

### Referenced Tests

- [`tests/docs/cli/param_group/04_credential_operations.md`](../../tests/docs/cli/param_group/04_credential_operations.md)

**Typical usage:**

```sh
clr isolated --creds ~/.claude/.credentials.json "Fix bug"
clr isolated --creds /path/to/creds.json --timeout 120 --trace "Refactor this"
clr refresh --creds ~/.claude/.credentials.json
clr refresh --creds creds.json --timeout 90 --trace
```

### Referenced User Stories

| # | User Story | Notes |
|---|-----------|-------|
| 1 | [010 Credential-isolated Execution](user_story/010_credential_isolated_execution.md) | `isolated` user story |
| 2 | [014 Credential Refresh](user_story/014_credential_refresh.md) | `refresh` user story |
