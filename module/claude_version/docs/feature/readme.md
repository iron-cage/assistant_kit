# Feature Doc Entity

### Scope

- **Purpose**: Document user-facing capabilities of the claude_version crate.
- **Responsibility**: Index of feature doc instances covering version management, process lifecycle, settings management, dry-run, CLI design, config command, params command, runtime file discovery, and path discovery.
- **In Scope**: All CLI commands, their parameters, execution modes, and behavioral contracts.
- **Out of Scope**: Version lock design pattern (→ `pattern/`), type inference algorithm (→ `algorithm/`).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [Version Management](001_version_management.md) | Install, guard, history, aliases, hot-swap | ✅ |
| 002 | [Process Lifecycle](002_process_lifecycle.md) | Process listing, SIGTERM/SIGKILL sequence, verification | ✅ |
| 003 | [Settings Management](003_settings_management.md) | Read/write settings.json, type inference, nested preservation | ✅ |
| 004 | [Dry Run](004_dry_run.md) | Mutation preview via dry::1 across all mutation commands | ✅ |
| 005 | [CLI Design](005_cli_design.md) | Command routing, parameter parsing, exit codes, help listing | ✅ |
| 006 | [Config Command](006_config_command.md) | Unified `.config` command with 4-layer resolution and catalog | ✅ |
| 007 | [Params Command](007_params_command.md) | `.params` command — full param catalog inspection with observable values | ✅ |
| 008 | [Runtime File Discovery](008_runtime_file_discovery.md) | `.runtime_files` command — enumerate all clv-managed runtime file paths | ✅ |
| 009 | [Path Discovery](009_path_discovery.md) | `.paths` command — labeled path discovery with single-key lookup | ⏳ |
