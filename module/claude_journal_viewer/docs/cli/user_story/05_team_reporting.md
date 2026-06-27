# User Story: Team Reporting

**As a** team lead overseeing multiple developers using CLR,
**I want to** generate usage reports across team members,
**so that** I can track team productivity and API costs.

### Persona

Team lead aggregating CLR usage across projects and team members.

### Primary Commands

| # | Command | Role in Story |
|---|---------|---------------|
| 3 | [`.stats`](../command/03_stats.md) | Aggregate stats by model/command |
| 8 | [`.export`](../command/08_export.md) | Export data for external reporting |

### Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC-01 | `clj .stats by::model since::30d` produces per-model cost report |
| AC-02 | `clj .stats by::command since::30d` shows command distribution |
| AC-03 | `clj .export format::csv since::30d output::report.csv` exports for spreadsheets |
| AC-04 | `clj .list format::json since::30d limit::0` exports all events as JSON |
| AC-05 | Reports include total cost, total tokens, success rate |
| AC-06 | `clj .list journal_dir::/shared/journal/ since::30d` reads team journal |

### Workflow

```bash
# Monthly report: costs by model
clj .stats by::model since::30d

# Command usage breakdown
clj .stats by::command since::30d

# Export for spreadsheet
clj .export format::csv since::30d output::~/monthly_report.csv

# Team journal (shared location)
CLR_JOURNAL_DIR=/shared/team_journal clj .stats by::day since::30d
```

### Referenced Parameters

| # | Parameter | Usage |
|---|-----------|-------|
| 10 | [`format`](../param/10_format.md) | CSV export for spreadsheets |
| 13 | [`by`](../param/13_by.md) | Group by model/command |
| 21 | [`journal_dir`](../param/21_journal_dir.md) | Read team journal |
| 23 | [`output`](../param/23_output.md) | Export file path |
