# Feature Doc Entity

### Scope

- **Purpose**: Document user-facing capabilities of the claude_version crate.
- **Responsibility**: Index of feature doc instances covering version management, process lifecycle, settings management, dry-run, and CLI design.
- **In Scope**: 12 CLI commands, their parameters, execution modes, and behavioral contracts.
- **Out of Scope**: Version lock design pattern (→ `pattern/`), type inference algorithm (→ `algorithm/`).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [Version Management](001_version_management.md) | Install, guard, history, aliases, hot-swap | ✅ |
| 002 | [Process Lifecycle](002_process_lifecycle.md) | Process listing, SIGTERM/SIGKILL sequence, verification | ✅ |
| 003 | [Settings Management](003_settings_management.md) | Read/write settings.json, type inference, nested preservation | ✅ |
| 004 | [Dry Run](004_dry_run.md) | Mutation preview via dry::1 across all mutation commands | ✅ |
| 005 | [CLI Design](005_cli_design.md) | Command routing, parameter parsing, exit codes, help listing | ✅ |
