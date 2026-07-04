# Feature Doc Entity

### Scope

**Responsibilities:** Behavioral requirements for the `claude_journal_viewer` crate.
**In Scope:** CLI event viewing, web-based dashboard, query filtering and aggregation.
**Out of Scope:** Journal write path (-> `claude_journal`), CLR integration (-> `claude_runner`).

### Responsibility Table

| # | File | Responsibility |
|---|------|----------------|
| 001 | `001_cli_viewing.md` | CLI commands for listing, searching, and analyzing journal events |
| 002 | `002_web_viewing.md` | Embedded HTTP server with single-page web dashboard |
| 003 | `003_filtering.md` | Query filter system: time ranges, type, command, model, exit code |
