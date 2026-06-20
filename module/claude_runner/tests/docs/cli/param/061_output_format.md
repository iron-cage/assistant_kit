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
| EC-7 | Without `--output-format` → no `--output-format` flag in assembled command | Behavioral Divergence |
| EC-8 | `CLR_OUTPUT_FORMAT=json` env var → forwarded | Env Var |
| EC-9 | `--output-format summary` dry-run → `--output-format json` in assembled command | Summary Variant |
| EC-10 | `--output-format summary` with fake claude JSON → stdout contains YAML header box | Summary Variant |
| EC-11 | `--output-format summary` with fake claude JSON → stdout contains text body after separator | Summary Variant |
| EC-12 | `CLR_OUTPUT_FORMAT=summary` env var → `--output-format json` in assembled command | Summary Variant |
| EC-13 | Multi-block JSON (thinking+tool_use+text) → topology shows all block types | Summary Variant |
| EC-14 | Claude exits non-zero with `summary` → raw output preserved, no JSON parse | Summary Variant |

## Test Coverage Summary

- Behavioral Divergence: 4 tests (EC-1, EC-4, EC-5, EC-7)
- Missing Value: 1 test (EC-2)
- Boundary Values: 1 test (EC-3)
- Documentation: 1 test (EC-6)
- Env Var: 1 test (EC-8)
- Summary Variant: 6 tests (EC-9, EC-10, EC-11, EC-12, EC-13, EC-14)

**Total:** 14 edge cases

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
- **Source:** [command/04_help.md](../../../../docs/cli/command/04_help.md)
- **Commands:** run, ask
---

### EC-7: Without `--output-format` → no `--output-format` flag in assembled command

- **Given:** clean environment
- **When:** `clr --dry-run "Fix bug"`
- **Then:** Assembled command does NOT contain `--output-format`
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

### EC-10: `--output-format summary` with fake claude JSON → YAML header box in stdout

- **Given:** Fake claude binary that outputs `{"id":"msg_01","type":"message","role":"assistant","content":[{"type":"text","text":"hello"}],"model":"test","stop_reason":"end_turn","stop_sequence":null,"usage":{"input_tokens":1,"output_tokens":1}}`
- **When:** `clr --output-format summary "msg"`
- **Then:** Stdout contains `model:` and `usage:` and `content:` (YAML header); stdout contains box-drawing characters
- **Exit:** 0
- **Source:** [061_output_format.md](../../../../docs/cli/param/061_output_format.md)
- **Commands:** run, ask
---

### EC-11: `--output-format summary` with fake claude JSON → text body after separator

- **Given:** Same fake claude binary as EC-10
- **When:** `clr --output-format summary "msg"`
- **Then:** Stdout contains `hello` (extracted text block content) after the YAML header section
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

### EC-13: Multi-block JSON → topology shows all block types in header

- **Given:** Fake claude binary that outputs a 3-block JSON response: `{"id":"msg_01","type":"message","role":"assistant","content":[{"type":"thinking","thinking":"...","signature":"sig"},{"type":"tool_use","id":"toolu_01","name":"Read","input":{"file_path":"/tmp/f"}},{"type":"text","text":"The result"}],"model":"test","stop_reason":"end_turn","stop_sequence":null,"usage":{"input_tokens":5,"output_tokens":10}}`
- **When:** `clr --output-format summary "msg"`
- **Then:** stdout YAML header contains `thinking`, `tool_use`, and `text` block entries with field topology; `"Read"` tool name appears; `{file_path}` input keys appear; `The result` appears in text body
- **Exit:** 0
- **Source:** [061_output_format.md](../../../../docs/cli/param/061_output_format.md)
- **Commands:** run, ask
---

### EC-14: Claude exits non-zero with `summary` → raw output preserved, no JSON parse

- **Given:** Fake claude binary that exits with code 2 and writes `Error: rate limit` to stderr
- **When:** `clr --output-format summary "msg"`
- **Then:** `Error: rate limit` appears in stderr; no YAML header in stdout; no JSON parse error in output
- **Exit:** 2
- **Source:** [061_output_format.md](../../../../docs/cli/param/061_output_format.md)
- **Commands:** run, ask
