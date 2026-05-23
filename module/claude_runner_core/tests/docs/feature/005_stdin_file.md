# Feature :: Stdin File Piping

Behavioral requirement cases for the `stdin_file` field on `ClaudeCommand`. See
[feature/005_stdin_file.md](../../../docs/feature/005_stdin_file.md) for the specification.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| FT-1 | `with_stdin_file(path)` → path appears in dry-run describe output | Behavioral Divergence |
| FT-2 | No `stdin_file` → describe output contains no stdin reference | Behavioral Divergence |
| FT-3 | Nonexistent file path → `execute()` returns `Err` with path in message | Error Path |
| FT-4 | `with_dry_run(true)` + nonexistent path → `execute()` returns `Ok` (file not opened) | Dry-Run Interaction |
| FT-5 | `with_stdin_file(a).with_stdin_file(b)` → describe shows `b`, not `a` (last-write wins) | Override Semantics |

## Test Coverage Summary

- Behavioral Divergence: 2 tests (FT-1, FT-2)
- Error Path: 1 test (FT-3)
- Dry-Run Interaction: 1 test (FT-4)
- Override Semantics: 1 test (FT-5)

**Total:** 5 feature cases

---

### FT-1: stdin_file in describe output

- **Given:** a temp file at a known path; `ClaudeCommand::new().with_stdin_file(path).with_dry_run(true)`
- **When:** `execute()` is called
- **Then:** Returns `Ok`; the describe output string contains `"< "` followed by the file path
- **Source:** [feature/005_stdin_file.md](../../../docs/feature/005_stdin_file.md)

---

### FT-2: No stdin_file → no stdin reference in describe

- **Given:** `ClaudeCommand::new().with_dry_run(true)` (no `with_stdin_file` call)
- **When:** `execute()` is called
- **Then:** Returns `Ok`; the describe output does NOT contain `"< "`
- **Source:** [feature/005_stdin_file.md](../../../docs/feature/005_stdin_file.md)

---

### FT-3: Nonexistent file → Err with path

- **Given:** a path that does not exist; `ClaudeCommand::new().with_stdin_file(nonexistent_path)` (no dry-run)
- **When:** `execute()` is called
- **Then:** Returns `Err`; the error message contains the file path string
- **Source:** [feature/005_stdin_file.md](../../../docs/feature/005_stdin_file.md)

---

### FT-4: dry_run skips file open for nonexistent path

- **Given:** a path that does not exist; `ClaudeCommand::new().with_stdin_file(nonexistent_path).with_dry_run(true)`
- **When:** `execute()` is called
- **Then:** Returns `Ok` (dry-run returns describe output before any file open attempt)
- **Source:** [feature/005_stdin_file.md](../../../docs/feature/005_stdin_file.md)

---

### FT-5: Last-write wins on repeated with_stdin_file

- **Given:** two distinct paths `path_a` and `path_b`; `ClaudeCommand::new().with_stdin_file(path_a).with_stdin_file(path_b).with_dry_run(true)`
- **When:** `execute()` is called
- **Then:** Returns `Ok`; describe output contains `path_b` and does NOT contain `path_a`
- **Source:** [feature/005_stdin_file.md](../../../docs/feature/005_stdin_file.md)
