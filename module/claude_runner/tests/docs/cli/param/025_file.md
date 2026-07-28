# Parameter :: `--file`

Edge case coverage for the `--file` parameter. See [025_file.md](../../../../docs/cli/param/025_file.md) for specification.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `--file <path>` → file content piped to subprocess stdin | Behavioral Divergence |
| EC-2 | Default (no `--file`, stdin not piped) → subprocess receives no stdin | Behavioral Divergence |
| EC-3 | Non-existent file path → error exit with descriptive message | Edge Case |
| EC-4 | `--help` output contains `--file` | Documentation |
| EC-5 | `--file` + `--model` → both applied, no conflict | Interaction |
| EC-6 | `--file` with `--dry-run` → no file opened; path in describe output | Edge Case |
| EC-7 | No `--file`, non-JSON content piped to stdin → forwarded verbatim to subprocess stdin | Behavioral Divergence |

## Test Coverage Summary

- Behavioral Divergence: 3 tests (EC-1, EC-2, EC-7)
- Edge Case: 2 tests (EC-3, EC-6)
- Interaction: 1 test (EC-5)
- Documentation: 1 test (EC-4)

**Total:** 7 edge cases

---

### EC-1: File content piped to stdin

- **Given:** a temp file exists at a known path containing "hello from file"
- **When:** `clr -p --file <path> "Repeat stdin verbatim"`
- **Then:** Claude subprocess receives the file content on stdin
- **Exit:** 0
- **Source:** [--file](../../../../docs/cli/param/025_file.md)
- **Commands:** run, ask

---

### EC-2: Default, stdin not piped → no stdin

- **Given:** clean environment; no `--file` flag; stdin not piped (or piped empty)
- **When:** `clr --dry-run "task"`
- **Then:** Assembled command does NOT contain `--file` references; describe output shows no stdin_file
- **Exit:** 0
- **Source:** [--file](../../../../docs/cli/param/025_file.md)
- **Commands:** run, ask

---

### EC-3: Non-existent file → error

- **Given:** a path that does not exist on the filesystem
- **When:** `clr --file /tmp/nonexistent_99999.txt "task"`
- **Then:** Exit 1; stderr contains the file path and an error message
- **Exit:** 1
- **Source:** [--file](../../../../docs/cli/param/025_file.md)
- **Commands:** run, ask

---

### EC-4: `--help` lists `--file`

- **Given:** clean environment
- **When:** `clr --help`
- **Then:** Stdout contains `--file`
- **Exit:** 0
- **Source:** [command/02_help.md](../../../../docs/cli/command/02_help.md)
- **Commands:** run, ask

---

### EC-5: `--file` + `--model` → both applied

- **Given:** a temp file exists; clean environment
- **When:** `clr --dry-run --file /tmp/x.txt --model sonnet "task"`
- **Then:** Assembled command contains `--model sonnet`; describe output includes file path
- **Exit:** 0
- **Source:** [--file](../../../../docs/cli/param/025_file.md)
- **Commands:** run, ask

---

### EC-6: `--file` + `--dry-run` → no file open

- **Given:** a non-existent path (to verify the file is NOT opened)
- **When:** `clr --dry-run --file /tmp/nonexistent_99999.txt "task"`
- **Then:** Exit 0 (dry-run short-circuits before file open); describe output includes the path
- **Exit:** 0
- **Source:** [--file](../../../../docs/cli/param/025_file.md)
- **Commands:** run, ask

---

### EC-7: No `--file`, non-JSON piped content → forwarded to stdin

- **Given:** no `--file` flag; stdin piped with non-empty content not starting with `{` (e.g. `"hello from pipe"`)
- **When:** `some_tool | clr -p "Repeat stdin verbatim"`
- **Then:** Claude subprocess receives the piped content on stdin, byte-for-byte
- **Exit:** 0
- **Source:** [--file](../../../../docs/cli/param/025_file.md), [feature/004_json_config.md](../../../../docs/feature/004_json_config.md)
- **Commands:** run, ask
- **Regression coverage:** `bug_reproducer_424_plain_pipe_content_forwarded_to_stdin` in `execution_mode_test.rs` (BUG-424)
