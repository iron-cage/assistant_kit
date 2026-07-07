# Command Tests

### Scope

- **Purpose**: Document integration test cases for each clv command.
- **Responsibility**: Index of per-command integration test case files covering command-level behavior.
- **In Scope**: All 16 clv command test files.
- **Out of Scope**: Per-parameter edge cases (→ `param/`), parameter group interactions (→ `param_group/`).

### Overview Table

| Name | Purpose | Status |
|------|---------|--------|
| 01_help.md | Integration tests for `.help` command | ✅ |
| 02_status.md | Integration tests for `.status` command | ✅ |
| 03_version_show.md | Integration tests for `.version.show` command | ✅ |
| 04_version_install.md | Integration tests for `.version.install` command | ✅ |
| 05_version_guard.md | Integration tests for `.version.guard` command | ✅ |
| 06_version_list.md | Integration tests for `.version.list` command | ✅ |
| 07_processes.md | Integration tests for `.processes` command | ✅ |
| 08_processes_kill.md | Integration tests for `.processes.kill` command | ✅ |
| 09_settings_show.md | Integration tests for `.settings.show` command | ✅ |
| 10_settings_get.md | Integration tests for `.settings.get` command | ✅ |
| 11_settings_set.md | Integration tests for `.settings.set` command | ✅ |
| 12_version_history.md | Integration tests for `.version.history` command | ✅ |
| 13_config.md | Integration tests for `.config` command (show-all/get/set/unset modes) | ✅ |
| 14_params.md | Integration tests for `.params` command (show-all/single/kind-filter modes) | ✅ |
| 15_runtime_files.md | Integration tests for `.runtime_files` command (path enumeration, HOME, exit codes) | ✅ |
| 16_paths.md | Integration tests for `.paths` command (show-all/single-key/format/verbosity modes) | ✅ |
| procedure.md | Workflow for creating and updating command test specs | ✅ |
