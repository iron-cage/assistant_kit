# Feature Doc Entity

### Scope

- **Purpose**: Defines the functional capabilities of `claude_profile` — account credential management and the `clp` CLI.
- **Responsibility**: Documents all functional requirements with their design, acceptance criteria, and test references.
- **In Scope**: FR-6 through FR-18; CLI commands and library API surface.
- **Out of Scope**: Quality constraints (→ invariant/), CLI design (→ cli/).

### Overview Table

| ID | Name | FR | Status |
|----|------|----|--------|
| 001 | [Account Store Initialization](001_account_store_init.md) | FR-6 | ✅ |
| 002 | [Save Account](002_account_save.md) | FR-7 | ✅ |
| 003 | [List Accounts](003_account_list.md) | FR-8 | ✅ |
| 004 | [Switch Account](004_account_switch.md) | FR-9 | ✅ |
| 005 | [Delete Account](005_account_delete.md) | FR-10 | ✅ |
| 006 | [Token Status](006_token_status.md) | FR-11 | ✅ |
| 007 | [File Topology](007_file_topology.md) | FR-12 | ✅ |
| 008 | [Auto Rotate](008_auto_rotate.md) | FR-13 | ✅ |
| 009 | [Token Usage Reporting](009_token_usage.md) | FR-14 | ✅ |
| 010 | [Persistent Storage Path](010_persistent_storage.md) | FR-15 | ✅ |
| 011 | [Account Status by Name](011_account_status_by_name.md) | FR-16 | ✅ |
| 012 | [Live Credentials Status](012_live_credentials_status.md) | FR-17 | ✅ |
| 013 | [Account Rate-Limit Utilization](013_account_limits.md) | FR-18 | 🔄 |
| 014 | [Account Rotation Daemon](014_rotation_daemon.md) | FR-19 | 🔄 |
