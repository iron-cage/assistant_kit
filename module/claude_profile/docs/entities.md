# Doc Entities

### Scope

- **Purpose**: Index of doc entity directories for `claude_profile`.
- **Responsibility**: Master entity registry for all behavioral requirement doc entities under `docs/`.
- **In Scope**: Behavioral entities (`feature/`, `invariant/`).
- **Out of Scope**: Test doc entities (→ `tests/doc/entities.md`).

## Master Doc Entities Table

| Type | Purpose | Master File | Instances |
|------|---------|-------------|----------:|
| `feature/` | Documents all functional requirements with their design, acceptance criteria, and test references. | [feature/readme.md](feature/readme.md) | 13 |
| `invariant/` | Documents all quality invariants with their statements, enforcement mechanisms, and violation consequences. | [invariant/readme.md](invariant/readme.md) | 5 |

## Master Doc Instances Table

| Entity | ID | Name | File |
|--------|----|------|------|
| feature | 001 | Account Store Initialization | [feature/001_account_store_init.md](feature/001_account_store_init.md) |
| feature | 002 | Save Account | [feature/002_account_save.md](feature/002_account_save.md) |
| feature | 003 | List Accounts | [feature/003_account_list.md](feature/003_account_list.md) |
| feature | 004 | Switch Account | [feature/004_account_switch.md](feature/004_account_switch.md) |
| feature | 005 | Delete Account | [feature/005_account_delete.md](feature/005_account_delete.md) |
| feature | 006 | Token Status | [feature/006_token_status.md](feature/006_token_status.md) |
| feature | 007 | File Topology | [feature/007_file_topology.md](feature/007_file_topology.md) |
| feature | 008 | Auto Rotate | [feature/008_auto_rotate.md](feature/008_auto_rotate.md) |
| feature | 009 | Token Usage Reporting | [feature/009_token_usage.md](feature/009_token_usage.md) |
| feature | 010 | Persistent Storage Path | [feature/010_persistent_storage.md](feature/010_persistent_storage.md) |
| feature | 011 | Account Status by Name | [feature/011_account_status_by_name.md](feature/011_account_status_by_name.md) |
| feature | 012 | Live Credentials Status | [feature/012_live_credentials_status.md](feature/012_live_credentials_status.md) |
| feature | 013 | Account Rate-Limit Utilization | [feature/013_account_limits.md](feature/013_account_limits.md) |
| invariant | 001 | Zero Third-Party Dependencies | [invariant/001_zero_third_party_deps.md](invariant/001_zero_third_party_deps.md) |
| invariant | 002 | Cross-Platform Compatibility | [invariant/002_cross_platform.md](invariant/002_cross_platform.md) |
| invariant | 003 | Clear Error Messages | [invariant/003_clear_errors.md](invariant/003_clear_errors.md) |
| invariant | 004 | No Process Execution | [invariant/004_no_process_execution.md](invariant/004_no_process_execution.md) |
| invariant | 005 | Atomic Account Switching | [invariant/005_atomic_switching.md](invariant/005_atomic_switching.md) |
