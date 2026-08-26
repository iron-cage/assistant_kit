# Parameter Tests

### Scope

- **Purpose**: Test case planning for CLI parameter doc instances in `docs/cli/param/`.
- **Responsibility**: Index of per-parameter edge case spec files covering default/absence behavior and single-parameter parsing.
- **In Scope**: All 26 `docs/cli/param/` doc instances — 25 live parameters plus one tombstone. Numbering runs 01-29 with gaps at `20`, `25`, and `26`, none of which are files; see [param/readme.md](../../../../docs/cli/param/readme.md) for what each gap was.
- **Out of Scope**: Group interaction rules (-> `../param_group/`), type-level validation (-> `../type/`).

Per-parameter edge case test indices for `claude_journal_viewer`. See [param/readme.md](../../../../docs/cli/param/readme.md) for the source doc instances.

### Responsibility Table

| Name | Purpose | Status |
|------|---------|--------|
| [01_since.md](01_since.md) | EC- tests for absence (no lower bound) and `.stats` default variance | ✅ |
| [02_until.md](02_until.md) | EC- tests for absence (no upper bound) and combination with `since` | ✅ |
| [03_type.md](03_type.md) | EC- tests for absence (all types) and `.stats` default variance | ✅ |
| [04_command.md](04_command.md) | EC- tests for absence and the exact-match constraint | ✅ |
| [05_exit.md](05_exit.md) | EC- tests for absence and specific error-class filtering | ✅ |
| [06_model.md](06_model.md) | EC- tests for absence and substring matching | ✅ |
| [07_dir.md](07_dir.md) | EC- tests for absence and subdirectory substring matching | ✅ |
| [08_creds.md](08_creds.md) | EC- tests for absence and exclusion of events with no creds field | ✅ |
| [09_limit.md](09_limit.md) | EC- tests for the default cap and the unlimited shortcut | ✅ |
| [10_format.md](10_format.md) | EC- tests for per-command default variance | ✅ |
| [11_sort.md](11_sort.md) | EC- tests for the default field and combination with `reverse` | ✅ |
| [12_reverse.md](12_reverse.md) | EC- tests for default ascending and reversed descending order | ✅ |
| [13_by.md](13_by.md) | EC- tests for the default grouping and combination with a time filter | ✅ |
| [14_pattern.md](14_pattern.md) | EC- tests for the required-parameter rule and literal substring matching | ✅ |
| [15_port.md](15_port.md) | EC- tests for the default port and the ephemeral shortcut | ✅ |
| [16_bind.md](16_bind.md) | EC- tests for the default loopback bind, the honored override, and bind failure | ✅ |
| [17_open.md](17_open.md) | EC- tests for default (no auto-open) and the auto-open shortcut | ✅ |
| [18_keep.md](18_keep.md) | EC- tests for the `30d` default and age-based duration parsing | ✅ |
| [19_dry_run.md](19_dry_run.md) | EC- tests for default (live deletion) and the preview mode | ✅ |
| [21_journal_dir.md](21_journal_dir.md) | EC- tests for the 3-level resolution order | ✅ |
| [22_verbosity.md](22_verbosity.md) | EC- tests for the three `.status` levels, clamping, and rejection | ✅ |
| [23_output.md](23_output.md) | EC- tests for the required-param rule and writing to a file | ✅ |
| [24_no_color.md](24_no_color.md) | EC- tests for default, explicit disable, and `NO_COLOR` env var | ✅ |
| [27_refresh.md](27_refresh.md) | EC- tests for default interval, disable shortcut, and custom interval | ✅ |
| [28_include_stdout.md](28_include_stdout.md) | EC- tests for unconditional output search and the flag's rejection | ✅ |
| [29_out.md](29_out.md) | EC- tests for `.chart`'s default path, the override, and `out` vs `output` | ✅ |
