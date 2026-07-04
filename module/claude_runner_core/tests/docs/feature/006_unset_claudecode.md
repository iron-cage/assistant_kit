# Feature :: CLAUDECODE Environment Variable Unsetting

### Scope

- **Purpose**: FT- test cases verifying the `unset_claudecode` field on `ClaudeCommand` and its effect on env removal and describe output.
- **Responsibility**: Acceptance criteria confirming default CLAUDECODE removal, opt-out behavior, `build_command()` wiring location, override semantics, and WYSIWYG `describe()` parity.
- **In Scope**: `with_unset_claudecode(true/false)` env-pair effect via `get_envs()`, default-true behavior, `build_command_for_test()` wiring, last-write-wins override, `describe()` prefix (`"env -u CLAUDECODE"` vs plain `"claude"`).
- **Out of Scope**: stdin file piping (-> `005_stdin_file.md`), `run_isolated()`/`IsolatedModel` behavior (-> `004_run_isolated.md`).

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
| FT-6 | Default `new()` → `describe()` starts with `"env -u CLAUDECODE"` (WYSIWYG invariant) | WYSIWYG Describe |
| FT-7 | `with_unset_claudecode(false)` → `describe()` starts with plain `"claude"` (no env prefix) | WYSIWYG Describe |

## Test Coverage Summary

- Behavioral Divergence: 2 tests (FT-1, FT-2)
- Default Confirmation: 1 test (FT-3)
- Wiring Location: 1 test (FT-4)
- Override Semantics: 1 test (FT-5)
- WYSIWYG Describe: 2 tests (FT-6, FT-7)

**Total:** 7 feature cases

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

---

### FT-6: Default → describe() starts with "env -u CLAUDECODE" (WYSIWYG invariant)

- **Given:** `ClaudeCommand::new()` (default: `unset_claudecode = true`); `describe()` called
- **When:** the returned string is inspected
- **Then:** The string starts with `"env -u CLAUDECODE"` — mirroring the `env_remove("CLAUDECODE")` in `build_command()`; describes WYSIWYG what will actually execute
- **Source:** [feature/006_unset_claudecode.md](../../../docs/feature/006_unset_claudecode.md), [feature/003_describe.md](../../../docs/feature/003_describe.md)

---

### FT-7: with_unset_claudecode(false) → describe() starts with plain "claude"

- **Given:** `ClaudeCommand::new().with_unset_claudecode(false)`; `describe()` called
- **When:** the returned string is inspected
- **Then:** The string starts with `"claude"` (no `env -u CLAUDECODE` prefix) — CLAUDECODE is NOT removed in `build_command()` so it does NOT appear in `describe()` output
- **Source:** [feature/006_unset_claudecode.md](../../../docs/feature/006_unset_claudecode.md), [feature/003_describe.md](../../../docs/feature/003_describe.md)
