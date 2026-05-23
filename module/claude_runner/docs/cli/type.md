# Types

### All Types (12 total)

| # | Type | Fundamental Type | Parameters | Purpose |
|---|------|-----------------|------------|---------|
| 1 | `MessageText` | String | [`[MESSAGE]`](param/01_message.md) | Free-form prompt text sent to the `claude` subprocess |
| 2 | `DirectoryPath` | String | [`--dir`](param/08_dir.md), [`--session-dir`](param/10_session_dir.md) | Filesystem path to a directory |
| 3 | `TokenLimit` | u32 | [`--max-tokens`](param/09_max_tokens.md) | Maximum output token count |
| 4 | `ModelName` | String | [`--model`](param/03_model.md) | Claude model identifier string |
| 5 | `VerbosityLevel` | u8 | [`--verbosity`](param/12_verbosity.md) | Runner diagnostic output gate (0–5) |
| 6 | `SystemPromptText` | String | [`--system-prompt`](param/15_system_prompt.md), [`--append-system-prompt`](param/16_append_system_prompt.md) | Free-form system prompt text (system turn, not user turn) |
| 7 | `EffortLevel` | enum | [`--effort`](param/17_effort.md) | Reasoning effort level forwarded to the `claude` subprocess |
| 8 | `CredentialsFilePath` | String | [`--creds`](param/19_creds.md) | Path to an existing credentials JSON file |
| 9 | `TimeoutSecs` | u64 | [`--timeout`](param/20_timeout.md) | Subprocess wait limit in seconds |
| 10 | `JsonSchemaText` | String | [`--json-schema`](param/23_json_schema.md) | JSON Schema object string for structured output |
| 11 | `McpConfigPath` | String | [`--mcp-config`](param/24_mcp_config.md) | Filesystem path to an MCP configuration JSON file |
| 12 | `FilePath` | String | [`--file`](param/25_file.md) | Filesystem path to a readable file piped as subprocess stdin |

**Total:** 12 types

---

### Type :: 1. `MessageText`

Free-form prompt text sent to Claude Code. Multiple positional words in
argv are joined with a single space.

- **Purpose:** Free-form prompt text sent to the `claude` subprocess
- **Fundamental Type:** String
- **Constants:** —
- **Constraints:** any UTF-8 text; no length limit
- **Parsing:** all non-flag tokens collected, joined with `" "`
- **Methods:** —

```sh
clr "Fix the auth bug"      # single-token message
clr Fix the auth bug        # multi-token → "Fix the auth bug"
clr -- --not-a-flag         # after --, everything is positional
```

### Referenced Parameters

- [`[MESSAGE]`](param/01_message.md)

### Referenced Commands

- [`run`](command.md#command--1-run) (via `[MESSAGE]`)
- [`isolated`](command.md#command--2-isolated) (via `[MESSAGE]`)
---

### Type :: 2. `DirectoryPath`

Filesystem path to a directory. Passed as-is to the subprocess working
directory or session storage environment variable.

- **Purpose:** Filesystem path to a directory
- **Fundamental Type:** String
- **Constants:** —
- **Constraints:** any valid filesystem path
- **Parsing:** consumed as the next token after `--dir` or `--session-dir`
- **Methods:** —

```sh
clr --dir /home/user/project "Fix bug"
clr --session-dir /tmp/sessions "test"
```

### Referenced Parameters

- [`--dir`](param/08_dir.md)
- [`--session-dir`](param/10_session_dir.md)

### Referenced Commands

- [`run`](command.md#command--1-run) (via `--dir`, `--session-dir`)
---

### Type :: 3. `TokenLimit`

Maximum number of output tokens for the Claude Code subprocess. Set via
the `CLAUDE_CODE_MAX_OUTPUT_TOKENS` environment variable.

- **Purpose:** Maximum output token count
- **Fundamental Type:** u32 (unsigned 32-bit integer)
- **Constants:** —
- **Constraints:** 0 to 4294967295; default 200000
- **Parsing:** `str::parse::<u32>()`; rejects negative, float, non-numeric
- **Methods:** —

```sh
# Valid
clr --max-tokens 0 "test"            # minimum
clr --max-tokens 4294967295 "test"   # maximum

# Invalid
clr --max-tokens -1 "test"           # negative → error
clr --max-tokens 4294967296 "test"   # overflow → error
clr --max-tokens abc "test"          # non-numeric → error
```

### Referenced Parameters

- [`--max-tokens`](param/09_max_tokens.md)

### Referenced Commands

- [`run`](command.md#command--1-run) (via `--max-tokens`)
---

### Type :: 4. `ModelName`

Identifier for a Claude model variant. Passed through to the claude
subprocess via `--model`.

- **Purpose:** Claude model identifier string
- **Fundamental Type:** String
- **Constants:** —
- **Constraints:** any non-empty string accepted by `claude --model`
- **Parsing:** consumed as the next token after `--model`
- **Methods:** —

```sh
clr --model sonnet -p "Explain"
clr --model opus "Fix bug"
```

### Referenced Parameters

- [`--model`](param/03_model.md)

### Referenced Commands

- [`run`](command.md#command--1-run) (via `--model`)
---

### Type :: 5. `VerbosityLevel`

Newtype wrapper over `u8` (range 0–5, default 3). Controls how much
diagnostic output the runner emits. Does not affect Claude Code output.

- **Purpose:** Runner diagnostic output gate (0–5)
- **Fundamental Type:** u8
- **Constants:** see below
- **Constraints:** 0 to 5; default 3
- **Parsing:** `VerbosityLevel::from_str()`; rejects non-integer and out-of-range
- **Methods:** see below

### Constants

| Level | Name | Predicate | Output |
|-------|------|-----------|--------|
| 0 | silent | — | All runner diagnostic output suppressed |
| 1 | errors | `shows_errors()` | Fatal errors only |
| 2 | warnings | `shows_warnings()` | Errors + warnings |
| 3 | normal | `shows_progress()` | Progress and status (default) |
| 4 | verbose | `shows_verbose_detail()` | Step-by-step command preview |
| 5 | debug | `shows_debug()` | Internal state, timing, paths |

`--dry-run` output is always emitted regardless of verbosity level.
Verbosity gates runner diagnostics only; `--dry-run` is core feature output.

### Methods

| Method | Returns true when |
|--------|-------------------|
| `get()` | — (returns inner u8) |
| `shows_errors()` | level >= 1 |
| `shows_warnings()` | level >= 2 |
| `shows_progress()` | level >= 3 |
| `shows_verbose_detail()` | level >= 4 |
| `shows_debug()` | level >= 5 |

```sh
clr --verbosity 0 "silent"   # suppress runner diagnostic output
clr --verbosity 4 "debug"    # command preview on stderr
clr --verbosity 6 "test"     # error: out of range
```

### Referenced Parameters

- [`--verbosity`](param/12_verbosity.md)

### Referenced Commands

- [`run`](command.md#command--1-run) (via `--verbosity`)
---

### Type :: 6. `SystemPromptText`

Free-form text that sets or extends the system prompt sent to the claude
subprocess. Semantically distinct from `MessageText` (Type 1): this is the
model's behavioral context (system turn), not the user's conversational input
(user turn).

- **Purpose:** Free-form system prompt text (system turn, not user turn)
- **Fundamental Type:** String
- **Constants:** —
- **Constraints:** any UTF-8 text; no length limit
- **Parsing:** consumed as the next token after `--system-prompt` or `--append-system-prompt`
- **Methods:** —

```sh
clr --system-prompt "You are a Rust expert. Be concise." "Review PR"
clr --append-system-prompt "Always respond in JSON." "List failing tests"
```

### Referenced Parameters

- [`--system-prompt`](param/15_system_prompt.md)
- [`--append-system-prompt`](param/16_append_system_prompt.md)

### Referenced Commands

- [`run`](command.md#command--1-run) (via `--system-prompt`, `--append-system-prompt`)
---

### Type :: 7. `EffortLevel`

Reasoning effort level passed to the `claude` subprocess via `--effort`.
Controls how much computation Claude allocates to reasoning before responding.
`clr` defaults to `max` — the claude binary's own default is `medium`.

- **Purpose:** Reasoning effort level forwarded to the `claude` subprocess
- **Fundamental Type:** enum (4 variants)
- **Constants:** see below
- **Constraints:** one of `low`, `medium`, `high`, `max`; `clr` default `max`; `claude` binary default `medium`
- **Parsing:** `EffortLevel::from_str()`; rejects unknown strings
- **Methods:** —

### Constants

| Level | Rust Variant | CLI String | Reasoning Budget |
|-------|-------------|------------|------------------|
| `low` | `EffortLevel::Low` | `low` | Minimal — fast, lowest token cost |
| `medium` | `EffortLevel::Medium` | `medium` | Standard — claude binary default |
| `high` | `EffortLevel::High` | `high` | Extended — more deliberate reasoning |
| `max` | `EffortLevel::Max` | `max` | Maximum — `clr` default for automation |

```sh
clr --effort max "Fix bug"     # explicitly maximum (same as default)
clr --effort high "Fix bug"    # extended reasoning
clr --effort medium "Fix bug"  # claude binary's default
clr --effort low "Fix bug"     # fast, minimal reasoning
clr --effort bad "Fix bug"     # error: unknown effort level
```

### Referenced Parameters

- [`--effort`](param/17_effort.md)

### Referenced Commands

- [`run`](command.md#command--1-run) (via `--effort`)
---

### Type :: 8. `CredentialsFilePath`

Filesystem path to an existing JSON file containing Claude OAuth credentials.
The file is read before subprocess launch and written back in-place if Claude
refreshes its OAuth token during the run.

- **Purpose:** Path to an existing credentials JSON file
- **Fundamental Type:** String
- **Constants:** —
- **Constraints:** file must exist and be readable at invocation time
- **Parsing:** consumed as the next token after `--creds`; path resolved
  against the caller's working directory, not the isolated temp `HOME`
- **Methods:** —

```sh
clr isolated --creds ~/.claude/.credentials.json "Fix bug"
clr isolated --creds /tmp/test_creds.json --timeout 10 "hi"
```

### Referenced Parameters

- [`--creds`](param/19_creds.md)

### Referenced Commands

- [`isolated`](command.md#command--2-isolated) (via `--creds`)
---

### Type :: 9. `TimeoutSecs`

Unsigned integer representing seconds to wait for the isolated Claude
subprocess to complete. Zero causes immediate expiry — useful for testing
the credential-refresh path without waiting for Claude to start.

- **Purpose:** Subprocess wait limit in seconds
- **Fundamental Type:** u64 (unsigned 64-bit integer)
- **Constants:** —
- **Constraints:** non-negative integer; no upper bound enforced by clr; default 30
- **Parsing:** `str::parse::<u64>()`; rejects negative, float, non-numeric
- **Methods:** —

```sh
clr isolated --creds creds.json --timeout 0 "test"    # immediate timeout
clr isolated --creds creds.json --timeout 30 "test"   # default (same as omitting)
clr isolated --creds creds.json --timeout 120 "test"  # 2-minute window
clr isolated --creds creds.json --timeout -1 "test"   # error: negative
```

### Referenced Parameters

- [`--timeout`](param/20_timeout.md)

### Referenced Commands

- [`isolated`](command.md#command--2-isolated) (via `--timeout`)
---

### Type :: 10. `JsonSchemaText`

JSON Schema document passed as a string to `--json-schema`. Must be a valid
JSON object conforming to JSON Schema specification (draft-07 or later).

- **Purpose:** JSON Schema object string for structured output
- **Fundamental Type:** String
- **Constants:** —
- **Constraints:** must parse as valid JSON; must be a JSON object (`{…}`)
- **Parsing:** consumed as the next token after `--json-schema`
- **Methods:** —

```sh
clr --json-schema '{"type":"object","properties":{"n":{"type":"string"}}}' "task"
clr --json-schema "$(cat schema.json)" "task"
```

### Referenced Parameters

- [`--json-schema`](param/23_json_schema.md)

### Referenced Commands

- [`run`](command.md#command--1-run) (via `--json-schema`)
---

### Type :: 11. `McpConfigPath`

Filesystem path to an MCP (Model Context Protocol) configuration JSON file.
Each value becomes one `--mcp-config` argument forwarded to the `claude`
subprocess.

- **Purpose:** Filesystem path to an MCP configuration JSON file
- **Fundamental Type:** String
- **Constants:** —
- **Constraints:** must be a valid filesystem path; file should exist and be valid JSON
- **Parsing:** consumed as the next token after `--mcp-config`; repeatable
- **Methods:** —

```sh
clr --mcp-config /path/to/mcp.json "task"
clr --mcp-config server1.json --mcp-config server2.json "task"
```

### Referenced Parameters

- [`--mcp-config`](param/24_mcp_config.md)

### Referenced Commands

- [`run`](command.md#command--1-run) (via `--mcp-config`)
---

### Type :: 12. `FilePath`

Filesystem path to a readable file whose content is piped as standard input
to the `claude` subprocess. The file is opened by the runner at spawn time;
if it cannot be read, `execute()` returns an error.

- **Purpose:** Filesystem path to a readable file piped as subprocess stdin
- **Fundamental Type:** String
- **Constants:** —
- **Constraints:** file must exist and be readable at invocation time
- **Parsing:** consumed as the next token after `--file`; path resolved
  against the caller's working directory
- **Methods:** —

```sh
clr --file notes.md "Summarise the above"
clr --file /tmp/diff.txt -p "Review this diff"
```

### Referenced Parameters

- [`--file`](param/25_file.md)

### Referenced Commands

- [`run`](command.md#command--1-run) (via `--file`)