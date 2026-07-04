# Parameter Tests

### Scope

- **Purpose**: Test case planning for CLI parameter doc instances in `docs/cli/param/`.
- **Responsibility**: Index of per-parameter edge case spec files covering default/absence behavior and single-parameter parsing.
- **In Scope**: All 28 `docs/cli/param/` doc instances.
- **Out of Scope**: Group interaction rules (-> `../param_group/`), type-level validation (-> `../type/`).

Per-parameter edge case test indices for `claude_journal_viewer`. See [param/readme.md](../../../../docs/cli/param/readme.md) for the source doc instances.

### Responsibility Table

| Name | Purpose | Status |
|------|---------|--------|
| `01_since.md` | EC- tests for absence (no lower bound) and `.stats` default variance | ✅ |
| `02_until.md` | EC- tests for absence (no upper bound) and combination with `since` | ✅ |
| `03_type.md` | EC- tests for absence (all types) and `.stats` default variance | ✅ |
| `04_command.md` | EC- tests for absence and the exact-match constraint | ✅ |
| `05_exit.md` | EC- tests for absence and specific error-class filtering | ✅ |
| `06_model.md` | EC- tests for absence and substring matching | ✅ |
| `07_dir.md` | EC- tests for absence and subdirectory substring matching | ✅ |
| `08_creds.md` | EC- tests for absence and exclusion of events with no creds field | ✅ |
| `09_limit.md` | EC- tests for the default cap and the unlimited shortcut | ✅ |
| `10_format.md` | EC- tests for per-command default variance | ✅ |
| `11_sort.md` | EC- tests for the default field and combination with `reverse` | ✅ |
| `12_reverse.md` | EC- tests for default ascending and reversed descending order | ✅ |
| `13_by.md` | EC- tests for the default grouping and combination with a time filter | ✅ |
| `14_pattern.md` | EC- tests for the required-parameter constraint and regex matching | ✅ |
| `15_port.md` | EC- tests for the default port and the ephemeral shortcut | ✅ |
| `16_bind.md` | EC- tests for localhost-only default and network-accessible binding | ✅ |
| `17_open.md` | EC- tests for default (no auto-open) and the auto-open shortcut | ✅ |
| `18_keep.md` | EC- tests for the required-parameter constraint and both retention modes | ✅ |
| `19_dry_run.md` | EC- tests for default (live deletion) and the preview mode | ✅ |
| `20_confirm.md` | EC- tests for default interactive prompt and the skip-prompt shortcut | ✅ |
| `21_journal_dir.md` | EC- tests for the 3-level resolution order | ✅ |
| `22_verbosity.md` | EC- tests for the default level and per-command meaning | ✅ |
| `23_output.md` | EC- tests for default (stdout) and writing to a file | ✅ |
| `24_no_color.md` | EC- tests for default, explicit disable, and `NO_COLOR` env var | ✅ |
| `25_wide.md` | EC- tests for default compact table and full-width mode | ✅ |
| `26_columns.md` | EC- tests for the default column set and a custom selection | ✅ |
| `27_refresh.md` | EC- tests for default interval, disable shortcut, and custom interval | ✅ |
| `28_include_stdout.md` | EC- tests for default (message-only) and extended search scope | ✅ |
