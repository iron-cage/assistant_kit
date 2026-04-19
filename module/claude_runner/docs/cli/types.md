# Types

### All Types (7 total)

| # | Type | Base | Used By | Purpose |
|---|------|------|---------|---------|
| 1 | `MessageText` | String | [`[MESSAGE]`](params.md#parameter--1-message) | Free-form prompt text |
| 2 | `DirectoryPath` | String | [`--dir`](params.md#parameter--8---dir), [`--session-dir`](params.md#parameter--10---session-dir) | Filesystem directory path |
| 3 | `TokenLimit` | u32 | [`--max-tokens`](params.md#parameter--9---max-tokens) | Maximum output token count |
| 4 | `ModelName` | String | [`--model`](params.md#parameter--3---model) | Claude model identifier |
| 5 | `VerbosityLevel` | u8 | [`--verbosity`](params.md#parameter--12---verbosity) | Runner output gate (0–5) |
| 6 | `SystemPromptText` | String | [`--system-prompt`](params.md#parameter--15---system-prompt), [`--append-system-prompt`](params.md#parameter--16---append-system-prompt) | Free-form system prompt text |
| 7 | `EffortLevel` | enum | [`--effort`](params.md#parameter--17---effort) | Reasoning effort level (low/medium/high/max) |

---

### Type :: 1. `MessageText`

Free-form prompt text sent to Claude Code. Multiple positional words in
argv are joined with a single space.

- **Base type:** String
- **Constraints:** any UTF-8 text; no length limit
- **Parsing:** all non-flag tokens collected, joined with `" "`
- **Used by:** [`[MESSAGE]`](params.md#parameter--1-message)

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
- **Used by:** [`--dir`](params.md#parameter--8---dir), [`--session-dir`](params.md#parameter--10---session-dir)

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
- **Used by:** [`--max-tokens`](params.md#parameter--9---max-tokens)

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
- **Used by:** [`--model`](params.md#parameter--3---model)

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
- **Used by:** [`--verbosity`](params.md#parameter--12---verbosity)

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
- **Used by:** [`--system-prompt`](params.md#parameter--15---system-prompt), [`--append-system-prompt`](params.md#parameter--16---append-system-prompt)

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
- **Used by:** [`--effort`](params.md#parameter--17---effort)

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
