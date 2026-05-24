# Type :: `FilePath`

Validation tests for the `FilePath` semantic type (String: filesystem path to a readable
file whose content is piped as stdin to the `claude` subprocess). Tests verify that the
runner opens and pipes the file at spawn time; file-not-found and unreadable-file errors
are raised by the runner, not deferred to the subprocess.

**Source:** [type/12_file_path.md](../../../../docs/cli/type/12_file_path.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| TC-1 | Absolute path to existing file → accepted and piped | Valid Input |
| TC-2 | Relative path to existing file → accepted and piped | Valid Input |
| TC-3 | Path with spaces → accepted and piped | Valid Input |
| TC-4 | Non-existent path → runner error before subprocess launch | Error Case |
| TC-5 | Path resolved relative to caller cwd (not `--dir`) | Edge Case |

## Test Coverage Summary

- Valid Input: 3 tests (TC-1, TC-2, TC-3)
- Error Case: 1 test (TC-4)
- Edge Case: 1 test (TC-5)

**Total:** 5 test cases

## Test Cases

---

### TC-1: Absolute path to existing file → accepted and piped

- **Given:** `/tmp/test_input.txt` exists and is readable
- **When:** `clr --file /tmp/test_input.txt --dry-run "Summarise"`
- **Then:** Exit 0; assembled command does not contain `--file` (it is a runner-consumed param); no error about the path
- **Exit:** 0
- **Source:** [type/12_file_path.md](../../../../docs/cli/type/12_file_path.md)

---

### TC-2: Relative path to existing file → accepted and piped

- **Given:** `./notes.md` exists in the current working directory
- **When:** `clr --file ./notes.md --dry-run "Summarise"`
- **Then:** Exit 0; no error; `--file` is consumed by the runner, not forwarded to subprocess
- **Exit:** 0
- **Source:** [type/12_file_path.md](../../../../docs/cli/type/12_file_path.md)

---

### TC-3: Path with spaces → accepted and piped

- **Given:** `/tmp/my notes.txt` exists and is readable
- **When:** `clr --file "/tmp/my notes.txt" --dry-run "Summarise"`
- **Then:** Exit 0; path parsed as a single token; no truncation at space boundary
- **Exit:** 0
- **Source:** [type/12_file_path.md](../../../../docs/cli/type/12_file_path.md)

---

### TC-4: Non-existent path → runner error before subprocess launch

- **Given:** `/tmp/no_such_file_xyz.txt` does not exist
- **When:** `clr --file /tmp/no_such_file_xyz.txt "Summarise"`
- **Then:** Exit 1; error message includes the path and OS error (e.g. "No such file or directory"); subprocess is never launched
- **Exit:** 1
- **Source:** [type/12_file_path.md](../../../../docs/cli/type/12_file_path.md)

---

### TC-5: Path resolved relative to caller cwd, not `--dir`

- **Given:** `./context.md` exists in the caller's cwd; `--dir /other/project` targets a different directory
- **When:** `clr --file ./context.md --dir /other/project --dry-run "task"`
- **Then:** Exit 0; file resolved against caller's cwd (not `/other/project`); `--dir` affects subprocess working directory only
- **Exit:** 0
- **Source:** [type/12_file_path.md](../../../../docs/cli/type/12_file_path.md)
