# Invariant Collection

### Scope

- **Purpose**: Defines the non-functional constraints that `claude_profile` must maintain at all times.
- **Responsibility**: Documents all quality invariants with their statements, enforcement mechanisms, and violation consequences.
- **In Scope**: invariant/001 through invariant/008 — all non-functional constraints and architectural guarantees for claude_profile.
- **Out of Scope**: Functional requirements (→ feature/), CLI design constraints (→ cli/).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [Zero Third-Party Dependencies](001_zero_third_party_deps.md) | Library path must have zero third-party crates.io dependencies | ✅ |
| 002 | [Cross-Platform Compatibility](002_cross_platform.md) | All path operations work correctly on Linux, macOS, and Windows | ✅ |
| 003 | [Clear Error Messages](003_clear_errors.md) | All errors name the relevant resource and state corrective action | ✅ |
| 004 | [No Process Execution](004_no_process_execution.md) | `std::process::Command` is forbidden anywhere in the library | ✅ |
| 005 | [Atomic Account Switching](005_atomic_switching.md) | Account switching uses write-then-rename to prevent credential corruption | ✅ |
| 006 | [Parameters Default to Active Context](006_param_defaults.md) | Every parameter must have a default unless requiring an explicit value is absolutely necessary | ✅ |
| 007 | [JSON Storage Format](007_json_storage_format.md) | All `.json` files written to disk use `serde_json::to_string_pretty` + trailing `\n` | ✅ |
| 008 | [Single Token Refresh Entry Point](008_single_token_refresh_entry.md) | All token refresh goes through `refresh_account_token()`; no direct `run_isolated()` calls | ✅ |
