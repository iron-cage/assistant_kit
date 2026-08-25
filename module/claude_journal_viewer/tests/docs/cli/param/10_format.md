# Parameter :: `format`

Edge case tests for the `format` parameter. Tests validate the
per-command default variance between `.list`/`.tail` and `.export`.

**Source:** [param/10_format.md](../../../../docs/cli/param/10_format.md)

## Test Case Index

| ID | Test Name | Category | Status | Implemented as |
|----|-----------|----------|--------|----------------|
| EC-1 | Absent on `.list` -> defaults to table | Per-Command Default | ✅ | `viewer_integration_test.rs::ec1_list_prints_table` |
| EC-2 | Absent on `.export` -> defaults to json | Per-Command Default | ⏳ | — |
| EC-3 | `format::csv` with `output` on `.export` -> CSV file written | Parsing | ✅ | `viewer_integration_test.rs::ec32_list_non_table_formats_match_export` |
| EC-4 | `.list format::X` is byte-identical to `.export format::X` | No Drift | ✅ | `viewer_integration_test.rs::ec32_list_non_table_formats_match_export` |
| EC-5 | `.tail format::json` emits objects per line, not an array | Streaming | ✅ | `viewer_integration_test.rs::ec34_tail_format_renders_and_rejects_before_blocking` |
| EC-6 | `.tail format::bogus` exits 1 before blocking | Error Handling | ✅ | `viewer_integration_test.rs::ec34_tail_format_renders_and_rejects_before_blocking` |

## Test Coverage Summary

- Per-Command Default: 2 tests (EC-1, EC-2)
- Parsing: 1 test (EC-3)
- No Drift: 1 test (EC-4)
- Streaming: 1 test (EC-5)
- Error Handling: 1 test (EC-6)

**Total:** 6 edge cases (5 executable)

EC-4 is what keeps `.list`'s delegation to `build_export_content` honest. Two
independent renderers for the same format name would drift, and the drift would
only ever be visible on whichever surface the reader was *not* using.

## Test Cases

---

### EC-1: Absent on `.list` -> defaults to table

- **Given:** journal with events
- **When:** `clj .list`
- **Then:** exit 0; output is rendered as an aligned table
- **Exit:** 0
- **Source:** [param/10_format.md](../../../../docs/cli/param/10_format.md)

---

### EC-2: Absent on `.export` -> defaults to json

- **Given:** journal with events
- **When:** `clj .export output::<path>`
- **Then:** exit 0; the file holds a JSON array
- **Exit:** 0
- **Source:** [param/10_format.md](../../../../docs/cli/param/10_format.md), [command/08_export.md](../../../../docs/cli/command/08_export.md)

---

### EC-3: `format::csv` with `output` on `.export` -> CSV file written

- **Given:** journal with events; the output file's parent directory exists
- **When:** `clj .export format::csv output::<path>`
- **Then:** exit 0; the file is written with a header row and comma-separated values
- **Exit:** 0
- **Source:** [param/10_format.md](../../../../docs/cli/param/10_format.md)

---

### EC-4: `.list format::X` is byte-identical to `.export format::X`

- **Given:** journal with events, and the same filter applied to both commands
- **When:** `clj .list format::X` and `clj .export format::X output::<path>` for X in json, jsonl, csv
- **Then:** exit 0 for both; `.list` stdout equals the exported file's contents
- **Exit:** 0
- **Source:** [param/10_format.md](../../../../docs/cli/param/10_format.md), [command/01_list.md](../../../../docs/cli/command/01_list.md)

---

### EC-5: `.tail format::json` emits objects per line, not an array

- **Given:** journal with events already written
- **When:** `clj .tail format::json`
- **Then:** the first stdout line parses on its own as a complete JSON object
- **And:** it is not `[`, which would open an array whose closing bracket a
  never-ending stream can never write
- **Exit:** killed by the caller — `.tail` does not exit on its own
- **Source:** [param/10_format.md](../../../../docs/cli/param/10_format.md), [command/02_tail.md](../../../../docs/cli/command/02_tail.md)

---

### EC-6: `.tail format::bogus` exits 1 before blocking

- **Given:** clean environment
- **When:** `clj .tail format::bogus`
- **Then:** exit 1 promptly, naming the offending value — not after an
  indefinite wait for an event that may never arrive
- **Exit:** 1
- **Source:** [param/10_format.md](../../../../docs/cli/param/10_format.md), [command/02_tail.md](../../../../docs/cli/command/02_tail.md)
