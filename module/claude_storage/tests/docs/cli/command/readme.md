# Command Tests

### Scope

- **Purpose**: Document integration test cases for each clg command.
- **Responsibility**: Index of per-command integration test case files covering command-level behavior.
- **In Scope**: All 13 clg command test files.
- **Out of Scope**: Per-parameter edge cases (→ `param/`), parameter group interactions (→ `param_group/`).

### Responsibility Table

| Name | Purpose | Status |
|------|---------|--------|
| 01_status.md | Integration tests for `.status` command | ✅ |
| 02_list.md | Integration tests for `.list` command (DEPRECATED, superseded by `.projects`) | ✅ |
| 03_show.md | Integration tests for `.show` command | ✅ |
| 04_count.md | Integration tests for `.count` command | ✅ |
| 05_search.md | Integration tests for `.search` command | ✅ |
| 06_export.md | Integration tests for `.export` command | ✅ |
| 07_projects.md | Integration tests for `.projects` command | ✅ |
| 08_project_path.md | Integration tests for `.project.path` command | ✅ |
| 09_project_exists.md | Integration tests for `.project.exists` command | ✅ |
| 10_session_dir.md | Integration tests for `.session.dir` command | ✅ |
| 11_session_ensure.md | Integration tests for `.session.ensure` command | ✅ |
| 12_tail.md | Integration tests for `.tail` command | ✅ |
| 13_usage.md | Integration tests for `.usage` command | ✅ |

### Cross-Command Dispatch Coverage

Not every test file maps to exactly one command row above. `tests/command_help_space_form_test.rs` covers the space-separated `<command> help` interception (`BUG-005`) — a pre-dispatch mechanism in `src/cli_main.rs` applying uniformly to every registered command, exercised here against `.list`/`.show`/`.search` as representative cases (T01-T10). It has no dedicated row since its subject is dispatch-level, not any single command's own behavior; see `docs/feature/001_cli_tool.md`'s Help rendering paragraph for the cross-cutting behavioral spec this file tests against.
