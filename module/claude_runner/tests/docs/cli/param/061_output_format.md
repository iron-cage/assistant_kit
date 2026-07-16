# Parameter :: `--output-format`

Edge case tests for the output format parameter. Tests validate value forwarding, missing-value rejection, valid enum values, and help documentation.

**Source:** [061_output_format.md](../../../../docs/cli/param/061_output_format.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `--output-format json` → flag forwarded to claude | Behavioral Divergence |
| EC-2 | `--output-format` without value → exit 1 | Missing Value |
| EC-3 | `--output-format` at end of argv → exit 1 | Boundary Values |
| EC-4 | `--output-format text` → forwarded | Behavioral Divergence |
| EC-5 | `--output-format stream-json` → forwarded | Behavioral Divergence |
| EC-6 | `--help` lists `--output-format` | Documentation |
| EC-7 | Without `--output-format` → runner auto-injects `--output-format json` in summary mode | Behavioral Divergence |
| EC-8 | `CLR_OUTPUT_FORMAT=json` env var → forwarded | Env Var |
| EC-9 | `--output-format summary` dry-run → `--output-format json` in assembled command | Summary Variant |
| EC-10 | `--output-format summary` with fake claude CLR envelope → stdout contains summary header | Summary Variant |
| EC-11 | `--output-format summary` with fake claude CLR envelope → `result` value after `---` separator | Summary Variant |
| EC-12 | `CLR_OUTPUT_FORMAT=summary` env var → `--output-format json` in assembled command | Summary Variant |
| EC-13 | CLR envelope with `is_error: true` → error status visible in summary header | Summary Variant |
| EC-14 | Claude exits non-zero with `summary` → raw output preserved, no JSON parse | Summary Variant |
| IT-4 | `--output-format stream-json` → NDJSON events consumed live, before subprocess exit | Streaming Behavior |
| IT-5 | `--output-format stream-json` → NDJSON events observed in emission order | Streaming Behavior |
| IT-6 | Without `stream-json` → output still delivered via unchanged batched path | Streaming Behavior |

## Test Coverage Summary

- Behavioral Divergence: 4 tests (EC-1, EC-4, EC-5, EC-7)
- Missing Value: 1 test (EC-2)
- Boundary Values: 1 test (EC-3)
- Documentation: 1 test (EC-6)
- Env Var: 1 test (EC-8)
- Summary Variant: 6 tests (EC-9, EC-10, EC-11, EC-12, EC-13, EC-14)
- Streaming Behavior: 3 tests (IT-4, IT-5, IT-6)

**Total:** 17 edge cases

## Test Cases
---

### EC-1: `--output-format json` → forwarded to assembled command

- **Given:** clean environment
- **When:** `clr --dry-run --output-format json "Fix bug"`
- **Then:** Assembled command contains `--output-format json`
- **Exit:** 0
- **Source:** [061_output_format.md](../../../../docs/cli/param/061_output_format.md)
- **Commands:** run, ask
---

### EC-2: `--output-format` without value → exit 1

- **Given:** clean environment
- **When:** `clr --output-format`
- **Then:** Exit 1; error about missing `--output-format` value
- **Exit:** 1
- **Source:** [061_output_format.md](../../../../docs/cli/param/061_output_format.md)
- **Commands:** run, ask
---

### EC-3: `--output-format` at end of argv → exit 1

- **Given:** clean environment
- **When:** `clr "Fix bug" --output-format`
- **Then:** Exit 1; error about missing `--output-format` value
- **Exit:** 1
- **Source:** [061_output_format.md](../../../../docs/cli/param/061_output_format.md)
- **Commands:** run, ask
---

### EC-4: `--output-format text` → forwarded

- **Given:** clean environment
- **When:** `clr --dry-run --output-format text "Fix bug"`
- **Then:** Assembled command contains `--output-format text`
- **Exit:** 0
- **Source:** [061_output_format.md](../../../../docs/cli/param/061_output_format.md)
- **Commands:** run, ask
---

### EC-5: `--output-format stream-json` → forwarded

- **Given:** clean environment
- **When:** `clr --dry-run --output-format stream-json "Fix bug"`
- **Then:** Assembled command contains `--output-format stream-json`
- **Exit:** 0
- **Source:** [061_output_format.md](../../../../docs/cli/param/061_output_format.md)
- **Commands:** run, ask
---

### EC-6: `--help` lists `--output-format`

- **Given:** clean environment
- **When:** `clr --help`
- **Then:** Stdout contains `--output-format`
- **Exit:** 0
- **Source:** [command/02_help.md](../../../../docs/cli/command/02_help.md)
- **Commands:** run, ask
---

### EC-7: Without `--output-format` → runner auto-injects `--output-format json` in summary mode

- **Given:** clean environment; no `--output-format` flag; no `CLR_OUTPUT_FORMAT` env var; default `--output-style summary`
- **When:** `clr --dry-run "Fix bug"`
- **Then:** Assembled command DOES contain `--output-format json` (auto-injected by Path B in `builder.rs` when `use_print && effective_style == "summary" && output_format.is_none()`)
- **Exit:** 0
- **Source:** [061_output_format.md](../../../../docs/cli/param/061_output_format.md)
- **Commands:** run, ask
---

### EC-8: `CLR_OUTPUT_FORMAT=json` env var → forwarded

- **Given:** `CLR_OUTPUT_FORMAT=json`
- **When:** `clr --dry-run "Fix bug"`
- **Then:** Assembled command contains `--output-format json`
- **Exit:** 0
- **Source:** [061_output_format.md](../../../../docs/cli/param/061_output_format.md)
- **Commands:** run, ask
---

### EC-9: `--output-format summary` dry-run → `--output-format json` in assembled command

- **Given:** clean environment
- **When:** `clr --dry-run --output-format summary "Fix bug"`
- **Then:** Assembled command contains `--output-format json` (NOT `summary`); `summary` is intercepted by runner
- **Exit:** 0
- **Source:** [061_output_format.md](../../../../docs/cli/param/061_output_format.md)
- **Commands:** run, ask
---

### EC-10: `--output-format summary` with fake claude CLR envelope → summary header in stdout

- **Given:** Fake claude binary that outputs `{"type":"result","subtype":"success","session_id":"00000000-0000-0000-0000-000000000001","is_error":false,"result":"hello","usage":{"input_tokens":1,"output_tokens":1},"total_cost_usd":0.0}`
- **When:** `clr --output-format summary "msg"`
- **Then:** Stdout contains `session_id:` and `input_tokens:` and `is_error:` (summary header); stdout contains `---` separator
- **Exit:** 0
- **Source:** [061_output_format.md](../../../../docs/cli/param/061_output_format.md)
- **Commands:** run, ask
---

### EC-11: `--output-format summary` with fake claude CLR envelope → `result` value after separator

- **Given:** Same fake claude binary as EC-10
- **When:** `clr --output-format summary "msg"`
- **Then:** Stdout contains `hello` (the `result` field value) after the `---` separator
- **Exit:** 0
- **Source:** [061_output_format.md](../../../../docs/cli/param/061_output_format.md)
- **Commands:** run, ask
---

### EC-12: `CLR_OUTPUT_FORMAT=summary` env var → `--output-format json` in assembled command

- **Given:** `CLR_OUTPUT_FORMAT=summary`
- **When:** `clr --dry-run "Fix bug"`
- **Then:** Assembled command contains `--output-format json` (NOT `summary`)
- **Exit:** 0
- **Source:** [061_output_format.md](../../../../docs/cli/param/061_output_format.md)
- **Commands:** run, ask
---

### EC-13: CLR envelope with `is_error: true` → error status visible in summary header

- **Given:** Fake claude binary that outputs `{"type":"result","subtype":"error","session_id":"00000000-0000-0000-0000-000000000002","is_error":true,"result":"Something went wrong","usage":{"input_tokens":2,"output_tokens":0},"total_cost_usd":0.0}`
- **When:** `clr --output-format summary "msg"`
- **Then:** Summary header contains `is_error:` and `subtype:`; `Something went wrong` appears after the `---` separator
- **Exit:** 0
- **Source:** [061_output_format.md](../../../../docs/cli/param/061_output_format.md)
- **Commands:** run, ask
---

### EC-14: Claude exits non-zero with `summary` → raw output preserved, no JSON parse

- **Given:** Fake claude binary that exits with code 2 and writes `Error: rate limit` to stderr
- **When:** `clr --output-format summary "msg"`
- **Then:** `Error: rate limit` appears in stderr; no summary header in stdout; no JSON parse error in output
- **Exit:** 2
- **Source:** [061_output_format.md](../../../../docs/cli/param/061_output_format.md)
- **Commands:** run, ask
---

### IT-4: `--output-format stream-json` → NDJSON events consumed live, before subprocess exit

- **Given:** Fake claude binary emitting 3 NDJSON lines (`seq:1`, `seq:2`, `seq:3`) separated by `sleep 2` delays (~4s total runtime)
- **When:** `clr --output-format stream-json --max-sessions 0 "hi"` (stdout piped, read incrementally)
- **Then:** First event arrives within 1800ms, second within a further 2200ms — both well before the fake process's ~4s total runtime; `child.try_wait()` returns `None` (still running) immediately after the second event is received, proving incremental (not batched) consumption
- **Exit:** 0
- **Source:** [061_output_format.md](../../../../docs/cli/param/061_output_format.md)
- **Commands:** run, ask
---

### IT-5: `--output-format stream-json` → NDJSON events observed in emission order

- **Given:** Fake claude binary emitting 3 NDJSON lines (`seq:1`, `seq:2`, `seq:3`) separated by `sleep 1` delays
- **When:** `clr --output-format stream-json --max-sessions 0 "hi"` (stdout read live, one event at a time, each checked against its own arrival deadline)
- **Then:** Events are observed in strict emission order (`seq:1` then `seq:2` then `seq:3`); each arrives within its own deadline rather than only after full subprocess completion
- **Exit:** 0
- **Source:** [061_output_format.md](../../../../docs/cli/param/061_output_format.md)
- **Commands:** run, ask
---

### IT-6: Without `stream-json` → output still delivered via unchanged batched path

- **Given:** Fake claude binary that echoes `plain claude output`
- **When:** `clr --output-format json --output-style raw --max-sessions 0 "hi"`
- **Then:** Stdout is exactly `plain claude output` — regression guard confirming the pre-existing batched path is unchanged for non-`stream-json` output formats
- **Exit:** 0
- **Source:** [061_output_format.md](../../../../docs/cli/param/061_output_format.md)
- **Commands:** run, ask
