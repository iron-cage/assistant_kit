# tests/

Integration tests for the `assistant` crate.

## Responsibility Table

| File | Responsibility |
|------|----------------|
| `cli_sanity.rs` | Compile and link sanity check for the `ast` binary against all Layer 2 crates |
| `aggregation.rs` | Super-app aggregation feature and invariant tests (FT-1..4, IC-1..2, negative) |
| `docs/` | Test surface specifications for feature and invariant doc instances |
