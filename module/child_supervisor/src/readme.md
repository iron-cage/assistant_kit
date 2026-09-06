# src/

Core library implementation for `child_supervisor`.

## Responsibility Table

| File | Responsibility |
|------|----------------|
| `lib.rs` | Crate root, module wiring, public re-exports |
| `error.rs` | Hand-rolled error type and crate `Result` alias |
| `output.rs` | Bounded, cursor-addressed session output and its pump thread |
| `table.rs` | Hosted-session table keyed by a caller-chosen id |
