# Test: Config Management

Acceptance tests for User Story 006. See [user_story/006_config_management.md](../../../../docs/cli/user_story/006_config_management.md) for specification.

### Scope

- **Purpose**: Verify `.config` provides 4-layer settings inspection and atomic read/write with type inference.
- **Responsibility**: Acceptance criteria coverage for the config management workflow.
- **Commands:** `.config`, `.help`
- **In Scope**: Show-all mode, single-key read, write with type inference, project scope, dry-run, unset, error handling.
- **Out of Scope**: Parameter edge cases (-> `../param/`), group interactions (-> `../param_group/`).

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| AT-1 | `.config` shows all settings with source annotations | Acceptance: show-all |
| AT-2 | `.config key::X` shows effective value with source layer | Acceptance: single read |
| AT-3 | `.config key::X format::json` returns value as JSON | Acceptance: JSON read |
| AT-4 | `.config key::X value::V` writes with type inference | Acceptance: write |
| AT-5 | `.config key::X value::V scope::project` writes to project | Acceptance: project write |
| AT-6 | `.config key::X value::V dry::1` previews without writing | Acceptance: dry-run |
| AT-7 | `.config key::X unset::1` removes key from user settings | Acceptance: unset |
| AT-8 | `.config key::X unset::1 scope::project` removes from project | Acceptance: project unset |
| AT-9 | Type inference: `"true"` → bool, `"42"` → number, `"hello"` → string | Acceptance: type inference |
| AT-10 | Invalid combination `value::V unset::1` → exit 1 | Acceptance: error handling |

## Test Coverage Summary

- Show-all: 1 test (AT-1)
- Single key read: 2 tests (AT-2, AT-3)
- Write operations: 3 tests (AT-4, AT-5, AT-6)
- Unset operations: 2 tests (AT-7, AT-8)
- Type inference: 1 test (AT-9)
- Error handling: 1 test (AT-10)

**Total:** 10 acceptance tests

---

### AT-1: `.config` shows all settings with source annotations

- **Given:** user settings has `theme: "dark"`; project settings has `model: "claude-opus-4-6"`; CLAUDE_MODEL is unset
- **When:** `clv .config`
- **Then:** exit 0; output lists all resolved keys, each annotated with its source layer (env/project/user/catalog)
- **Exit:** 0
- **Source:** [user_story/006 — AC1](../../../../docs/cli/user_story/006_config_management.md)

---

### AT-2: `.config key::X` shows effective value with source layer

- **Given:** user settings has `theme: "dark"`
- **When:** `clv .config key::theme`
- **Then:** exit 0; output shows `theme: "dark"` annotated with source layer `[user]`
- **Exit:** 0
- **Source:** [user_story/006 — AC2](../../../../docs/cli/user_story/006_config_management.md)

---

### AT-3: `.config key::X format::json` returns value as JSON

- **Given:** user settings has `theme: "dark"`
- **When:** `clv .config key::theme format::json`
- **Then:** exit 0; output is valid JSON containing the effective value and source layer
- **Exit:** 0
- **Source:** [user_story/006 — AC3](../../../../docs/cli/user_story/006_config_management.md)

---

### AT-4: `.config key::X value::V` writes with type inference

- **Given:** `~/.claude/settings.json` accessible; HOME is set
- **When:** `clv .config key::theme value::dark`
- **Then:** exit 0; `~/.claude/settings.json` updated atomically; `"theme"` stored as JSON string `"dark"`
- **Exit:** 0
- **Source:** [user_story/006 — AC4](../../../../docs/cli/user_story/006_config_management.md)

---

### AT-5: `.config key::X value::V scope::project` writes to project

- **Given:** cwd accessible; user settings file unchanged
- **When:** `clv .config key::theme value::dark scope::project`
- **Then:** exit 0; `.claude/settings.json` in cwd contains `"theme": "dark"`; `~/.claude/settings.json` unchanged
- **Exit:** 0
- **Source:** [user_story/006 — AC5](../../../../docs/cli/user_story/006_config_management.md)

---

### AT-6: `.config key::X value::V dry::1` previews without writing

- **Given:** `~/.claude/settings.json` exists
- **When:** `clv .config key::theme value::dark dry::1`
- **Then:** exit 0; output shows `[dry-run]` preview of what would be written; settings file not modified
- **Exit:** 0
- **Source:** [user_story/006 — AC6](../../../../docs/cli/user_story/006_config_management.md)

---

### AT-7: `.config key::X unset::1` removes key from user settings

- **Given:** `~/.claude/settings.json` contains key `theme`
- **When:** `clv .config key::theme unset::1`
- **Then:** exit 0; `theme` key no longer present in `~/.claude/settings.json`
- **Exit:** 0
- **Source:** [user_story/006 — AC7](../../../../docs/cli/user_story/006_config_management.md)

---

### AT-8: `.config key::X unset::1 scope::project` removes from project

- **Given:** `.claude/settings.json` in cwd contains key `model`; user settings unchanged
- **When:** `clv .config key::model unset::1 scope::project`
- **Then:** exit 0; `model` key removed from `.claude/settings.json` in cwd; `~/.claude/settings.json` unchanged
- **Exit:** 0
- **Source:** [user_story/006 — AC8](../../../../docs/cli/user_story/006_config_management.md)

---

### AT-9: Type inference: `"true"` → bool, `"42"` → number, `"hello"` → string

- **Given:** `~/.claude/settings.json` accessible
- **When:** `clv .config key::enabled value::true` then `clv .config key::count value::42` then `clv .config key::label value::hello`
- **Then:** exit 0 for each; `enabled` stored as JSON `true` (bool); `count` stored as JSON `42` (number); `label` stored as JSON `"hello"` (string)
- **Exit:** 0
- **Source:** [user_story/006 — AC9](../../../../docs/cli/user_story/006_config_management.md)

---

### AT-10: Invalid combination `value::V unset::1` → exit 1

- **Given:** any invocation with both `value::` and `unset::1`
- **When:** `clv .config key::theme value::dark unset::1`
- **Then:** exit 1; error message states `value::` and `unset::` are mutually exclusive; no file modified
- **Exit:** 1
- **Source:** [user_story/006 — AC10](../../../../docs/cli/user_story/006_config_management.md)
