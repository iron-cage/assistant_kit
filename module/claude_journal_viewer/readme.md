# claude_journal_viewer

CLI and web viewer for CLR journal events. Binary: `clj`.

### Scope

Provides `.list`, `.tail`, `.stats`, `.search`, `.serve`, `.prune`, `.status`, and `.export` commands using unilang `.command param::value` syntax. Web viewer embeds a single-page HTML app served by `tiny-http`. Reads journal data via `claude_journal::JournalReader`.

### Responsibility Table

| File | Responsibility |
|------|----------------|
| `src/lib.rs` | Public module re-exports and command registration |
| `src/cli_main.rs` | Binary entry point with unilang command dispatch |
| `src/cli/mod.rs` | Command dispatch routing |
| `src/cli/list.rs` | `.list` command — filtered event table |
| `src/cli/tail.rs` | `.tail` command — real-time event following |
| `src/cli/stats.rs` | `.stats` command — aggregate statistics |
| `src/cli/search.rs` | `.search` command — full-text event search |
| `src/cli/serve.rs` | `.serve` command — embedded web viewer |
| `src/cli/prune.rs` | `.prune` command — retention management |
| `src/cli/status.rs` | `.status` command — journal health report |
| `src/cli/export.rs` | `.export` command — multi-format file export |
| `src/web/index.html` | Embedded single-page web viewer |
| `claude_journal.commands.yaml` | Unilang command definitions |
| `docs/` | Feature docs, CLI reference, invariant constraints |
| `tests/` | Command integration tests |
