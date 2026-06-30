# Feature Test Surface

### Scope

- **Purpose**: Test case specifications for claude_version feature doc instances.
- **Responsibility**: Per-feature test specs covering acceptance criteria and behavioral verification.
- **In Scope**: Feature test planning (FT- prefix, min 4 cases per spec).
- **Out of Scope**: CLI command tests (-> `cli/command/`), parameter edge cases (-> `cli/param/`).

### Overview Table

| Name | Purpose | Status |
|------|---------|--------|
| 001_version_management.md | FT- test cases for version install/guard/history/aliases | ✅ |
| 002_process_lifecycle.md | FT- test cases for /proc scanning and SIGTERM/SIGKILL | ✅ |
| 003_settings_management.md | FT- test cases for settings.json read/write/type-preservation | ✅ |
| 004_dry_run.md | FT- test cases for dry::1 preview mode across mutation commands | ✅ |
| 005_cli_design.md | FT- test cases for 5-phase unilang pipeline and exit codes | ✅ |
| 006_config_command.md | FT- test cases for `.config` command: show-all/get/set/unset/resolution/catalog | ✅ |
| 007_params_command.md | FT- test cases for `.params` command: show-all/single/kind-filter/format/errors | ✅ |
| procedure.md | Workflow for creating and updating feature test specs | ✅ |
