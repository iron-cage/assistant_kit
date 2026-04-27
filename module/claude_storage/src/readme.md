# src/

Source code for the `claude_storage` crate.

## Responsibility Table

| File | Responsibility |
|------|----------------|
| `lib.rs` | Crate root; re-exports from core; exposes `cli` and `cli_main` modules |
| `main.rs` | `claude_storage` binary entry; thin wrapper calling `cli_main::run()` |
| `cli_main.rs` | Shared CLI pipeline (REPL + one-shot); `pub fn run()` |
| `cli/` | Per-command CLI routines: one file per command, shared utilities in `storage.rs` and `format.rs` |
| `bin/` | Alias binary entry points (`clg`) |
