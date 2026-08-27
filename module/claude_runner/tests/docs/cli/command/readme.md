# Command Tests

### Scope

- **Purpose**: Document integration test cases for each clr command.
- **Responsibility**: Index of per-command integration test case files covering command-level behavior.
- **In Scope**: `run` command tests, `help` command tests, `isolated` command tests, `refresh` command tests, `ask` command tests, `ps` command tests, `kill` command tests, `tools` command tests, `scope` command tests, `query` command tests, `topic` command tests.
- **Out of Scope**: Per-parameter edge cases (→ `param/`), parameter group interactions (→ `param_group/`).

Per-command integration test case indices for `clr`. See [command/](../../../../docs/cli/command/) for specification.

### Responsibility Table

| Name | Purpose | Status |
|------|---------|--------|
| [01_run.md](01_run.md) | Integration tests for the `run` command (default) | ✅ |
| [02_help.md](02_help.md) | Integration tests for the `help` command (`--help` / `-h`) | ✅ |
| [03_isolated.md](03_isolated.md) | Integration tests for the `isolated` command | ✅ |
| [04_refresh.md](04_refresh.md) | Integration tests for the `refresh` command | ✅ |
| [05_ask.md](05_ask.md) | Integration tests for the `ask` command | ✅ |
| [06_ps.md](06_ps.md) | Integration tests for the `ps` command | ✅ |
| [07_kill.md](07_kill.md) | Integration tests for the `kill` command | ✅ |
| [08_tools.md](08_tools.md) | Integration tests for the `tools` command | ✅ |
| [09_scope.md](09_scope.md) | Integration tests for the `scope` command | ✅ |
| [10_query.md](10_query.md) | Integration tests for the `query` command | ✅ |
| [11_topic.md](11_topic.md) | Integration tests for the `topic` command | ✅ |
| [12_topics.md](12_topics.md) | Integration tests for the `topics` command | ✅ |
