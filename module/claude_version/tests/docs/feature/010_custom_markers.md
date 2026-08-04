# Feature Test: Custom Version Markers

### Scope

- **Purpose**: FT- test cases for `.version.mark` CRUD, custom marker resolution, and `.version.list` integration.
- **Responsibility**: Acceptance criteria verifying marker creation, removal, name validation, resolution integration, and list rendering.
- **In Scope**: `.version.mark`, `version::` resolution with custom markers, `.version.list` integration, `.version.show` label reverse-lookup.
- **Out of Scope**: Built-in alias resolution (→ `001_version_management.md`), version guard integration (→ `05_version_guard.md`).

Feature test surface for custom markers. See [feature/010_custom_markers.md](../../../docs/feature/010_custom_markers.md) for specification.

## Behavioral Divergence Pair

Two valid invocations produce distinct output:

- **Input A:** `clv .version.mark name::team-pin version::2.1.220 dry::1` → output shows "would create marker" (no file write)
- **Input B:** `clv .version.mark name::team-pin unset::1 dry::1` → output shows "would remove marker" (no file write)

Both are valid invocations; the mutation direction differs.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| FT-1 | Create marker → appears in `.version.list` | Marker CRUD |
| FT-2 | Remove marker → absent from `.version.list` | Marker CRUD |
| FT-3 | Custom marker name accepted by `.version.install version::name dry::1` | Resolution Integration |
| FT-4 | Invalid name (uppercase start) → exit 1 | Name Validation |
| FT-5 | `dry::1` does not write `version-markers.json` | Preference Isolation |
| FT-6 | Marker matching installed version → label shown by `.version.show` | Show Integration |

## Test Coverage Summary

- Marker CRUD: 2 tests (FT-1, FT-2)
- Resolution Integration: 1 test (FT-3)
- Name Validation: 1 test (FT-4)
- Preference Isolation: 1 test (FT-5)
- Show Integration: 1 test (FT-6)

**Total:** 6 tests

---

### FT-1: Create marker → appears in `.version.list`

- **Given:** isolated HOME with no `version-markers.json`
- **When:** `clv .version.mark name::my-pin version::2.1.220`; then `clv .version.list`
- **Then:** exit 0; `.version.list` output contains `my-pin`
- **Exit:** 0
- **Source:** [feature/010_custom_markers.md](../../../docs/feature/010_custom_markers.md)

---

### FT-2: Remove marker → absent from `.version.list`

- **Given:** isolated HOME with `version-markers.json` containing `my-pin → 2.1.220`
- **When:** `clv .version.mark name::my-pin unset::1`; then `clv .version.list`
- **Then:** exit 0; `.version.list` output does not contain `my-pin`
- **Exit:** 0
- **Source:** [feature/010_custom_markers.md](../../../docs/feature/010_custom_markers.md)

---

### FT-3: Custom marker accepted by `.version.install`

- **Given:** isolated HOME with `version-markers.json` containing `my-pin → 2.1.220`
- **When:** `clv .version.install version::my-pin dry::1`
- **Then:** exit 0; stdout contains `2.1.220`; dry-run marker present
- **Exit:** 0
- **Source:** [feature/010_custom_markers.md](../../../docs/feature/010_custom_markers.md)

---

### FT-4: Invalid name (uppercase start) → exit 1

- **Given:** clean environment
- **When:** `clv .version.mark name::MyPin version::2.1.220`
- **Then:** exit 1; stderr contains error about invalid marker name
- **Exit:** 1
- **Source:** [feature/010_custom_markers.md](../../../docs/feature/010_custom_markers.md)

---

### FT-5: `dry::1` does not write `version-markers.json`

- **Given:** isolated HOME with no `version-markers.json`
- **When:** `clv .version.mark name::my-pin version::2.1.220 dry::1`
- **Then:** exit 0; `version-markers.json` does not exist after the call
- **Exit:** 0
- **Source:** [feature/010_custom_markers.md](../../../docs/feature/010_custom_markers.md)

---

---

### FT-6: Marker matching installed version → label shown by `.version.show`

- **Given:** claude installed; isolated HOME with `version-markers.json` containing a marker whose `value` equals the installed semver (e.g. `{"name":"my-pin","value":"<installed>","description":""}`).
- **When:** `clv .version.show v::1`
- **Then:** exit 0; stdout contains `[my-pin]`
- **Exit:** 0
- **Source:** [feature/010_custom_markers.md](../../../docs/feature/010_custom_markers.md)

---

### Source Functions

| Function | File |
|----------|------|
| `ft010_1_create_marker_appears_in_list` | `tests/cli/mutation_version_mark_test.rs` |
| `ft010_2_remove_marker_absent_from_list` | `tests/cli/mutation_version_mark_test.rs` |
| `ft010_3_custom_marker_accepted_by_install` | `tests/cli/mutation_version_mark_test.rs` |
| `ft010_4_invalid_name_exits_1` | `tests/cli/mutation_version_mark_test.rs` |
| `ft010_5_dry_does_not_write_markers_file` | `tests/cli/mutation_version_mark_test.rs` |
| `ft010_6_marker_label_shown_by_version_show` | `tests/cli/read_version_test.rs` |
