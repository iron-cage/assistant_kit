# Doc Entities

### Scope

- **Purpose**: Index of doc entity directories for `claude_version`.
- **Responsibility**: Master entity registry for all behavioral requirement doc entities under `docs/`.
- **In Scope**: Behavioral entities (`algorithm/`, `feature/`, `pattern/`).
- **Out of Scope**: Test doc entities (→ `tests/doc/entities.md`).

## Master Doc Entities Table

| Type | Purpose | Master File | Instances |
|------|---------|-------------|----------:|
| `algorithm/` | Index of algorithm doc instances covering settings type inference. | [algorithm/readme.md](algorithm/readme.md) | 1 |
| `feature/` | Index of feature doc instances covering version management, process lifecycle, settings management, dry-run, and CLI design. | [feature/readme.md](feature/readme.md) | 5 |
| `pattern/` | Index of pattern doc instances covering version lock strategy. | [pattern/readme.md](pattern/readme.md) | 1 |

## Master Doc Instances Table

| Entity | ID | Name | File |
|--------|----|------|------|
| algorithm | 001 | Settings Type Inference | [algorithm/001_settings_type_inference.md](algorithm/001_settings_type_inference.md) |
| feature | 001 | Version Management | [feature/001_version_management.md](feature/001_version_management.md) |
| feature | 002 | Process Lifecycle | [feature/002_process_lifecycle.md](feature/002_process_lifecycle.md) |
| feature | 003 | Settings Management | [feature/003_settings_management.md](feature/003_settings_management.md) |
| feature | 004 | Dry Run | [feature/004_dry_run.md](feature/004_dry_run.md) |
| feature | 005 | CLI Design | [feature/005_cli_design.md](feature/005_cli_design.md) |
| pattern | 001 | Version Lock | [pattern/001_version_lock.md](pattern/001_version_lock.md) |
