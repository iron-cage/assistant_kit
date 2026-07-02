# Test: `.runtime_files`

### Scope

- **Purpose**: Integration test cases for the `.runtime_files` command.
- **Responsibility**: Test factor analysis, case index, and expected behavior for path enumeration, format, HOME dependency, and error paths.
- **In Scope**: Path completeness, output format, absolute path validation, HOME expansion, exit codes, pipeline composability.
- **Out of Scope**: Cache content and JSON schema (→ `../../feature/001_version_management.md`), per-runtime-file lifecycle tests (→ `../../runtime_file/001_version_history_cache.md`).

Integration test planning for `.runtime_files`. See [cli/command/root.md](../../../../docs/cli/command/root.md) for specification.

## Test Factor Analysis

### Factor 1: HOME environment variable

| Level | Description | Equivalence Class |
|-------|-------------|-------------------|
| set, non-empty | Valid HOME path | Happy path |
| unset | HOME absent from environment | Error: exit 2 |
| empty string | HOME set to "" | Error: exit 2 |

### Factor 2: Runtime file existence on disk

| Level | Description | Equivalence Class |
|-------|-------------|-------------------|
| files present | All listed paths exist on disk | Valid |
| files absent | None of the listed paths exist on disk | Also valid (stateless listing) |

### Factor 3: Output format

| Level | Description | Equivalence Class |
|-------|-------------|-------------------|
| default | One absolute path per line, no headers | Required |

---

## Test Matrix

### Positive Tests

| TC | Description | Exit | Factors |
|----|-------------|------|---------|
| IT-1 | Basic invocation exits 0; version_history_cache.json path present in stdout | 0 | F1=set |
| IT-2 | Output lines are raw absolute paths only; no headers, labels, or decorations | 0 | F1=set |
| IT-3 | Each non-empty output line begins with `/` (absolute path) | 0 | F1=set |
| IT-4 | Path contains HOME value as prefix — output reflects current HOME | 0 | F1=set |
| IT-5 | HOME=/custom/path → stdout contains `/custom/path/.claude/.transient/version_history_cache.json` | 0 | F1=set |
| IT-6 | Command exits 0 when listed files do not exist on disk | 0 | F1=set, F2=absent |
| IT-7 | All registered runtime file paths present in output (completeness cross-coverage) | 0 | F1=set |
| IT-8 | Pipeline composability: output piped to `wc -l` gives expected path count (≥1) | 0 | F1=set |

### Negative Tests

| TC | Description | Exit | Factors |
|----|-------------|------|---------|
| IT-9 | HOME unset → exit 2; no path output | 2 | F1=unset |

### Summary

- **Total:** 9 tests (8 positive, 1 negative)
- **Negative ratio:** 11.1% (structurally limited for a path-enumeration command)
- **TC range:** IT-1 to IT-9

---

## Coverage Verification

### Exit Status Coverage

| Exit Code | Meaning | Tests |
|-----------|---------|-------|
| 0 | Success | IT-1 through IT-8 |
| 2 | HOME unset or I/O error | IT-9 |

### Factor Coverage

| Factor | Tests |
|--------|-------|
| HOME set (happy path) | IT-1, IT-2, IT-3, IT-4, IT-5, IT-6, IT-7, IT-8 |
| HOME unset (error) | IT-9 |
| Files absent on disk | IT-6 |
| Pipeline composability | IT-8 |

### Runtime File Cross-Coverage

| Requirement | Tests |
|-------------|-------|
| All runtime file paths in output | IT-7 |
| Command succeeds when component not running | IT-6 |
| Output is one path per line, no headers | IT-2 |

---

## Test Case Details

---

### IT-1: Basic invocation exits 0 with expected path

- **Given:** `HOME=/tmp/test_home`
- **When:** `clv .runtime_files`
- **Then:** exit 0; stdout contains `/tmp/test_home/.claude/.transient/version_history_cache.json`
- **Exit:** 0
- **Source:** [cli/command/root.md — Command 15](../../../../docs/cli/command/root.md)

---

### IT-2: Output is raw paths only — no headers or decorations

- **Given:** `HOME=/tmp/test_home`
- **When:** `clv .runtime_files`
- **Then:** exit 0; every non-empty stdout line is a raw path string; no heading text, label prefixes, trailing commas, or brackets appear; exactly one path per non-empty line
- **Exit:** 0
- **Source:** [cli/command/root.md — Command 15](../../../../docs/cli/command/root.md)

---

### IT-3: Each output line is an absolute path

- **Given:** `HOME=/tmp/test_home`
- **When:** `clv .runtime_files`
- **Then:** exit 0; every non-empty stdout line starts with `/`; no relative paths present
- **Exit:** 0
- **Source:** [cli/command/root.md — Command 15](../../../../docs/cli/command/root.md)

---

### IT-4: Path reflects current HOME value

- **Given:** `HOME=/tmp/test_home`
- **When:** `clv .runtime_files`
- **Then:** exit 0; stdout contains a path that starts with `/tmp/test_home`; path contains the HOME value as its prefix
- **Exit:** 0
- **Source:** [cli/command/root.md — Command 15](../../../../docs/cli/command/root.md)

---

### IT-5: Custom HOME produces correct path prefix

- **Given:** `HOME=/custom/path`
- **When:** `clv .runtime_files`
- **Then:** exit 0; stdout contains `/custom/path/.claude/.transient/version_history_cache.json`; path starts with `/custom/path`
- **Exit:** 0
- **Source:** [cli/command/root.md — Command 15](../../../../docs/cli/command/root.md)

---

### IT-6: Command exits 0 when files do not exist on disk

- **Given:** `HOME=/tmp/test_home` where `.claude/.transient/version_history_cache.json` does NOT exist on disk
- **When:** `clv .runtime_files`
- **Then:** exit 0; stdout still contains the expected path string; absence of the file on disk is not an error
- **Exit:** 0
- **Source:** [cli/command/root.md — Command 15](../../../../docs/cli/command/root.md)

---

### IT-7: All registered runtime file paths present in output

- **Given:** `HOME=/tmp/test_home`
- **When:** `clv .runtime_files`
- **Then:** exit 0; stdout contains the version_history_cache.json path; all paths registered in `docs/runtime_file/` are present
- **Exit:** 0
- **Source:** [cli/command/root.md — Command 15](../../../../docs/cli/command/root.md)

---

### IT-8: Pipeline composability — output can be consumed by line

- **Given:** `HOME=/tmp/test_home`
- **When:** `clv .runtime_files | wc -l`
- **Then:** exit 0; line count equals the number of registered runtime files (at least 1); output is machine-parseable without preprocessing
- **Exit:** 0
- **Source:** [cli/command/root.md — Command 15](../../../../docs/cli/command/root.md)

---

### IT-9: HOME unset → exit 2

- **Given:** `HOME` environment variable unset (removed from environment)
- **When:** `clv .runtime_files`
- **Then:** exit 2; no path output on stdout; stderr or stdout indicates HOME is required
- **Exit:** 2
- **Source:** [cli/command/root.md — Command 15](../../../../docs/cli/command/root.md)

---

## Source Functions Table

| Function | File | Test Cases |
|----------|------|------------|
| `it1_runtime_files_exits_0_with_cache_path` | `tests/cli/runtime_files_test.rs` | IT-1 |
| `it2_output_is_raw_paths_only` | `tests/cli/runtime_files_test.rs` | IT-2 |
| `it3_each_line_is_absolute` | `tests/cli/runtime_files_test.rs` | IT-3 |
| `it4_path_reflects_home` | `tests/cli/runtime_files_test.rs` | IT-4 |
| `it5_custom_home_prefix` | `tests/cli/runtime_files_test.rs` | IT-5 |
| `it6_succeeds_when_files_absent` | `tests/cli/runtime_files_test.rs` | IT-6 |
| `it7_all_registered_paths_present` | `tests/cli/runtime_files_test.rs` | IT-7 |
| `it8_pipeline_composable_line_count` | `tests/cli/runtime_files_test.rs` | IT-8 |
| `it9_home_unset_exits_2` | `tests/cli/runtime_files_test.rs` | IT-9 |
