# src/

Source code for the `claude_tools` crate.

## Responsibility Table

| File | Responsibility |
|------|----------------|
| `lib.rs` | Crate root; feature-gate declarations and Layer 2 registry aggregation |
| `main.rs` | `clt` binary entry point; delegates to aggregated command registries |
