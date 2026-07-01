# Test: `.paths`

Integration test planning for the `.paths` command. See [command/namespace.md](../../../../docs/cli/command/004_paths.md#command--8-paths) for specification.

### Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| IT-1 | Shows all 7 canonical paths with labels | Basic Invocation |
| IT-2 | `format::json` returns JSON object with all path keys | Output Format |
| IT-3 | HOME not set exits 2 | Error Handling |
| IT-4 | All output paths are absolute | Path Correctness |
| IT-5 | Paths resolve from actual HOME value | Path Correctness |
| IT-6 | `field::credential_store` outputs single raw path value | Field Selection |
| IT-7 | `field::` with unknown name exits 1 | Error Handling |
| IT-8 | `no_color::1` accepted; path output unchanged (no color codes emitted) | Edge Param |

### Test Coverage Summary

- Basic Invocation: 1 test
- Output Format: 1 test
- Error Handling: 2 tests
- Path Correctness: 2 tests
- Field Selection: 1 test
- Edge Param: 1 test

**Total:** 8 integration tests

---

### IT-1: Shows all 7 canonical paths with labels

- **Given:** `HOME` is set to a valid directory (e.g., `/home/testuser`). No special account or credential state required.
- **When:** `clp .paths`
- **Then:** Seven labeled lines on stdout covering: `credentials`, `credential_store`, `projects`, `stats`, `settings`, `session-env`, `sessions`. Exit 0.; all 7 labeled paths present in output
- **Exit:** 0
- **Source:** [command/004_paths.md — .paths](../../../../docs/cli/command/004_paths.md#command--8-paths)

---

### IT-2: JSON format returns object with all path keys

- **Given:** `HOME` set to `/home/testuser`.
- **When:** `clp .paths format::json`
- **Then:** Valid JSON object on stdout with keys for all 7 paths plus `base`, exit 0.; valid JSON object with all expected path keys
- **Exit:** 0
- **Source:** [command/004_paths.md — .paths](../../../../docs/cli/command/004_paths.md#command--8-paths)

---

### IT-3: HOME not set exits 2

- **Given:** Unset the `HOME` environment variable before invocation.
- **When:** `env -u HOME clp .paths`
- **Then:** Error message on stderr indicating `HOME` is not set, exit 2.; stderr contains error about HOME not set
- **Exit:** 2
- **Source:** [command/004_paths.md — .paths](../../../../docs/cli/command/004_paths.md#command--8-paths)

---

### IT-4: All output paths are absolute

- **Given:** `HOME` set to `/home/testuser`.
- **When:** `clp .paths`
- **Then:** Seven labeled lines, each path starting with `/`, exit 0.; all paths are absolute
- **Exit:** 0
- **Source:** [command/004_paths.md — .paths](../../../../docs/cli/command/004_paths.md#command--8-paths)

---

### IT-5: Paths resolve from actual HOME value

- **Given:** Set `HOME` to a custom temporary directory (e.g., `/tmp/test_home_12345`). Create `$HOME/.claude/` directory.
- **When:** `HOME=/tmp/test_home_12345 clp .paths`
- **Then:** All paths rooted under `/tmp/test_home_12345/.claude/`, exit 0.; all paths rooted under the custom HOME value
- **Exit:** 0
- **Source:** [command/004_paths.md — .paths](../../../../docs/cli/command/004_paths.md#command--8-paths)

---

### IT-6: `field::credential_store` outputs single raw path value

- **Given:** `HOME` is set to a valid temporary directory.
- **When:** `clp .paths field::credential_store`
- **Then:** A single line on stdout containing the credential store path (no label prefix, no JSON wrapper). Exit 0.
- **Exit:** 0
- **Status:** ✅ Implemented
- **Source:** tests/cli/token_paths_test.rs :: p009_paths_field_returns_single_value

---

### IT-7: `field::` with unknown name exits 1

- **Given:** `HOME` is set to a valid temporary directory.
- **When:** `clp .paths field::nonexistent`
- **Then:** Error message on stderr listing valid field names. Exit 1.
- **Exit:** 1
- **Status:** ✅ Implemented
- **Source:** tests/cli/token_paths_test.rs :: p010_paths_field_unknown_exits_1

---

### IT-8: `no_color::1` accepted; path output unchanged

- **Given:** `HOME` is set to a valid temporary directory.
- **When:** `clp .paths no_color::1`
- **Then:** Seven labeled lines on stdout (identical to IT-1 output). No ANSI color codes present. Exit 0. `.paths` uses plain-text formatting regardless of `no_color::` — the flag is accepted without error.
- **Exit:** 0
- **Source:** [command/004_paths.md — .paths](../../../../docs/cli/command/004_paths.md#command--8-paths)
