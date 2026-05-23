# Command Tests

### Scope

- **Purpose**: Document integration test cases for each cm command.
- **Responsibility**: Index of per-command integration test case files covering command-level behavior.
- **In Scope**: All 12 cm command test files.
- **Out of Scope**: Per-parameter edge cases (→ `param/`), parameter group interactions (→ `param_group/`).

### Overview Table

| Name | Purpose | Status |
|------|---------|--------|
| 001_help.md | Integration tests for `.help` command | ✅ |
| 002_status.md | Integration tests for `.status` command | ✅ |
| 003_version_show.md | Integration tests for `.version.show` command | ✅ |
| 004_version_install.md | Integration tests for `.version.install` command | ✅ |
| 005_version_guard.md | Integration tests for `.version.guard` command | ✅ |
| 006_version_list.md | Integration tests for `.version.list` command | ✅ |
| 007_processes.md | Integration tests for `.processes` command | ✅ |
| 008_processes_kill.md | Integration tests for `.processes.kill` command | ✅ |
| 009_settings_show.md | Integration tests for `.settings.show` command | ✅ |
| 010_settings_get.md | Integration tests for `.settings.get` command | ✅ |
| 011_settings_set.md | Integration tests for `.settings.set` command | ✅ |
| 012_version_history.md | Integration tests for `.version.history` command | ✅ |
| procedure.md | Workflow for creating and updating command test specs | ✅ |
