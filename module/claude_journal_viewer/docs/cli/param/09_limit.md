# CLI Parameter: limit

Maximum number of events to display or return. Acts as a cap
after filtering and sorting are applied. Set to `0` for unlimited.

The default is **per command, not global**: only `.list` applies one. `.stats`,
`.search`, and `.export` leave `limit` unset when it is absent, so an unfiltered
run of any of them reads the whole journal. Verify with a journal holding more
than 50 events:

```bash
clj .list   journal_dir::<dir> format::jsonl | wc -l   # 50 — the cap
clj .stats  journal_dir::<dir> | tail -1               # Total: <all of them>
clj .search journal_dir::<dir> pattern::<common> | tail -1   # N match(es), N > 50
```

`.tail` does not accept `limit` at all — it follows the journal forward with no
end to stop at, so a cap there would parse and then apply to nothing.

- **Type:** [`Integer`](../type/04_integer.md)
- **Default:** 50 on `.list`; unset (unlimited) on `.stats`, `.search`, `.export`
- **Required:** No

```bash
clj .list                             # Default: 50 events
clj .list limit::100                  # Up to 100 events
clj .list limit::0                    # All matching events (no cap)
clj .search pattern::"error" limit::10  # First 10 events searched, not 10 matches
```

### Referenced Type

| Type | Kind | Fundamental | Key Constraint |
|------|------|-------------|----------------|
| [`Integer`](../type/04_integer.md) | Fundamental | Integer | Non-negative; 0 = unlimited |

### Referenced Parameter Groups

| # | Group | Membership |
|---|-------|------------|
| 2 | [Display](../param_group/02_display.md) | Full |

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 1 | [`.list`](../command/01_list.md) | 50 | Applied after `sort`/`reverse` |
| 3 | [`.stats`](../command/03_stats.md) | -- | Caps the events aggregated, not the rows reported |
| 4 | [`.search`](../command/04_search.md) | -- | Caps the events searched, before `pattern` is applied |
| 8 | [`.export`](../command/08_export.md) | -- | Uncapped export writes the whole journal |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 1 | [Cost Tracking](../user_story/001_cost_tracking.md) | Developer |
| 4 | [Capacity Planning](../user_story/004_capacity_planning.md) | Developer |
