# tests/

Integration tests for the `assistant` crate.

## Responsibility Table

| File | Responsibility |
|------|----------------|
| `cli_sanity.rs` | Compile and link sanity check for the `ast` binary against all Layer 2 crates |
| `aggregation.rs` | Behavioral spec verification: super-app aggregation and completeness (FT-1..FT-4, IC-1..IC-2) |
| `docs/` | Test spec files defining expected behavior per doc instance |
