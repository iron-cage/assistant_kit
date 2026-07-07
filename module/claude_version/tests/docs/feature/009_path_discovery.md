# Feature Test: Path Discovery

### Scope

- **Purpose**: FT- test cases for the `.paths` command — path enumeration, single-key lookup, verbosity-tiered unresolvable-path handling, and error paths.
- **Responsibility**: Acceptance criteria verifying that all known clv paths are reported, labeled and unlabeled modes both work, and unresolvable paths are handled per verbosity level.
- **In Scope**: Show-all completeness, single-key mode, v::0/1/2 rendering differences, unresolvable path handling, HOME dependency, exit codes.
- **Out of Scope**: Unlabeled pipeline-only enumeration (→ `008_runtime_file_discovery.md`), runtime file lifecycle (→ `../runtime_file/`).

Feature test surface for `.paths`. See [feature/009_path_discovery.md](../../../docs/feature/009_path_discovery.md) for specification.

## Behavioral Divergence Pair

Two valid `.paths` invocations that produce structurally different outcomes:

- **Input A:** `HOME=/valid/home clv .paths` → exit 0; all 5 labeled paths emitted
- **Input B:** `HOME= clv .paths` → exit 2; no path output

## Test Case Index

| FT | Scenario | Source fn |
|----|----------|-----------|
| FT-1 | `.paths` show-all exits 0; all 5 keys present with labels | `ft1_show_all_exits_0_with_all_keys` |
| FT-2 | `.paths v::0` outputs plain unlabeled paths, one per line | `ft2_v0_output_is_unlabeled` |
| FT-3 | `.paths key::versions_dir` returns exactly that single path | `ft3_single_key_returns_one_path` |
| FT-4 | Unresolvable `project_settings` shown as "(none found)" at v::1 | `ft4_unresolvable_shown_as_none_found_v1` |
| FT-5 | Unresolvable `project_settings` omitted entirely at v::0 | `ft5_unresolvable_omitted_at_v0` |
| FT-6 | HOME unset → exit 2 | `ft6_home_unset_exits_2` |
| FT-7 | `key::bogus` → exit 1 | `ft7_invalid_key_exits_1` |

## Test Coverage Summary

- Path completeness: 1 test (FT-1)
- Output format (unlabeled): 1 test (FT-2)
- Single-key mode: 1 test (FT-3)
- Unresolvable path handling: 2 tests (FT-4, FT-5)
- Error paths: 2 tests (FT-6, FT-7)

**Total:** 7 tests

---

### FT-1: show-all exits 0 with all 5 keys present

- **Given:** `HOME=/tmp/test_home` (directory exists)
- **When:** `clv .paths`
- **Then:** exit 0; stdout contains labels for `settings`, `project_settings`, `versions_dir`, `binary_symlink`, `version_history_cache`
- **Exit:** 0
- **Source:** [feature/009_path_discovery.md — Design: Path list](../../../docs/feature/009_path_discovery.md)

---

### FT-2: v::0 output is plain, unlabeled, one per line

- **Given:** `HOME=/tmp/test_home`
- **When:** `clv .paths v::0`
- **Then:** exit 0; stdout lines are raw absolute paths only (excluding any unresolved key); no label prefixes
- **Exit:** 0
- **Source:** [feature/009_path_discovery.md — Design: Output format](../../../docs/feature/009_path_discovery.md)

---

### FT-3: single-key mode returns exactly one path

- **Given:** `HOME=/tmp/test_home`
- **When:** `clv .paths key::versions_dir v::0`
- **Then:** exit 0; stdout is exactly `/tmp/test_home/.local/share/claude/versions` (plus trailing newline); no other keys present
- **Exit:** 0
- **Source:** [feature/009_path_discovery.md — Design: Output format](../../../docs/feature/009_path_discovery.md)

---

### FT-4: unresolvable project_settings shown as "(none found)" at v::1

- **Given:** `HOME=/tmp/test_home`; current directory has no ancestor `.claude/settings.json`
- **When:** `clv .paths`
- **Then:** exit 0; stdout contains a `project_settings:` line with a `(none found)` placeholder
- **Exit:** 0
- **Source:** [feature/009_path_discovery.md — Design: Behavior](../../../docs/feature/009_path_discovery.md)

---

### FT-5: unresolvable project_settings omitted entirely at v::0

- **Given:** `HOME=/tmp/test_home`; current directory has no ancestor `.claude/settings.json`
- **When:** `clv .paths v::0`
- **Then:** exit 0; stdout contains no line for `project_settings` (neither path nor placeholder)
- **Exit:** 0
- **Source:** [feature/009_path_discovery.md — Design: Output format](../../../docs/feature/009_path_discovery.md)

---

### FT-6: HOME unset → exit 2

- **Given:** `HOME` environment variable unset or empty
- **When:** `clv .paths`
- **Then:** exit 2; no path output; stderr or stdout indicates HOME is required
- **Exit:** 2
- **Source:** [feature/009_path_discovery.md — Design: Exit codes](../../../docs/feature/009_path_discovery.md)

---

### FT-7: `key::bogus` → exit 1

- **Given:** `HOME=/tmp/test_home`
- **When:** `clv .paths key::bogus`
- **Then:** exit 1; stderr contains a message naming the unknown key and the valid key set
- **Exit:** 1
- **Source:** [feature/009_path_discovery.md — Design: Exit codes](../../../docs/feature/009_path_discovery.md)

---

## Source Functions

| Function | File | Test Cases |
|----------|------|------------|
| `ft1_show_all_exits_0_with_all_keys` | `tests/cli/paths_test.rs` | FT-1 |
| `ft2_v0_output_is_unlabeled` | `tests/cli/paths_test.rs` | FT-2 |
| `ft3_single_key_returns_one_path` | `tests/cli/paths_test.rs` | FT-3 |
| `ft4_unresolvable_shown_as_none_found_v1` | `tests/cli/paths_test.rs` | FT-4 |
| `ft5_unresolvable_omitted_at_v0` | `tests/cli/paths_test.rs` | FT-5 |
| `ft6_home_unset_exits_2` | `tests/cli/paths_test.rs` | FT-6 |
| `ft7_invalid_key_exits_1` | `tests/cli/paths_test.rs` | FT-7 |
