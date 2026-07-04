# Test: Settings Management

Acceptance tests for User Story 004. See [user_story/004_settings_management.md](../../../../docs/cli/user_story/004_settings_management.md) for specification.

### Scope

- **Purpose**: Verify settings read/write workflow via `.settings.*` commands.
- **Responsibility**: Acceptance criteria coverage for the settings management scenario.
- **Commands:** `.settings.show`, `.settings.get`, `.settings.set`
- **In Scope**: Show-all, JSON format, get single key, dry-run set, atomic set with type inference.
- **Out of Scope**: Config resolution (-> `.config` command), version management (-> `02_version_upgrade.md`).

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| US-1 | `.settings.show` prints all key-value pairs | Acceptance: show-all |
| US-2 | `.settings.show format::json` returns JSON object | Acceptance: JSON format |
| US-3 | `.settings.get key::X` prints value; exits 2 if absent | Acceptance: get |
| US-4 | `.settings.set key::X value::V dry::1` previews without write | Acceptance: dry-run |
| US-5 | `.settings.set key::X value::V` writes with type inference | Acceptance: set |
| US-6 | Type inference: booleans, integers, floats inferred correctly | Acceptance: type inference |

## Test Coverage Summary

- Show-all: 1 test (US-1)
- JSON format: 1 test (US-2)
- Get single key: 1 test (US-3)
- Dry-run set: 1 test (US-4)
- Atomic set: 1 test (US-5)
- Type inference: 1 test (US-6)

**Total:** 6 tests

---

### US-1: `.settings.show` prints all key-value pairs

- **Given:** `~/.claude/settings.json` contains `{"theme": "dark", "autoUpdates": true}`
- **When:** `clv .settings.show`
- **Then:** exit 0; output contains both key-value pairs
- **Exit:** 0
- **Source:** [user_story/004 -- AC bullet 1](../../../../docs/cli/user_story/004_settings_management.md)

---

### US-2: `.settings.show format::json` returns JSON object

- **Given:** `~/.claude/settings.json` contains at least one setting
- **When:** `clv .settings.show format::json`
- **Then:** exit 0; valid JSON object with all settings
- **Exit:** 0
- **Source:** [user_story/004 -- AC bullet 2](../../../../docs/cli/user_story/004_settings_management.md)

---

### US-3: `.settings.get key::X` prints value; exits 2 if absent

- **Given:** `~/.claude/settings.json` contains `{"theme": "dark"}`
- **When:** `clv .settings.get key::theme` and `clv .settings.get key::nonexistent`
- **Then:** first exits 0 with "dark"; second exits 2
- **Exit:** 0 (found) / 2 (absent)
- **Source:** [user_story/004 -- AC bullet 3](../../../../docs/cli/user_story/004_settings_management.md)

---

### US-4: `.settings.set key::X value::V dry::1` previews without write

- **Given:** `~/.claude/settings.json` exists
- **When:** `clv .settings.set key::theme value::light dry::1`
- **Then:** exit 0; stdout shows preview; file unchanged
- **Exit:** 0
- **Source:** [user_story/004 -- AC bullet 4](../../../../docs/cli/user_story/004_settings_management.md)

---

### US-5: `.settings.set key::X value::V` writes with type inference

- **Given:** `~/.claude/settings.json` exists
- **When:** `clv .settings.set key::theme value::light`
- **Then:** exit 0; `~/.claude/settings.json` contains `"theme": "light"` atomically written
- **Exit:** 0
- **Source:** [user_story/004 -- AC bullet 5](../../../../docs/cli/user_story/004_settings_management.md)

---

### US-6: Type inference for booleans, integers, and floats

- **Given:** `~/.claude/settings.json` exists
- **When:** `clv .settings.set key::a value::true` then `clv .settings.set key::b value::42` then `clv .settings.set key::c value::3.14`
- **Then:** settings file contains JSON `true` (bool), `42` (integer), `3.14` (float)
- **Exit:** 0
- **Source:** [user_story/004 -- AC bullet 6](../../../../docs/cli/user_story/004_settings_management.md)

---

### Source Functions

| Function | File | Status |
|----------|------|--------|
| `us01_004_settings_show_all_pairs` | `tests/cli/user_story_test.rs` | ✅ |
| `us02_004_settings_show_json` | `tests/cli/user_story_test.rs` | ✅ |
| `us03_004_settings_get_found_and_missing` | `tests/cli/user_story_test.rs` | ✅ |
| `us04_004_settings_set_dry_preview` | `tests/cli/user_story_test.rs` | ✅ |
| `us05_004_settings_set_writes_atomically` | `tests/cli/user_story_test.rs` | ✅ |
| `us06_004_settings_set_type_inference` | `tests/cli/user_story_test.rs` | ✅ |
