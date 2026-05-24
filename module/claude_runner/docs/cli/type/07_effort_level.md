# CLI Type: EffortLevel

Reasoning effort level passed to the `claude` subprocess via `--effort`.
Controls how much computation Claude allocates to reasoning before responding.
`clr` defaults to `max` — the claude binary's own default is `medium`.

- **Purpose:** Reasoning effort level forwarded to the `claude` subprocess
- **Fundamental Type:** enumeration (4 variants)
- **Constants:** see below
- **Constraints:** one of `low`, `medium`, `high`, `max`; `clr` default `max`; `claude` binary default `medium`
- **Parsing:** string-to-enum parse; rejects unknown strings
- **Methods:** —

### Constants

| Level | CLI String | Reasoning Budget |
|-------|------------|------------------|
| `low` | `low` | Minimal — fast, lowest token cost |
| `medium` | `medium` | Standard — claude binary default |
| `high` | `high` | Extended — more deliberate reasoning |
| `max` | `max` | Maximum — `clr` default for automation |

```sh
clr --effort max "Fix bug"     # explicitly maximum (same as default)
clr --effort high "Fix bug"    # extended reasoning
clr --effort medium "Fix bug"  # claude binary's default
clr --effort low "Fix bug"     # fast, minimal reasoning
clr --effort bad "Fix bug"     # error: unknown effort level
```

### Referenced Commands

| # | Command | Via Parameter |
|---|---------|---------------|
| 1 | [`run`](../command/01_run.md) | `--effort` |
| 5 | [`ask`](../command/05_ask.md) | `--effort` |

### Referenced Parameters

| # | Parameter | Commands |
|---|-----------|----------|
| 17 | [`--effort`](../param/017_effort.md) | 2 |
