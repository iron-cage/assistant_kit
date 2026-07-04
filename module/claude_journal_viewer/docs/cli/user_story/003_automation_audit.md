# User Story: Automation Audit

**As a** developer running automated CLR pipelines,
**I want to** audit all invocations for compliance and correctness,
**so that** I can verify automation behavior and maintain quality.

### Persona

Developer maintaining CI/CD pipelines or cron-based CLR automation.

### Primary Commands

| # | Command | Role in Story |
|---|---------|---------------|
| 1 | [`.list`](../command/01_list.md) | Review invocation history |
| 4 | [`.search`](../command/04_search.md) | Find specific patterns |
| 8 | [`.export`](../command/08_export.md) | Export for external analysis |

### Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC-01 | `clj .list command::ask since::7d` shows all automated ask invocations |
| AC-02 | `clj .list dir::/ci/project since::1d` filters by project directory |
| AC-03 | `clj .list creds::automation.json` shows all runs with specific credentials |
| AC-04 | `clj .export format::csv since::7d output::/tmp/audit.csv` produces audit trail |
| AC-05 | `clj .search pattern::"unexpected" include_stdout::1 since::7d` finds anomalies |
| AC-06 | `clj .serve` dashboard allows visual inspection of automation patterns |
| AC-07 | Export includes all event fields for forensic analysis |

### Workflow

```bash
# Daily: audit check
clj .list since::1d command::ask

# Credential review
clj .list creds::automation.json since::7d

# Export for compliance
clj .export format::csv since::30d output::~/audit_report.csv

# Anomaly detection
clj .search pattern::"(?i)(warn|unexpected|failed)" include_stdout::1 since::7d
```

### Referenced Parameters

| # | Parameter | Usage |
|---|-----------|-------|
| 04 | [`command`](../param/04_command.md) | Filter by CLR command |
| 07 | [`dir`](../param/07_dir.md) | Filter by project directory |
| 08 | [`creds`](../param/08_creds.md) | Filter by credential file |
| 10 | [`format`](../param/10_format.md) | Export format |
| 23 | [`output`](../param/23_output.md) | Export file path |
