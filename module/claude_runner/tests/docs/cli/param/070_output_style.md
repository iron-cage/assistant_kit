# Parameter :: `--output-style` (run/ask)

Edge case coverage for the `--output-style` parameter on the `run`/`ask` dispatch paths. See [070_output_style.md](../../../../docs/cli/param/070_output_style.md) for specification.

**Scope note:** `--output-style` is a print-mode runner-level rendering parameter. It controls whether `clr` routes captured stdout through `render_summary()` in `summary.rs` (`summary`) or returns raw claude output (`raw`). The parameter has no effect in interactive mode (`run_interactive()`).

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-01 | `clr run -m "x"` (no flags) → stdout contains `╭`; exit 0 | Default |
| EC-02 | `--output-style summary -m "x"` → stdout contains `╭`; exit 0 | Explicit |
| EC-03 | `--output-style raw -m "x"` → stdout does NOT contain `╭`; exit 0 | Behavioral Divergence |
| EC-04 | `CLR_OUTPUT_STYLE=raw` (no flag) → stdout does NOT contain `╭`; exit 0 | Env Var |
| EC-05 | `--output-format text --output-style summary -m "x"` → stdout does NOT contain `╭`; exit 0 | Behavioral Divergence |
| EC-06 | `--output-format json --output-style raw -m "x"` → stdout does NOT contain `╭`; exit 0 | Behavioral Divergence |
| EC-07 | `--output-style invalid` → exit 1; stderr contains `"invalid output-style 'invalid'"` | Validation |
| EC-08 | `--output-format summary -m "x"` (legacy, no `--output-style`) → stdout contains `╭`; exit 0 | Legacy Alias |
| EC-09 | `clr ask -m "x"` (no flags) → stdout contains `╭`; exit 0 | Default |
| EC-10 | `--dry-run --output-style summary -m "x"` → stderr trace contains `"--output-format json"` | Dry-Run |
| EC-11 | `CLR_OUTPUT_STYLE=raw` + `--output-style summary` flag → stdout contains `╭`; flag wins | CLI-wins |
| EC-12 | `CLR_OUTPUT_STYLE=bogus clr run -m "x"` → exit 1; stderr contains `"CLR_OUTPUT_STYLE: invalid value 'bogus'"` | Env Var Validation |
| EC-13 | `--output-format stream-json --output-style summary -m "x"` → stdout does NOT contain `╭`; exit 0 | Behavioral Divergence |

## Test Coverage Summary

- Default: 2 tests (EC-01, EC-09)
- Explicit: 1 test (EC-02)
- Behavioral Divergence: 4 tests (EC-03, EC-05, EC-06, EC-13)
- Env Var: 1 test (EC-04)
- Validation: 1 test (EC-07)
- Legacy Alias: 1 test (EC-08)
- Dry-Run: 1 test (EC-10)
- CLI-wins: 1 test (EC-11)
- Env Var Validation: 1 test (EC-12)

**Total:** 13 test cases

## Architectural Constraint

All 13 tests use a fake `claude` subprocess to avoid live API calls. The fake claude script emits the JSON object:
```json
{"type":"message","content":[{"type":"text","text":"hello"}],"id":"x","role":"assistant","model":"claude-sonnet-4-6","stop_reason":"end_turn","usage":{"input_tokens":1,"output_tokens":1}}
```
Tests assert `╭` presence (stdout.contains("╭")) or absence (!stdout.contains("╭")) to detect whether `render_summary()` fired.

EC-05 and EC-13 verify the graceful fallback path: `--output-format text`/`stream-json` forwarded to claude causes `render_summary()` to receive non-JSON input, return `None`, and fall through to the raw output via `unwrap_or(out)` in `execution.rs`.

EC-08 verifies two independent code paths both fire for `--output-format summary` legacy alias: (1) `builder.rs` translates `"summary"` → `"json"` for the subprocess via the `if let Some(ref fmt)` block; (2) `execution.rs` predicate `output_style.unwrap_or("summary") == "summary"` fires because `output_style` is `None` — triggering `render_summary()`. Both paths are required for the legacy alias to work.

EC-10 dry-run verifies that when `--output-style summary` is set and `--output-format` is absent, `builder.rs` injects `--output-format json` into the subprocess command. The dry-run stderr trace must contain the literal substring `"--output-format json"`.

---

### EC-01: Default output-style is `summary` — stdout contains `╭`

- **Given:** no `--output-style` flag; no `CLR_OUTPUT_STYLE`; fake claude fixture; `-p --max-sessions 0`
- **When:** `clr -p --max-sessions 0 "x"` with fake claude emitting JSON
- **Then:** Exit 0; stdout contains `╭`; `render_summary()` fired because `output_style.unwrap_or("summary") == "summary"` with auto-injected `--output-format json`
- **Exit:** 0
- **Source:** [070_output_style.md](../../../../docs/cli/param/070_output_style.md) Default-summary behavior

---

### EC-02: Explicit `--output-style summary` → stdout contains `╭`

- **Given:** `--output-style summary`; fake claude fixture; `-p --max-sessions 0`
- **When:** `clr -p --max-sessions 0 --output-style summary "x"` with fake claude
- **Then:** Exit 0; stdout contains `╭`; explicit `summary` identical to default path
- **Exit:** 0
- **Source:** [070_output_style.md](../../../../docs/cli/param/070_output_style.md) Default (`summary`)

---

### EC-03: `--output-style raw` → stdout does NOT contain `╭`

- **Given:** `--output-style raw`; fake claude fixture; `-p --max-sessions 0`
- **When:** `clr -p --max-sessions 0 --output-style raw "x"` with fake claude
- **Then:** Exit 0; stdout does NOT contain `╭`; `render_summary()` not called; raw claude output
- **Exit:** 0
- **Source:** [070_output_style.md](../../../../docs/cli/param/070_output_style.md) Values (`raw`)

---

### EC-04: `CLR_OUTPUT_STYLE=raw` → stdout does NOT contain `╭`

- **Given:** `CLR_OUTPUT_STYLE=raw`; no `--output-style` flag; fake claude fixture; `-p --max-sessions 0`
- **When:** `CLR_OUTPUT_STYLE=raw clr -p --max-sessions 0 "x"` with fake claude
- **Then:** Exit 0; stdout does NOT contain `╭`; env var applied when flag absent
- **Exit:** 0
- **Source:** [070_output_style.md](../../../../docs/cli/param/070_output_style.md) Env var (`CLR_OUTPUT_STYLE`)

---

### EC-05: `--output-format text --output-style summary` → stdout does NOT contain `╭`; exit 0

- **Given:** `--output-format text`; `--output-style summary`; fake claude fixture; `-p --max-sessions 0`
- **When:** `clr -p --max-sessions 0 --output-format text --output-style summary "x"` with fake claude
- **Then:** Exit 0; stdout does NOT contain `╭`; `render_summary()` called but receives plain text; returns `None`; raw text passed through via `unwrap_or(out)` at `execution.rs` call site
- **Exit:** 0
- **Source:** [070_output_style.md](../../../../docs/cli/param/070_output_style.md) Combinations table (`text` / `summary` row)

---

### EC-06: `--output-format json --output-style raw` → stdout does NOT contain `╭`

- **Given:** `--output-format json`; `--output-style raw`; fake claude fixture; `-p --max-sessions 0`
- **When:** `clr -p --max-sessions 0 --output-format json --output-style raw "x"` with fake claude
- **Then:** Exit 0; stdout does NOT contain `╭`; `render_summary()` not called despite JSON output; raw JSON from fake claude
- **Exit:** 0
- **Source:** [070_output_style.md](../../../../docs/cli/param/070_output_style.md) Combinations table (`any` / `raw` row)

---

### EC-07: `--output-style invalid` → exit 1; stderr contains validation message

- **Given:** `--output-style invalid`
- **When:** `clr --output-style invalid`
- **Then:** Exit 1; stderr contains `"invalid output-style 'invalid' — expected: summary, raw"`
- **Exit:** 1
- **Source:** [070_output_style.md](../../../../docs/cli/param/070_output_style.md) Validation

---

### EC-08: `--output-format summary` legacy alias → stdout contains `╭`

- **Given:** `--output-format summary`; no `--output-style` flag; fake claude fixture; `-p --max-sessions 0`
- **When:** `clr -p --max-sessions 0 --output-format summary "x"` with fake claude
- **Then:** Exit 0; stdout contains `╭`; two independent code paths both fire: (1) `builder.rs` `if let Some(ref fmt)` block translates `"summary"` → `"json"` and forwards `--output-format json` to claude subprocess; (2) `execution.rs` predicate `output_style.unwrap_or("summary") == "summary"` fires because `output_style` is `None` — triggering `render_summary()` post-processing
- **Exit:** 0
- **Source:** [070_output_style.md](../../../../docs/cli/param/070_output_style.md) Legacy alias

---

### EC-09: `clr ask` default → stdout contains `╭`

- **Given:** no `--output-style` flag; no `CLR_OUTPUT_STYLE`; fake claude fixture; `--max-sessions 0`
- **When:** `clr ask --max-sessions 0 -m "x"` with fake claude
- **Then:** Exit 0; stdout contains `╭`; `ask` uses same `build_claude_command()` path as `run`; `effective_style` defaults to `"summary"`
- **Exit:** 0
- **Source:** [070_output_style.md](../../../../docs/cli/param/070_output_style.md) Referenced Commands (`ask` row)

---

### EC-10: `--dry-run --output-style summary` → dry-run trace contains `--output-format json`

- **Given:** `--dry-run`; `--output-style summary`; `-p --max-sessions 0`
- **When:** `clr -p --dry-run --max-sessions 0 --output-style summary "x"`
- **Then:** Exit 0 (dry-run); stderr trace contains the literal substring `"--output-format json"`; `builder.rs` injection branch fires because `use_print && effective_style == "summary" && cli.output_format.is_none()`; no real subprocess launched
- **Exit:** 0
- **Source:** [070_output_style.md](../../../../docs/cli/param/070_output_style.md) `--output-format json` auto-injection

---

### EC-11: `CLR_OUTPUT_STYLE=raw` + `--output-style summary` flag → flag wins; stdout contains `╭`

- **Given:** `CLR_OUTPUT_STYLE=raw`; `--output-style summary` flag; fake claude fixture; `-p --max-sessions 0`
- **When:** `CLR_OUTPUT_STYLE=raw clr -p --max-sessions 0 --output-style summary "x"` with fake claude
- **Then:** Exit 0; stdout contains `╭`; CLI flag wins over env var (standard precedence)
- **Exit:** 0
- **Source:** [070_output_style.md](../../../../docs/cli/param/070_output_style.md) Env var (CLI flag wins when both set)

---

### EC-12: `CLR_OUTPUT_STYLE=bogus` → exit 1; stderr contains validation message

- **Given:** `CLR_OUTPUT_STYLE=bogus`; no `--output-style` flag
- **When:** `CLR_OUTPUT_STYLE=bogus clr run -m "x"`
- **Then:** Exit 1; stderr contains `"CLR_OUTPUT_STYLE: invalid value 'bogus' — expected: summary, raw"`
- **Exit:** 1
- **Source:** [070_output_style.md](../../../../docs/cli/param/070_output_style.md) Env var validation

---

### EC-13: `--output-format stream-json --output-style summary` → stdout does NOT contain `╭`; exit 0

- **Given:** `--output-format stream-json`; `--output-style summary`; fake claude fixture; `-p --max-sessions 0`
- **When:** `clr -p --max-sessions 0 --output-format stream-json --output-style summary "x"` with fake claude
- **Then:** Exit 0; stdout does NOT contain `╭`; `stream-json` forwarded verbatim; `render_summary()` receives non-JSON stream; returns `None`; raw stream passed through via `unwrap_or(out)` — same fallback as EC-05
- **Exit:** 0
- **Source:** [070_output_style.md](../../../../docs/cli/param/070_output_style.md) Combinations table (`stream-json` / `summary` row)
