# Feature Doc Entity

### Scope

- **Purpose**: Document behavioral requirement cases for `claude_runner_core` feature doc instances.
- **Responsibility**: Index of per-feature test spec files covering library API contracts.
- **In Scope**: Feature docs 004 (`run_isolated`), 005 (`stdin_file`) and 006 (`unset_claudecode`).
- **Out of Scope**: Builder method edge cases (→ `tests/builder_*_test.rs`), API contracts (→ `tests/docs/api/`).

Per-feature behavioral test specs for `claude_runner_core`. See [docs/feature/readme.md](../../../docs/feature/readme.md) for the feature doc index.

### Responsibility Table

| File | Responsibility |
|------|----------------|
| 004_run_isolated.md | Behavioral cases for `run_isolated()` / `IsolatedModel` feature |
| 005_stdin_file.md | Behavioral cases for `stdin_file` / `with_stdin_file()` feature |
| 006_unset_claudecode.md | Behavioral cases for `unset_claudecode` / `with_unset_claudecode()` feature |

### Index

| Feature | File | Tests | Status |
|---------|------|-------|--------|
| [Run Isolated](../../../docs/feature/004_run_isolated.md) | [004_run_isolated.md](004_run_isolated.md) | 6 FT | ✅ |
| [Stdin File Piping](../../../docs/feature/005_stdin_file.md) | [005_stdin_file.md](005_stdin_file.md) | 8 FT | ✅ |
| [CLAUDECODE Unsetting](../../../docs/feature/006_unset_claudecode.md) | [006_unset_claudecode.md](006_unset_claudecode.md) | 7 FT | ✅ |
