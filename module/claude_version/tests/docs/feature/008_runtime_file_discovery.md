# Feature Test: Runtime File Discovery

### Scope

- **Purpose**: FT- test cases for the `.runtime_files` command — path enumeration, output format, and error paths.
- **Responsibility**: Acceptance criteria verifying that all managed runtime file paths are emitted, one absolute path per line, in pipeline-composable format.
- **In Scope**: Path output completeness, absolute path format, success when files are absent from disk, HOME dependency, exit codes.
- **Out of Scope**: Version history cache content and lifecycle (→ `../../algorithm/002_config_resolution.md`), config resolution tests (→ `006_config_command.md`).

Feature test surface for `.runtime_files`. See [feature/008_runtime_file_discovery.md](../../../docs/feature/008_runtime_file_discovery.md) for specification.

## Behavioral Divergence Pair

Two valid `.runtime_files` invocations that produce structurally different outcomes:

- **Input A:** `HOME=/valid/home clv .runtime_files` → exit 0; absolute path list emitted
- **Input B:** `HOME= clv .runtime_files` → exit 2; no path output

## Test Case Index

| FT | Scenario | Source fn |
|----|----------|-----------|
| FT-1 | `.runtime_files` exits 0; version_history_cache.json path present in output | `ft1_show_all_exits_0_with_cache_path` |
| FT-2 | Output format: one absolute path per line, no headers or decorations | `ft2_output_format_one_path_per_line` |
| FT-3 | Command succeeds even when the listed files do not yet exist on disk | `ft3_succeeds_when_files_absent` |
| FT-4 | Path is absolute and derived from HOME (not relative or hardcoded) | `ft4_path_absolute_and_uses_home_expansion` |
| FT-5 | HOME unset → exit 2 | `ft5_home_unset_exits_2` |

## Test Coverage Summary

- Path completeness: 1 test (FT-1)
- Output format: 1 test (FT-2)
- Stateless resolution: 1 test (FT-3)
- Absolute path with HOME: 1 test (FT-4)
- Error path: 1 test (FT-5)

**Total:** 5 tests

---

### FT-1: show-all exits 0 with version_history_cache.json path

- **Given:** `HOME=/tmp/test_home` (directory exists); `CLAUDE_CODE_BASH_TIMEOUT` unset
- **When:** `clv .runtime_files`
- **Then:** exit 0; stdout contains the string `/tmp/test_home/.claude/.transient/version_history_cache.json`
- **Exit:** 0
- **Source:** [feature/008_runtime_file_discovery.md — Design: Path list](../../../docs/feature/008_runtime_file_discovery.md)

---

### FT-2: output is one path per line, no headers or decorations

- **Given:** `HOME=/tmp/test_home`
- **When:** `clv .runtime_files`
- **Then:** exit 0; stdout lines are raw absolute paths only; no heading text, no label prefixes, no trailing commas or brackets; exactly one path per non-empty line
- **Exit:** 0
- **Source:** [feature/008_runtime_file_discovery.md — Design: Output format](../../../docs/feature/008_runtime_file_discovery.md)

---

### FT-3: command succeeds even when files do not exist on disk

- **Given:** `HOME=/tmp/test_home` where `.claude/.transient/version_history_cache.json` does NOT exist
- **When:** `clv .runtime_files`
- **Then:** exit 0; stdout still contains the path; absence of the file on disk does not cause a failure
- **Exit:** 0
- **Source:** [feature/008_runtime_file_discovery.md — Design: Behavior](../../../docs/feature/008_runtime_file_discovery.md)

---

### FT-4: path is absolute and uses HOME expansion

- **Given:** `HOME=/custom/user_home`
- **When:** `clv .runtime_files`
- **Then:** exit 0; stdout contains `/custom/user_home/.claude/.transient/version_history_cache.json`; path begins with `/` (absolute); path contains the HOME value as prefix
- **Exit:** 0
- **Source:** [feature/008_runtime_file_discovery.md — Design: Path list](../../../docs/feature/008_runtime_file_discovery.md)

---

### FT-5: HOME unset → exit 2

- **Given:** `HOME` environment variable unset or empty
- **When:** `clv .runtime_files`
- **Then:** exit 2; no path output; stderr or stdout indicates HOME is required
- **Exit:** 2
- **Source:** [feature/008_runtime_file_discovery.md — Design: Exit codes](../../../docs/feature/008_runtime_file_discovery.md)

---

## Source Functions

| Function | File | Test Cases |
|----------|------|------------|
| `ft1_show_all_exits_0_with_cache_path` | `tests/cli/runtime_files_test.rs` | FT-1 |
| `ft2_output_format_one_path_per_line` | `tests/cli/runtime_files_test.rs` | FT-2 |
| `ft3_succeeds_when_files_absent` | `tests/cli/runtime_files_test.rs` | FT-3 |
| `ft4_path_absolute_and_uses_home_expansion` | `tests/cli/runtime_files_test.rs` | FT-4 |
| `ft5_home_unset_exits_2` | `tests/cli/runtime_files_test.rs` | FT-5 |
