# tests/

### Scope

**Responsibilities:** Automated integration tests for the `claude_journal_viewer` crate — the `clj` binary (`.list`/`.tail`/`.stats`/`.search`/`.serve`/`.prune`/`.status`/`.export`/`.chart`) and its optional `routines` feature (`register_commands()` for `ast .journal.*` unilang integration).
**In Scope:** All crate functionality exercised via the compiled `clj` binary and the public library API.
**Out of Scope:** Manual testing, test planning documents (→ `docs/`).

### Domain Map

| Domain | File | Tests What |
|--------|------|------------|
| Library API (TC-001) | `lib_test.rs` | `register_commands()` is callable and leaves the registry unchanged |
| `clj` binary (EC-1–EC-34) | `viewer_integration_test.rs` | `.list` table/JSON output and `type`/`since` validation, `.stats by::model` aggregation and `by::dir`/`by::agent` count-ranked grouping with `(no dir)`/`(no agent)` buckets and invalid-`by` validation (EC-21–EC-24, task 543), `.search pattern::` filtering, `.prune dry_run::1` and filename-date prune semantics (old dated file deleted; non-date `.jsonl` and today's file survive), `.status` health report, `.export format::json`, parse-time type validation, `NO_COLOR=1` ANSI suppression, `.tail` blocking behavior, `.chart` rendering, empty-`HOME` journal resolution (EC-25), and parameter-name semantics (EC-26–EC-29): `exit::` and `dir::` filter what their docs say they filter, an unrecognised key exits 1 instead of being ignored, and `no_color::1` matches the env var; and `.list sort::`/`reverse::`/`limit::` ordering plus `.tail format::` rendering (EC-30–EC-34): every documented sort field orders correctly in both directions against a fixture whose append order matches none of them, bad `sort::`/`reverse::` values exit 1, `limit` caps after the sort with `limit::0` meaning unlimited, `.list`'s non-table formats are byte-identical to `.export`'s, and `.tail` renders each format with a bad one rejected before the follow loop blocks |
| `.serve` HTTP (FT-1–FT-12, IT-4, TC-3, IN-1–IN-3) | `serve_test.rs` | Startup line and loopback default, `GET /` dashboard, `/api/events` with query filtering and 400 on bad values or unrecognised keys, `/api/stats` grouping and 400 on bad `by`, `/api/health` structure and empty-journal nulls, 404 JSON for unknown `/api/*`, `port::`/`refresh::`/`open::` handling, CDN-free HTML, SIGTERM termination, `bind::` interface selection with its exposure warning, and exit-1 bind failures on a busy port or unparseable address |
