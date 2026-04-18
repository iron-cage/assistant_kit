# Parameter Groups

### All Groups (3 total)

| # | Group | Parameters | Purpose |
|---|-------|------------|---------|
| 1 | Claude-Native Flags | 4 | Flags passed through to the claude subprocess |
| 2 | Runner Control | 11 | Flags consumed by the runner itself |
| 3 | System Prompt | 2 | Flags that inject or extend the system prompt sent to claude |

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

**Coherence test:** "Is this flag consumed by the claude subprocess?" — YES for all 4.

**Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| [`-p`/`--print`](params.md#parameter--2---print) | bool | Print mode (default when message given) |
| [`--model`](params.md#parameter--3---model) | [`ModelName`](types.md#type--4-modelname) | Model selection |
| [`--verbose`](params.md#parameter--4---verbose) | bool | Claude verbose output |
| [`--effort`](params.md#parameter--17---effort) | [`EffortLevel`](types.md#type--7-effortlevel) | Reasoning effort level (default: max) |

**Used by:** [`run`](commands.md#command--1-run)

**Why NOT in this group:**
- `--dir`: sets runner working directory, not a claude flag
- `--max-tokens`: set via env var by runner, not a claude CLI flag
- `--dry-run`: prevents execution entirely, runner-only concern
- `--new-session`: controls runner session behavior, not forwarded to claude
- `--no-skip-permissions`: controls whether runner injects `--dangerously-skip-permissions`; consumed by runner, not forwarded to claude
- `--no-effort-max`: controls whether runner injects `--effort max`; consumed by runner, not forwarded to claude

**Typical usage:**

```sh
clr -p "Fix bug" --model sonnet --verbose
```

---

### Group :: 2. Runner Control

Flags consumed by the runner itself before or instead of invoking the
claude subprocess. These control execution behavior, not Claude Code
behavior.

**Coherence test:** "Is this flag consumed by the runner, not Claude?" — YES for all 11.

**Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| [`--no-skip-permissions`](params.md#parameter--5---no-skip-permissions) | bool | Disable automatic permission bypass |
| [`--interactive`](params.md#parameter--6---interactive) | bool | Interactive TTY passthrough when message given |
| [`--new-session`](params.md#parameter--7---new-session) | bool | Start fresh session (disable default continuation) |
| [`--dir`](params.md#parameter--8---dir) | [`DirectoryPath`](types.md#type--2-directorypath) | Working directory |
| [`--max-tokens`](params.md#parameter--9---max-tokens) | [`TokenLimit`](types.md#type--3-tokenlimit) | Max output tokens |
| [`--session-dir`](params.md#parameter--10---session-dir) | [`DirectoryPath`](types.md#type--2-directorypath) | Session storage |
| [`--dry-run`](params.md#parameter--11---dry-run) | bool | Preview without executing |
| [`--verbosity`](params.md#parameter--12---verbosity) | [`VerbosityLevel`](types.md#type--5-verbositylevel) | Runner output level |
| [`--trace`](params.md#parameter--13---trace) | bool | Print env+command to stderr then execute |
| [`--no-ultrathink`](params.md#parameter--14---no-ultrathink) | bool | Disable default ultrathink message suffix |
| [`--no-effort-max`](params.md#parameter--18---no-effort-max) | bool | Suppress default `--effort max` injection |

**Used by:** [`run`](commands.md#command--1-run)

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
| [`--system-prompt`](params.md#parameter--15---system-prompt) | [`SystemPromptText`](types.md#type--6-systemprompttext) | Set system prompt (replaces the default) |
| [`--append-system-prompt`](params.md#parameter--16---append-system-prompt) | [`SystemPromptText`](types.md#type--6-systemprompttext) | Append text to the default system prompt |

**Used by:** [`run`](commands.md#command--1-run)

**Why NOT in this group:**
- `--model`, `--print`, `--verbose`: Claude-native but not system-prompt related
- All Runner Control flags: consumed by the runner, not forwarded to claude
- `[MESSAGE]`: user-turn content, not system-turn context

**Typical usage:**

```sh
clr --system-prompt "You are a Rust expert." "Review this PR"
clr --append-system-prompt "Always respond in JSON." "List failing tests"
```
