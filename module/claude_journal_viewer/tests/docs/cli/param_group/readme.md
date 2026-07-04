# Parameter Group Tests

### Scope

- **Purpose**: Test case planning for parameter group doc instances in `docs/cli/param_group/`.
- **Responsibility**: Index of per-group interaction test case spec files covering co-occurrence, precedence, and mutual-exclusivity rules.
- **In Scope**: All 5 `docs/cli/param_group/` doc instances: Filtering, Display, Aggregation, Search, Global.
- **Out of Scope**: Single-parameter edge cases (-> `../param/`), command-level happy paths (-> `../command/`).

Per-group test case indices for `claude_journal_viewer`. See [param_group/readme.md](../../../../docs/cli/param_group/readme.md) for the source doc instances.

### Responsibility Table

| Name | Purpose | Status |
|------|---------|--------|
| `01_filtering.md` | CC- tests for time-window construction and AND-combination | ✅ |
| `02_display.md` | CC- tests for sort/reverse, wide/columns, and limit ordering | ✅ |
| `03_aggregation.md` | CC- tests for `.stats`/`.prune` scoping and dry_run/confirm precedence | ✅ |
| `04_search.md` | CC- tests for required `pattern` and stdout search scope | ✅ |
| `05_global.md` | CC- tests for journal_dir/NO_COLOR resolution order | ✅ |
