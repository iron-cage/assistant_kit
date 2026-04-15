# src/

Source code for the `claude_version` crate (binary + library).

## Responsibility Table

| File | Responsibility |
|------|----------------|
| `lib.rs` | Crate root; feature-gated module declarations; `run_cli()` 5-phase pipeline entry |
| `main.rs` | Thin wrapper delegating to `run_cli()` |
| `bin/` | Alias binary entry points (`clv`) |
| `output.rs` | `OutputOptions`, `OutputFormat`; text/json formatting utilities |
| `adapter.rs` | Argv-to-unilang token conversion: alias expansion, bool/int validation |
| `commands.rs` | 12 command routines returning `Result<OutputData, ErrorData>` |
| `settings_io.rs` | Atomic read/write of Claude's `settings.json` |
