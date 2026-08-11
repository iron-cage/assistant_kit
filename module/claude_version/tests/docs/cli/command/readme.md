# Command Tests

### Scope

- **Purpose**: Document integration test cases for each clv command.
- **Responsibility**: Index of per-command integration test case files covering command-level behavior.
- **In Scope**: All 16 clv command test files (07_ps.md and 08_ps_kill.md replace the former 07_processes.md and 08_processes_kill.md).
- **Out of Scope**: Per-parameter edge cases (→ `param/`), parameter group interactions (→ `param_group/`).

### Overview Table

| Name | Purpose | Status |
|------|---------|--------|
| 01_help.md | Integration tests for `.help` command | ✅ |
| 02_status.md | Integration tests for `.status` command | ✅ |
| 03_version_show.md | Integration tests for `.version.show` command | ✅ |
| 04_version_install.md | Integration tests for `.version.install` command | ✅ |
| 05_version_guard.md | Integration tests for `.version.guard` command | ✅ |
| 06_version_list.md | Integration tests for `.version.list` command (alias listing + release history via `mode::`) | ✅ |
| 07_ps.md | Integration tests for `.ps` command | ✅ |
| 08_ps_kill.md | Integration tests for `.ps.kill` command | ✅ |
| 09_settings_show.md | Integration tests for `.settings.show` command | ✅ |
| 10_settings_get.md | Integration tests for `.settings.get` command | ✅ |
| 11_settings_set.md | Integration tests for `.settings.set` command | ✅ |
| 13_config.md | Integration tests for `.config` command (show-all/get/set/unset modes) | ✅ |
| 14_params.md | Integration tests for `.params` command (show-all/single/kind-filter modes) | ✅ |
| 15_runtime_files.md | Integration tests for `.runtime_files` command (path enumeration, HOME, exit codes) | ✅ |
| 16_version_paths.md | Integration tests for `.version.paths` command (show-all/single-key/format/verbosity modes) | ✅ |
| 17_version_mark.md | Integration tests for `.version.mark` command (CRUD, validation, dry-run, format) | ✅ |
| procedure.md | Workflow for creating and updating command test specs | ✅ |
