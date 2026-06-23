# CLI Parameter: --output-style

Runner-level output rendering control for print-mode (`run`/`ask`). Controls whether
`clr` passes captured stdout through `render_summary()` in `summary.rs` (`summary`) or
returns the raw claude output without post-processing (`raw`).

- **Type:** enum (`summary` | `raw`)
- **Default:** `summary`
- **Command:** [`run`](../command/01_run.md), [`ask`](../command/05_ask.md)
- **Group:** [Runner Control](../param_group/02_runner_control.md)

```sh
clr -p "summarise logs"                          # default: summary header rendered
clr -p "summarise logs" --output-style summary   # explicit: same as default
clr -p "summarise logs" --output-style raw       # bypass render_summary(); raw claude output
CLR_OUTPUT_STYLE=raw clr -p "task"              # env-var equivalent of --output-style raw
clr -p "task" --output-style invalid            # exit 1: invalid value
```

**Separation of concerns:** `--output-style` is a runner-level rendering parameter
distinct from `--output-format` (a claude passthrough). `--output-format` selects what
Claude emits (`text`/`json`/`stream-json`) and is forwarded verbatim to the `claude`
subprocess. `--output-style` controls what `clr` does with that output. When
`--output-style summary` is active and `--output-format` is absent, `clr` injects
`--output-format json` automatically so that `render_summary()` receives parseable input.

**Default-summary behavior:** `clr run -m "..."` (no flags) uses `--output-style summary`
by default, which routes stdout through `render_summary()`. If rendering fails (non-JSON
input, e.g. when `--output-format text` is explicitly set), `render_summary()` returns
`None` and `clr` falls back to the raw output unchanged (`unwrap_or(out)` in
`execution.rs`). The default therefore degrades gracefully.

**Validation:** Invalid values exit 1 immediately with:
`invalid output-style '{v}' — expected: summary, raw`

**Legacy alias:** `--output-format summary` remains supported for backward compatibility
as a special alias in `builder.rs`. It co-operates with `--output-style`: both the
builder alias (translates `summary`→`json` for the subprocess) and the execution
predicate (triggers `render_summary()`) fire independently when `--output-format summary`
is specified without an explicit `--output-style`.

**Interactive mode:** `--output-style` is a print-mode concept; in interactive mode
(`run_interactive()`) no stdout capture occurs and the parameter has no effect.

**Env var:** `CLR_OUTPUT_STYLE` — applied when `--output-style` is absent; accepts
`summary` or `raw`; invalid values exit 1 with:
`CLR_OUTPUT_STYLE: invalid value '{v}' — expected: summary, raw`
CLI flag wins when both are set (standard precedence).

### Combinations

| `--output-format` | `--output-style` | Behavior |
|-------------------|------------------|----------|
| absent | `summary` (default) | `clr` injects `--output-format json`; `render_summary()` renders key:val header |
| `text` | `summary` | `--output-format text` forwarded; `render_summary()` receives plain text; returns `None`; raw text passed through |
| `json` | `summary` | `--output-format json` forwarded; `render_summary()` renders key:val header |
| `stream-json` | `summary` | `--output-format stream-json` forwarded; `render_summary()` receives non-JSON stream; returns `None`; raw stream passed through |
| `summary` (legacy) | absent | builder alias translates to `json`; execution predicate fires via `output_style.unwrap_or("summary")`; key:val header rendered |
| any | `raw` | no `--output-format json` injection; `render_summary()` not called; raw claude output |

### Referenced Parameter Groups

| # | Group | Membership | Co-members |
|---|-------|------------|------------|
| 2 | [Runner Control](../param_group/02_runner_control.md) | Full | `--dry-run`, `--verbosity`, `--trace`, `--timeout`, `--output-format`, ... |

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 1 | [`run`](../command/01_run.md) | `summary` | Applied in `run_print_mode()` via `output_style.unwrap_or("summary")` |
| 5 | [`ask`](../command/05_ask.md) | `summary` | Pure alias for run; same rendering path |

### See Also

- [`071_summary_fields.md`](071_summary_fields.md) — `--summary-fields` controls which fields appear in summary header (only effective when `--output-style summary`)
- [`061_output_format.md`](061_output_format.md) — `--output-format` passthrough (what claude emits; distinct from rendering)
- [`execution.rs`](../../../../src/cli/execution.rs) — `render_summary()` call site (predicate at ~line 501)
- [`builder.rs`](../../../../src/cli/builder.rs) — `effective_style` injection branch (~line 186)
- [`summary.rs`](../../../../src/cli/summary.rs) — `render_summary()` implementation
