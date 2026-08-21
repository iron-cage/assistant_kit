# claude_journal_viewer

CLI and web viewer for CLR journal events. Binary: `clj`.

### Scope

Provides `.list`, `.tail`, `.stats`, `.search`, `.serve`, `.prune`, `.status`, `.export`, and `.chart` commands using unilang `.command param::value` syntax. Web viewer embeds a single-page HTML app served by `tiny-http`. Reads journal data via `claude_journal::JournalReader`.

### Responsibility Table

| File | Responsibility |
|------|----------------|
| `src/lib.rs` | Public module re-exports and crate docs |
| `src/cli_main.rs` | `clj` binary — arg parsing, command dispatch, `.tail`/`.serve` loops, embedded web page, help text |
| `src/output.rs` | Shared command output logic — filters, formatting, all `.list`/`.stats`/`.search`/`.status`/`.prune`/`.export`/`.chart` bodies |
| `src/routines.rs` | Unilang routine adapters exposing the same commands to the assistant registry |
| `claude_journal.commands.yaml` | Unilang command definitions |
| `verb/` | Shell scripts implementing do-protocol verbs for this crate. |
| `docs/` | Feature docs, CLI reference, invariant constraints |
| `tests/` | Command integration tests |
