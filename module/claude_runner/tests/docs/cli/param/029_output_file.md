# Parameter :: `--output-file`

Edge case coverage for the `--output-file` parameter. See [029_output_file.md](../../../../docs/cli/param/029_output_file.md) for specification.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `--output-file <path>` → file created with stdout content | Behavioral Divergence |
| EC-2 | Default (no `--output-file`) → no file created | Behavioral Divergence |
| EC-3 | Non-writable path → exit 1, error to stderr | Error Handling |
| EC-4 | `--output-file` + `--strip-fences` → stripped text in file AND stdout | Interaction |
| EC-5 | `--output-file` + `--dry-run` → no file created | Edge Case |
| EC-6 | `--help` output contains `--output-file` | Documentation |

## Test Coverage Summary

- Behavioral Divergence: 2 tests (EC-1, EC-2)
- Error Handling: 1 test (EC-3)
- Interaction: 1 test (EC-4)
- Edge Case: 1 test (EC-5)
- Documentation: 1 test (EC-6)

**Total:** 6 edge cases

## Implementation Notes

| EC | Test Function | File |
|----|---------------|------|
| EC-1 | `ec1_tee_file_content_equals_stdout` | `output_file_test.rs` (uses fake claude, `cfg(unix)`) |
| EC-2 | `ec2_no_output_file_no_artifact` | `output_file_test.rs` |
| EC-3 | `ec3_nonwritable_path_exits_1` | `output_file_test.rs` (uses fake claude, `cfg(unix)`) |
| EC-4 | `ec4_strip_fences_and_output_file_same_content` | `output_file_test.rs` |
| EC-5 | `ec5_dry_run_skips_output_file_write` | `output_file_test.rs` |
| EC-6 | `ec6_help_lists_output_file`, `ec6b_ask_help_lists_output_file` | `output_file_test.rs` |

---

### EC-1: File created with stdout content

- **Given:** a writable temp file path; a dry-run message so subprocess is not spawned
- **When:** `clr --output-file <path> --dry-run "task"` ... but for real output: `clr -p --output-file <path> "Repeat: hello"`
- **Then:** `<path>` exists and contains the captured stdout; stdout also printed
- **Exit:** 0
- **Source:** [029_output_file.md](../../../../docs/cli/param/029_output_file.md)
- **Commands:** run, ask

---

### EC-2: Default — no file created

- **Given:** clean environment; no `--output-file`
- **When:** `clr --dry-run "task"`
- **Then:** no file artifact on disk; stdout goes to terminal
- **Exit:** 0
- **Source:** [029_output_file.md](../../../../docs/cli/param/029_output_file.md)
- **Commands:** run, ask

---

### EC-3: Non-writable path → error

- **Given:** a path whose parent directory does not exist
- **When:** `clr -p --output-file /nonexistent_dir/out.txt "task"`
- **Then:** Exit 1; stderr contains the file path and an OS error
- **Exit:** 1
- **Source:** [029_output_file.md](../../../../docs/cli/param/029_output_file.md)
- **Commands:** run, ask

---

### EC-4: `--output-file` + `--strip-fences` → same stripped content in both destinations

- **Given:** a writable temp path; claude returns a fenced code block
- **When:** `clr -p --strip-fences --output-file <path> "task"`
- **Then:** file content equals stdout content; neither contains opening/closing fence markers
- **Exit:** 0
- **Source:** [029_output_file.md](../../../../docs/cli/param/029_output_file.md), [--strip-fences](../../../../docs/cli/param/026_strip_fences.md)
- **Commands:** run, ask

---

### EC-5: `--output-file` + `--dry-run` → no file created

- **Given:** a path that should NOT be created
- **When:** `clr --dry-run --output-file /tmp/should_not_exist_99999.txt "task"`
- **Then:** Exit 0; the file does NOT exist at the given path
- **Exit:** 0
- **Source:** [029_output_file.md](../../../../docs/cli/param/029_output_file.md)
- **Commands:** run, ask

---

### EC-6: `--help` lists `--output-file`

- **Given:** clean environment
- **When:** `clr --help`
- **Then:** Stdout contains `--output-file`
- **Exit:** 0
- **Source:** [command/02_help.md](../../../../docs/cli/command/02_help.md)
- **Commands:** run, ask
