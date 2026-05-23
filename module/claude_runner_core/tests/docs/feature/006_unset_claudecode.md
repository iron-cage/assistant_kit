# Feature :: CLAUDECODE Environment Variable Unsetting

Behavioral requirement cases for the `unset_claudecode` field on `ClaudeCommand`. See
[feature/006_unset_claudecode.md](../../../docs/feature/006_unset_claudecode.md) for the specification.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| FT-1 | Default `new()` → `CLAUDECODE` marked for removal in command env | Behavioral Divergence |
| FT-2 | `with_unset_claudecode(false)` → `CLAUDECODE` NOT marked for removal | Behavioral Divergence |
| FT-3 | `with_unset_claudecode(true)` explicit → same as default (CLAUDECODE removed) | Default Confirmation |
| FT-4 | env removal is in `build_command()` — applies in dry-run inspect via `build_command_for_test()` | Wiring Location |
| FT-5 | `with_unset_claudecode(false).with_unset_claudecode(true)` → last-write wins (CLAUDECODE removed) | Override Semantics |

## Test Coverage Summary

- Behavioral Divergence: 2 tests (FT-1, FT-2)
- Default Confirmation: 1 test (FT-3)
- Wiring Location: 1 test (FT-4)
- Override Semantics: 1 test (FT-5)

**Total:** 5 feature cases

---

### FT-1: Default → CLAUDECODE removed

- **Given:** `ClaudeCommand::new()` with default settings; `build_command_for_test()` called
- **When:** the built command's env pairs are inspected via `get_envs()`
- **Then:** `get_envs()` contains an entry `("CLAUDECODE", None)` — the key is present with a `None` value, indicating removal
- **Source:** [feature/006_unset_claudecode.md](../../../docs/feature/006_unset_claudecode.md)

---

### FT-2: `with_unset_claudecode(false)` → CLAUDECODE NOT removed

- **Given:** `ClaudeCommand::new().with_unset_claudecode(false)`; `build_command_for_test()` called
- **When:** the built command's env pairs are inspected via `get_envs()`
- **Then:** `get_envs()` does NOT contain any entry `("CLAUDECODE", None)` — no removal registered
- **Source:** [feature/006_unset_claudecode.md](../../../docs/feature/006_unset_claudecode.md)

---

### FT-3: Explicit `with_unset_claudecode(true)` → same as default

- **Given:** `ClaudeCommand::new().with_unset_claudecode(true)`; `build_command_for_test()` called
- **When:** the built command's env pairs are inspected via `get_envs()`
- **Then:** `get_envs()` contains `("CLAUDECODE", None)` — identical to the default behavior
- **Source:** [feature/006_unset_claudecode.md](../../../docs/feature/006_unset_claudecode.md)

---

### FT-4: env_remove wired in build_command, not execute — visible via build_command_for_test

- **Given:** `ClaudeCommand::new()` (unset_claudecode = true by default)
- **When:** `build_command_for_test()` is called directly (not `execute()`)
- **Then:** The returned `std::process::Command` has `("CLAUDECODE", None)` in `get_envs()` — proving the wiring is in `build_command()`, not deferred to `execute()`
- **Source:** [feature/006_unset_claudecode.md](../../../docs/feature/006_unset_claudecode.md)

---

### FT-5: Last-write wins on repeated with_unset_claudecode

- **Given:** `ClaudeCommand::new().with_unset_claudecode(false).with_unset_claudecode(true)`; `build_command_for_test()` called
- **When:** the built command's env pairs are inspected via `get_envs()`
- **Then:** `get_envs()` contains `("CLAUDECODE", None)` — the final `true` value wins
- **Source:** [feature/006_unset_claudecode.md](../../../docs/feature/006_unset_claudecode.md)
