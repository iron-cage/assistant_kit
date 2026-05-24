# CLI Type: VerbosityLevel

Integer value in the range 0–5 (default 3). Controls how much diagnostic
output the runner emits. Does not affect Claude Code output.

- **Purpose:** Runner diagnostic output gate (0–5)
- **Fundamental Type:** unsigned 8-bit integer
- **Constants:** see below
- **Constraints:** 0 to 5; default 3
- **Parsing:** integer parse; rejects non-integer and out-of-range
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
| `get()` | — (returns integer value) |
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

### Referenced Commands

| # | Command | Via Parameter |
|---|---------|---------------|
| 1 | [`run`](../command/01_run.md) | `--verbosity` |
| 5 | [`ask`](../command/05_ask.md) | `--verbosity` |

### Referenced Parameters

| # | Parameter | Commands |
|---|-----------|----------|
| 12 | [`--verbosity`](../param/012_verbosity.md) | 2 |
