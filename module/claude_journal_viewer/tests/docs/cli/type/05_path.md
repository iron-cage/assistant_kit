# Type :: `Path`

Validation tests for the `Path` semantic type. Tests validate tilde
expansion, existence checks for read/write paths, and the exception for
substring-filter paths.

**Source:** [type/05_path.md](../../../../docs/cli/type/05_path.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| TC-1 | Absolute path accepted | Parsing |
| TC-2 | `~/...` expanded to `$HOME/...` | Tilde Expansion |
| TC-3 | Non-existent `journal_dir` (read path) -> exit 1 | Error Handling |
| TC-4 | Non-writable `output` (write path) -> exit 1 | Error Handling |
| TC-5 | `dir` filter accepts a non-existent substring (not existence-checked) | Exception |

## Test Coverage Summary

- Parsing: 1 test (TC-1)
- Tilde Expansion: 1 test (TC-2)
- Error Handling: 2 tests (TC-3, TC-4)
- Exception: 1 test (TC-5)

**Total:** 5 test cases

## Test Cases

---

### TC-1: Absolute path accepted

- **Given:** `/tmp/existing_journal` exists and contains journal files
- **When:** `clj .list journal_dir::/tmp/existing_journal`
- **Then:** exit 0; events are read from the given absolute path
- **Exit:** 0
- **Source:** [type/05_path.md](../../../../docs/cli/type/05_path.md)

---

### TC-2: `~/...` expanded to `$HOME/...`

- **Given:** `$HOME/.clr/journal` exists and contains journal files
- **When:** `clj .list journal_dir::~/.clr/journal`
- **Then:** exit 0; `~` is resolved to the `$HOME` value before the directory is opened
- **Exit:** 0
- **Source:** [type/05_path.md](../../../../docs/cli/type/05_path.md)

---

### TC-3: Non-existent `journal_dir` (read path) -> exit 1

- **Given:** `/tmp/does_not_exist_journal` does not exist
- **When:** `clj .list journal_dir::/tmp/does_not_exist_journal`
- **Then:** exit 1; stderr contains `journal directory '/tmp/does_not_exist_journal' does not exist`
- **Exit:** 1
- **Source:** [type/05_path.md](../../../../docs/cli/type/05_path.md), [param/21_journal_dir.md](../../../../docs/cli/param/21_journal_dir.md)

---

### TC-4: Non-writable `output` (write path) -> exit 1

- **Given:** `/nonexistent_parent_dir/out.csv` whose parent directory does not exist
- **When:** `clj .export format::csv output::/nonexistent_parent_dir/out.csv`
- **Then:** exit 1; stderr contains `cannot write to '/nonexistent_parent_dir/out.csv'`
- **Exit:** 1
- **Source:** [type/05_path.md](../../../../docs/cli/type/05_path.md), [param/23_output.md](../../../../docs/cli/param/23_output.md)

---

### TC-5: `dir` filter accepts a non-existent substring (not existence-checked)

- **Given:** journal with events carrying working-directory values
- **When:** `clj .list dir::/this/path/was/never/created`
- **Then:** exit 0; no existence check is performed on the `dir` filter value, since it is a substring match, not a read path
- **Exit:** 0
- **Source:** [type/05_path.md](../../../../docs/cli/type/05_path.md), [param/07_dir.md](../../../../docs/cli/param/07_dir.md)
