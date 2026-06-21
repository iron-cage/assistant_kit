# Feature Test Surface

### Scope

- **Purpose**: Test case specifications for claude_version feature doc instances.
- **Responsibility**: Per-feature test specs covering acceptance criteria and behavioral verification.
- **In Scope**: Feature test planning (FT- prefix, min 4 cases per spec).
- **Out of Scope**: CLI command tests (-> `cli/command/`), parameter edge cases (-> `cli/param/`).

### Overview Table

| Name | Purpose | Status |
|------|---------|--------|
| 01_version_management.md | FT- test cases for version install/guard/history/aliases | ✅ |
| 02_process_lifecycle.md | FT- test cases for /proc scanning and SIGTERM/SIGKILL | ✅ |
| 03_settings_management.md | FT- test cases for settings.json read/write/type-preservation | ✅ |
| 04_dry_run.md | FT- test cases for dry::1 preview mode across mutation commands | ✅ |
| 05_cli_design.md | FT- test cases for 5-phase unilang pipeline and exit codes | ✅ |
| 06_config_command.md | FT- test cases for `.config` command: show-all/get/set/unset/resolution/catalog | ✅ |
| 07_params_command.md | FT- test cases for `.params` command: show-all/single/kind-filter/format/errors | ⏳ |
| procedure.md | Workflow for creating and updating feature test specs | ✅ |
