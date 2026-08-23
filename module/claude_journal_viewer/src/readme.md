# src/

CLI and web viewer for CLR journal events.

## Responsibility Table

| File | Responsibility |
|------|----------------|
| `lib.rs` | Crate root; public module re-exports and crate docs |
| `cli_main.rs` | `clj` binary: arg parsing, dispatch, `.tail`/`.serve` loops |
| `output.rs` | Shared command bodies, filter construction, and output formatting |
| `routines.rs` | Unilang routine adapters for `ast .journal.*` integration |
