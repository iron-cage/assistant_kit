# Doc Entities

Master index of all documentation entities and their instances for `claude_profile`.

### Doc Entity Registry

| Entity | Directory | Count | Responsibility |
|--------|-----------|-------|----------------|
| Feature | [feature/](feature/readme.md) | 18 | Functional requirements (FR-6 – FR-22) |
| Invariant | [invariant/](invariant/readme.md) | 5 | Non-functional constraints (NFR-1, NFR-3 – NFR-6) |
| CLI Design | [cli/](cli/readme.md) | 7 | Command, parameter, and workflow reference |

---

### Feature Doc Instances

| ID | File | Req | Summary |
|----|------|-----|---------|
| 001 | [feature/001_account_store_init.md](feature/001_account_store_init.md) | FR-6 | Account store initialization and path resolution |
| 002 | [feature/002_account_save.md](feature/002_account_save.md) | FR-7 | Save current credentials as a named account profile |
| 003 | [feature/003_account_list.md](feature/003_account_list.md) | FR-8 | List accounts with field-presence control (`.accounts`) |
| 004 | [feature/004_account_use.md](feature/004_account_use.md) | FR-9 | Atomic credential rotation to a named account |
| 005 | [feature/005_account_delete.md](feature/005_account_delete.md) | FR-10 | Delete account with active-account safety guard |
| 006 | [feature/006_token_status.md](feature/006_token_status.md) | FR-11 | OAuth token expiry classification |
| 007 | [feature/007_file_topology.md](feature/007_file_topology.md) | FR-12 | Canonical `~/.claude/` path topology |
| 008 | [feature/008_auto_rotate.md](feature/008_auto_rotate.md) | FR-13 | Auto-rotate to the account with the highest token expiry |
| 009 | [feature/009_token_usage.md](feature/009_token_usage.md) | FR-14 | All-accounts live quota reporting |
| 010 | [feature/010_persistent_storage.md](feature/010_persistent_storage.md) | FR-15 | Persistent storage path resolution from `$PRO` / `$HOME` |
| 011 | [feature/011_account_status_by_name.md](feature/011_account_status_by_name.md) | FR-16 | Named account scoping for `.accounts name::` |
| 012 | [feature/012_live_credentials_status.md](feature/012_live_credentials_status.md) | FR-17 | Live credentials status without account store dependency |
| 013 | [feature/013_account_limits.md](feature/013_account_limits.md) | FR-18 | Rate-limit utilization via live HTTP response headers |
| 014 | [feature/014_rich_account_metadata.md](feature/014_rich_account_metadata.md) | FR-20 | Rich OAuth metadata fields on `.credentials.status` and `.accounts` |
| 015 | [feature/015_name_shortcut_syntax.md](feature/015_name_shortcut_syntax.md) | FR-21 | Positional bare arg and prefix resolution for `name::` on four account commands |
| 016 | [feature/016_current_account_awareness.md](feature/016_current_account_awareness.md) | FR-22 | Current account detection via token match; divergence display in `.accounts` and `.usage` |
| 017 | [feature/017_token_refresh.md](feature/017_token_refresh.md) | — | `refresh::` param; retry-on-auth-error via `run_isolated()`; credential write-back |
| 018 | [feature/018_live_monitor.md](feature/018_live_monitor.md) | — | `live::`, `interval::`, `jitter::` params; continuous refresh with staggered fetches |

---

### Invariant Doc Instances

| ID | File | Req | Summary |
|----|------|-----|---------|
| 001 | [invariant/001_zero_third_party_deps.md](invariant/001_zero_third_party_deps.md) | NFR-1 | Library path has zero third-party crates.io dependencies |
| 002 | [invariant/002_cross_platform.md](invariant/002_cross_platform.md) | NFR-3 | All path operations work correctly on Linux, macOS, and Windows |
| 003 | [invariant/003_clear_errors.md](invariant/003_clear_errors.md) | NFR-4 | All errors name the relevant resource and state a corrective action |
| 004 | [invariant/004_no_process_execution.md](invariant/004_no_process_execution.md) | NFR-5 | `std::process::Command` is forbidden in the library path |
| 005 | [invariant/005_atomic_switching.md](invariant/005_atomic_switching.md) | NFR-6 | Account switching uses write-then-rename to prevent corruption |

---

### CLI Design Doc Instances

| File | Responsibility |
|------|----------------|
| [cli/commands.md](cli/commands.md) | Per-command specification for all 11 commands (9 visible + 2 hidden) |
| [cli/params.md](cli/params.md) | Per-parameter specification (name, format, dry, field-presence) |
| [cli/parameter_groups.md](cli/parameter_groups.md) | Parameter grouping, shared behavior, and output-control groups |
| [cli/parameter_interactions.md](cli/parameter_interactions.md) | Cross-parameter interaction rules |
| [cli/types.md](cli/types.md) | Input type definitions and validation rules |
| [cli/workflows.md](cli/workflows.md) | End-to-end workflow examples |
| [cli/dictionary.md](cli/dictionary.md) | Canonical domain vocabulary |
