# Command Tests

### Scope

- **Purpose**: Test case planning for command doc instances in `docs/cli/command/`.
- **Responsibility**: Index of per-command test case spec files covering parameter filtering, sorting, formatting, and error handling.
- **In Scope**: All 9 `docs/cli/command/` doc instances: list, tail, stats, search, serve, prune, status, export, chart.
- **Out of Scope**: Parameter-level edge cases (-> `../param/`), end-to-end persona workflows (-> `../user_story/`).

Per-command test case indices for `claude_journal_viewer`. See [command/readme.md](../../../../docs/cli/command/readme.md) for the source doc instances.

### Responsibility Table

| Name | Purpose | Status |
|------|---------|--------|
| [01_list.md](01_list.md) | IT- tests for `.list` filtering, sorting, and formatting | ✅ |
| [02_tail.md](02_tail.md) | IT- tests for `.tail` real-time following and filters | ✅ |
| [03_stats.md](03_stats.md) | IT- tests for `.stats` aggregation and grouping | ✅ |
| [04_search.md](04_search.md) | IT- tests for `.search` substring matching and filters | ✅ |
| [05_serve.md](05_serve.md) | IT- tests for `.serve` web viewer startup and options | ✅ |
| [06_prune.md](06_prune.md) | IT- tests for `.prune` retention deletion and gates | ✅ |
| [07_status.md](07_status.md) | IT- tests for `.status` health reporting at each verbosity | ✅ |
| [08_export.md](08_export.md) | IT- tests for `.export` format selection and uncapped output | ✅ |
| [09_chart.md](09_chart.md) | IT- tests for `.chart` output path, empty journal, and error paths | ✅ |
