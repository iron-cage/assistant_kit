# Feature Test: Dry Run

### Scope

- **Purpose**: FT- test cases for dry::1 preview mode across all mutation commands.
- **Responsibility**: Acceptance criteria verifying dry-run output prefix, no side effects, parity with action message, and dry+force precedence.
- **In Scope**: `dry::1` on `.version.install`, `.processes.kill`, `.settings.set`; parity requirement; `dry::1 force::1` precedence.
- **Out of Scope**: Individual command normal-mode behavior (-> `01_version_management.md`, `02_process_lifecycle.md`, `03_settings_management.md`).

Feature test surface for dry-run mode. See [feature/004_dry_run.md](../../../../docs/feature/004_dry_run.md) for specification.

## Behavioral Divergence Pair

Two valid invocations of the same command with and without `dry::1` produce distinct output:

- **Input A:** `clv .version.install dry::1` → stdout contains `"[dry-run]"` prefix; no actual install attempted
- **Input B:** `clv .version.install` (without dry::1) → stdout does NOT contain `"[dry-run]"` prefix; install attempted

Both are valid invocations; the `[dry-run]` prefix presence differs.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| FT-1 | `.version.install dry::1` → `[dry-run]` prefix, no install | No Side Effects |
| FT-2 | `.settings.set dry::1` → `[dry-run]` prefix, settings file unchanged | No Side Effects |
| FT-3 | `.processes.kill dry::1` → `[dry-run]` prefix, exit 0 | No Side Effects |
| FT-4 | `dry::1 force::1` → dry wins, `[dry-run]` prefix present | Precedence |

## Test Coverage Summary

- No Side Effects: 3 tests (FT-1, FT-2, FT-3)
- Precedence: 1 test (FT-4)

**Total:** 4 tests

---

### FT-1: `.version.install dry::1` → `[dry-run]` prefix, no install

- **Given:** clean environment
- **When:** `clv .version.install dry::1`
- **Then:** stdout contains `"[dry-run]"`; no installer process spawned; exit 0
- **Exit:** 0
- **Source:** [feature/004_dry_run.md — Affected commands](../../../../docs/feature/004_dry_run.md)

---

### FT-2: `.settings.set dry::1` → `[dry-run]` prefix, settings file unchanged

- **Given:** isolated HOME with empty `settings.json`; record its content before
- **When:** `clv .settings.set key::x value::1 dry::1`
- **Then:** stdout contains `"[dry-run]"`; `settings.json` is unchanged; exit 0
- **Exit:** 0
- **Source:** [feature/004_dry_run.md — Affected commands](../../../../docs/feature/004_dry_run.md)

---

### FT-3: `.processes.kill dry::1` → `[dry-run]` prefix, exit 0

- **Given:** environment with no detectable claude processes
- **When:** `clv .processes.kill dry::1`
- **Then:** stdout contains `"[dry-run]"`; exit 0
- **Exit:** 0
- **Source:** [feature/004_dry_run.md — Affected commands](../../../../docs/feature/004_dry_run.md)

---

### FT-4: `dry::1 force::1` → dry wins, `[dry-run]` prefix present

- **Given:** clean environment
- **When:** `clv .version.install dry::1 force::1`
- **Then:** stdout contains `"[dry-run]"`; `force::1` does not override dry-run; exit 0
- **Exit:** 0
- **Source:** [feature/004_dry_run.md — Precedence](../../../../docs/feature/004_dry_run.md)

---

### Source Functions

| Function | File |
|----------|------|
| `tc300_version_install_dry_shows_prefix` | `integration/mutation_commands_test.rs` |
| `tc330_settings_set_dry_preview_no_change` | `integration/mutation_commands_test.rs` |
| `tc311_processes_kill_dry_no_processes` | `integration/mutation_commands_test.rs` |
| `tc303_version_install_dry_wins_over_force` | `integration/mutation_commands_test.rs` |
