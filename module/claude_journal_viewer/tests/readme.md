# tests/

### Scope

**Responsibilities:** Automated integration tests for the `claude_journal_viewer` crate — the `clj` binary (`.list`/`.tail`/`.stats`/`.search`/`.serve`/`.prune`/`.status`/`.export`) and its optional `routines` feature (`register_commands()` for `ast .journal.*` unilang integration).
**In Scope:** All crate functionality exercised via the compiled `clj` binary and the public library API.
**Out of Scope:** Manual testing, test planning documents (→ `docs/`).

### Domain Map

| Domain | File | Tests What |
|--------|------|------------|
| Library API (TC-001) | `lib_test.rs` | `register_commands()` is callable and leaves the registry unchanged |
| `clj` binary (EC-1–EC-13) | `viewer_integration_test.rs` | `.list` table/JSON output and `type`/`since` validation, `.stats by::model` aggregation, `.search pattern::` filtering, `.prune dry_run::1`, `.status` health report, `.export format::json`, parse-time type validation, `NO_COLOR=1` ANSI suppression, `.serve` HTTP GET `/`, `.tail` blocking behavior |
