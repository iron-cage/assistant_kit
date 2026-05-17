# Types

### All Types (11 total)

| # | Type | Base | Used By | Purpose |
|---|------|------|---------|---------|
| 1 | `MessageText` | String | [`[MESSAGE]`](param/01_message.md) | Free-form prompt text |
| 2 | `DirectoryPath` | String | [`--dir`](param/08_dir.md), [`--session-dir`](param/10_session_dir.md) | Filesystem directory path |
| 3 | `TokenLimit` | u32 | [`--max-tokens`](param/09_max_tokens.md) | Maximum output token count |
| 4 | `ModelName` | String | [`--model`](param/03_model.md) | Claude model identifier |
| 5 | `VerbosityLevel` | u8 | [`--verbosity`](param/12_verbosity.md) | Runner output gate (0–5) |
| 6 | `SystemPromptText` | String | [`--system-prompt`](param/15_system_prompt.md), [`--append-system-prompt`](param/16_append_system_prompt.md) | Free-form system prompt text |
| 7 | `EffortLevel` | enum | [`--effort`](param/17_effort.md) | Reasoning effort level (low/medium/high/max) |
| 8 | `CredentialsFilePath` | String | [`--creds`](param/19_creds.md) | Path to existing credentials JSON file |
| 9 | `TimeoutSecs` | u64 | [`--timeout`](param/20_timeout.md) | Subprocess wait limit in seconds |
| 10 | `JsonSchemaText` | String | [`--json-schema`](param/23_json_schema.md) | JSON Schema object string for structured output |
| 11 | `McpConfigPath` | String | [`--mcp-config`](param/24_mcp_config.md) | Filesystem path to MCP config JSON file |

**Total:** 11 types

---

### Type :: 1. `MessageText`

Free-form prompt text sent to Claude Code. Multiple positional words in
argv are joined with a single space.

- **Base type:** String
- **Constraints:** any UTF-8 text; no length limit
- **Parsing:** all non-flag tokens collected, joined with `" "`
- **Used by:** [`[MESSAGE]`](param/01_message.md)

```sh
clr "Fix the auth bug"      # single-token message
clr Fix the auth bug        # multi-token → "Fix the auth bug"
clr -- --not-a-flag         # after --, everything is positional
```

---

### Type :: 2. `DirectoryPath`

Filesystem path to a directory. Passed as-is to the subprocess working
directory or session storage environment variable.

- **Base type:** String
- **Constraints:** any valid filesystem path
- **Parsing:** consumed as the next token after `--dir` or `--session-dir`
- **Used by:** [`--dir`](param/08_dir.md), [`--session-dir`](param/10_session_dir.md)

```sh
clr --dir /home/user/project "Fix bug"
clr --session-dir /tmp/sessions "test"
```

---

### Type :: 3. `TokenLimit`

Maximum number of output tokens for the Claude Code subprocess. Set via
the `CLAUDE_CODE_MAX_OUTPUT_TOKENS` environment variable.

- **Base type:** u32 (unsigned 32-bit integer)
- **Constraints:** 0 to 4294967295
- **Default:** 200000
- **Parsing:** `str::parse::<u32>()`; rejects negative, float, non-numeric
- **Validation errors:**
  - Non-numeric: `"invalid --max-tokens value: {raw}\nExpected unsigned integer 0–4294967295"`
- **Used by:** [`--max-tokens`](param/09_max_tokens.md)

```sh
# Valid
clr --max-tokens 0 "test"            # minimum
clr --max-tokens 4294967295 "test"   # maximum

# Invalid
clr --max-tokens -1 "test"           # negative → error
clr --max-tokens 4294967296 "test"   # overflow → error
clr --max-tokens abc "test"          # non-numeric → error
```

---

### Type :: 4. `ModelName`

Identifier for a Claude model variant. Passed through to the claude
subprocess via `--model`.

- **Base type:** String
- **Constraints:** any non-empty string accepted by `claude --model`
- **Parsing:** consumed as the next token after `--model`
- **Used by:** [`--model`](param/03_model.md)

```sh
clr --model sonnet -p "Explain"
clr --model opus "Fix bug"
```

---

### Type :: 5. `VerbosityLevel`

Newtype wrapper over `u8` (range 0–5, default 3). Controls how much
diagnostic output the runner emits. Does not affect Claude Code output.

- **Base type:** u8
- **Constraints:** 0 to 5
- **Default:** 3 (normal)
- **Rust type:** `claude_runner::VerbosityLevel` (public newtype in `src/verbosity.rs`)
- **Parsing:** `VerbosityLevel::from_str()`; rejects non-integer and out-of-range
- **Validation errors:**
  - Non-integer: `"invalid verbosity level: {s}\nExpected integer 0–5"`
  - Out of range: `"verbosity level out of range: {n}\nExpected 0–5"`
- **Used by:** [`--verbosity`](param/12_verbosity.md)

**Level Semantics:**

| Level | Name | Predicate | Output |
|-------|------|-----------|--------|
| 0 | silent | — | All runner diagnostic output suppressed |
| 1 | errors | `shows_errors()` | Fatal errors only |
| 2 | warnings | `shows_warnings()` | Errors + warnings |
| 3 | normal | `shows_progress()` | Progress and status (default) |
| 4 | verbose | `shows_verbose_detail()` | Step-by-step command preview |
| 5 | debug | `shows_debug()` | Internal state, timing, paths |

**Note:** `--dry-run` output is always emitted regardless of verbosity level.
Verbosity gates runner diagnostics only; `--dry-run` is core feature output.

**Methods:**

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

---

### Type :: 6. `SystemPromptText`

Free-form text that sets or extends the system prompt sent to the claude
subprocess. Semantically distinct from `MessageText` (Type 1): this is the
model's behavioral context (system turn), not the user's conversational input
(user turn).

- **Base type:** String
- **Constraints:** any UTF-8 text; no length limit
- **Parsing:** consumed as the next token after `--system-prompt` or `--append-system-prompt`
- **Used by:** [`--system-prompt`](param/15_system_prompt.md), [`--append-system-prompt`](param/16_append_system_prompt.md)

```sh
clr --system-prompt "You are a Rust expert. Be concise." "Review PR"
clr --append-system-prompt "Always respond in JSON." "List failing tests"
```

---

### Type :: 7. `EffortLevel`

Reasoning effort level passed to the `claude` subprocess via `--effort`.
Controls how much computation Claude allocates to reasoning before responding.
`clr` defaults to `max` — the claude binary's own default is `medium`.

- **Base type:** enum (4 variants)
- **Rust type:** `claude_runner_core::EffortLevel` (public enum in `claude_runner_core/src/types.rs`)
- **Parsing:** `EffortLevel::from_str()`; rejects unknown strings
- **Default (clr):** `max` (injected automatically; see `--effort`)
- **Default (claude binary):** `medium`
- **Validation errors:**
  - Unknown level: `"unknown effort level: '{s}' — valid values: low, medium, high, max"`
- **Used by:** [`--effort`](param/17_effort.md)

**Level Semantics:**

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

---

### Type :: 8. `CredentialsFilePath`

Filesystem path to an existing JSON file containing Claude OAuth credentials.
The file is read before subprocess launch and written back in-place if Claude
refreshes its OAuth token during the run.

- **Base type:** String
- **Constraints:** file must exist and be readable at invocation time
- **Parsing:** consumed as the next token after `--creds`; path resolved
  against the caller's working directory, not the isolated temp `HOME`
- **Used by:** [`--creds`](param/19_creds.md)

```sh
clr isolated --creds ~/.claude/.credentials.json "Fix bug"
clr isolated --creds /tmp/test_creds.json --timeout 10 "hi"
```

---

### Type :: 9. `TimeoutSecs`

Unsigned integer representing seconds to wait for the isolated Claude
subprocess to complete. Zero causes immediate expiry — useful for testing
the credential-refresh path without waiting for Claude to start.

- **Base type:** u64 (unsigned 64-bit integer)
- **Constraints:** non-negative integer; no upper bound enforced by clr
- **Default:** 30
- **Parsing:** `str::parse::<u64>()`; rejects negative, float, non-numeric
- **Validation errors:**
  - Non-numeric: `"invalid --timeout value: {raw}\nExpected non-negative integer"`
- **Used by:** [`--timeout`](param/20_timeout.md)

```sh
clr isolated --creds creds.json --timeout 0 "test"    # immediate timeout
clr isolated --creds creds.json --timeout 30 "test"   # default (same as omitting)
clr isolated --creds creds.json --timeout 120 "test"  # 2-minute window
clr isolated --creds creds.json --timeout -1 "test"   # error: negative
```

---

### Type :: 10. `JsonSchemaText`

JSON Schema document passed as a string to `--json-schema`. Must be a valid
JSON object conforming to JSON Schema specification (draft-07 or later).

- **Base type:** String
- **Constraints:** must parse as valid JSON; must be a JSON object (`{…}`)
- **Parsing:** consumed as the next token after `--json-schema`
- **Validation errors:**
  - Not valid JSON: forwarded to `claude`; subprocess rejects with error
- **Used by:** [`--json-schema`](param/23_json_schema.md)

```sh
clr --json-schema '{"type":"object","properties":{"n":{"type":"string"}}}' "task"
clr --json-schema "$(cat schema.json)" "task"
```

---

### Type :: 11. `McpConfigPath`

Filesystem path to an MCP (Model Context Protocol) configuration JSON file.
Each value becomes one `--mcp-config` argument forwarded to the `claude`
subprocess.

- **Base type:** String
- **Constraints:** must be a valid filesystem path; file should exist and be valid JSON
- **Parsing:** consumed as the next token after `--mcp-config`; repeatable
- **Used by:** [`--mcp-config`](param/24_mcp_config.md)

```sh
clr --mcp-config /path/to/mcp.json "task"
clr --mcp-config server1.json --mcp-config server2.json "task"
```
