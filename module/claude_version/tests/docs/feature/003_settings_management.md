# Feature Test: Settings Management

### Scope

- **Purpose**: FT- test cases for settings.json read, write, type preservation, and atomic write.
- **Responsibility**: Acceptance criteria verifying round-trip consistency, atomic creation, nested object preservation, and HOME dependency.
- **In Scope**: `.settings.show`, `.settings.get`, `.settings.set`, round-trip, atomic creation, nested object pass-through, HOME=unset error.
- **Out of Scope**: Type inference algorithm (-> `../../algorithm/01_settings_type_inference.md`), dry-run semantics (-> `04_dry_run.md`).

Feature test surface for settings management. See [feature/003_settings_management.md](../../../docs/feature/003_settings_management.md) for specification.

## Behavioral Divergence Pair

Two valid `.settings.set` values store different JSON types:

- **Input A:** `clv .settings.set key::flag value::true` → `settings.json` contains `"flag": true` (JSON boolean)
- **Input B:** `clv .settings.set key::count value::42` → `settings.json` contains `"count": 42` (JSON integer)

Both are valid invocations; the stored JSON type differs.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| FT-1 | `value::true` stores boolean `true` (not string `"true"`) | Type Preservation |
| FT-2 | `value::42` stores integer `42` (not string `"42"`) | Type Preservation |
| FT-3 | Set + get round-trip returns stored value | Round-Trip |
| FT-4 | Creates `settings.json` when file is absent | Atomic Creation |
| FT-5 | `HOME` unset → exit 2 | HOME Dependency |

## Test Coverage Summary

- Type Preservation: 2 tests (FT-1, FT-2)
- Round-Trip: 1 test (FT-3)
- Atomic Creation: 1 test (FT-4)
- HOME Dependency: 1 test (FT-5)

**Total:** 5 tests

---

### FT-1: `value::true` stores boolean `true` (not string `"true"`)

- **Given:** isolated HOME with empty `settings.json`
- **When:** `clv .settings.set key::flag value::true`
- **Then:** `settings.json` contains `"flag": true` (unquoted boolean); exit 0
- **Exit:** 0
- **Source:** [feature/003_settings_management.md — Commands](../../../docs/feature/003_settings_management.md), [algorithm/001_settings_type_inference.md](../../../docs/algorithm/001_settings_type_inference.md)

---

### FT-2: `value::42` stores integer `42` (not string `"42"`)

- **Given:** isolated HOME with empty `settings.json`
- **When:** `clv .settings.set key::count value::42`
- **Then:** `settings.json` contains `"count": 42` (unquoted integer); exit 0
- **Exit:** 0
- **Source:** [feature/003_settings_management.md — Commands](../../../docs/feature/003_settings_management.md), [algorithm/001_settings_type_inference.md](../../../docs/algorithm/001_settings_type_inference.md)

---

### FT-3: Set + get round-trip returns stored value

- **Given:** isolated HOME with empty `settings.json`
- **When:** `clv .settings.set key::color value::blue` then `clv .settings.get key::color`
- **Then:** `settings.get` stdout contains `"blue"`; exit 0 on both commands
- **Exit:** 0
- **Source:** [feature/003_settings_management.md — Commands](../../../docs/feature/003_settings_management.md)

---

### FT-4: Creates `settings.json` when file is absent

- **Given:** isolated HOME with no `.claude/` directory
- **When:** `clv .settings.set key::x value::1`
- **Then:** `settings.json` is created and contains `"x": 1`; exit 0
- **Exit:** 0
- **Source:** [feature/003_settings_management.md — Atomic write](../../../docs/feature/003_settings_management.md)

---

### FT-5: `HOME` unset → exit 2

- **Given:** environment with `HOME` unset
- **When:** `clv .settings.set key::x value::1`
- **Then:** exit 2 (runtime error, HOME missing)
- **Exit:** 2
- **Source:** [feature/003_settings_management.md — HOME dependency](../../../docs/feature/003_settings_management.md)

---

### Source Functions

| Function | File |
|----------|------|
| `tc322_settings_set_value_true_stores_bool` | `integration/mutation_commands_test.rs` |
| `tc325_settings_set_value_42_stores_number` | `integration/mutation_commands_test.rs` |
| `ft003_settings_set_get_round_trip` | `integration/feature_surface_test.rs` |
| `tc328_settings_set_creates_file_when_absent` | `integration/mutation_commands_test.rs` |
| `tc331_settings_set_home_not_set_exits_2` | `integration/mutation_commands_test.rs` |
