# Feature Tests

### Scope

- **Purpose**: Document behavioral requirement cases for `claude_runner_core` feature doc instances.
- **Responsibility**: Index of per-feature test spec files covering library API contracts.
- **In Scope**: Feature docs 005 (`stdin_file`) and 006 (`unset_claudecode`).
- **Out of Scope**: Builder method edge cases (→ `tests/builder_*_test.rs`), API contracts (→ `tests/docs/api/`).

Per-feature behavioral test specs for `claude_runner_core`. See [docs/feature/readme.md](../../../docs/feature/readme.md) for the feature doc index.

### Responsibility Table

| File | Responsibility |
|------|----------------|
| 005_stdin_file.md | Behavioral cases for `stdin_file` / `with_stdin_file()` feature |
| 006_unset_claudecode.md | Behavioral cases for `unset_claudecode` / `with_unset_claudecode()` feature |

### Index

| Feature | File | Tests | Status |
|---------|------|-------|--------|
| [Stdin File Piping](../../../docs/feature/005_stdin_file.md) | [005_stdin_file.md](005_stdin_file.md) | 5 FT | ⏳ |
| [CLAUDECODE Unsetting](../../../docs/feature/006_unset_claudecode.md) | [006_unset_claudecode.md](006_unset_claudecode.md) | 5 FT | ⏳ |
