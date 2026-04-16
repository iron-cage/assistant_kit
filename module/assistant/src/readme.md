# src/

Source code for the `assistant` crate.

## Responsibility Table

| File | Responsibility |
|------|----------------|
| `lib.rs` | Crate root; feature-gate declarations and Layer 2 registry aggregation |
| `main.rs` | `ast` binary entry point; delegates to aggregated command registries |
