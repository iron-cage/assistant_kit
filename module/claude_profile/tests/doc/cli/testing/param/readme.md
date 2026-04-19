# Parameter Tests

### Scope

- **Purpose**: Document edge case coverage for individual clp parameters.
- **Responsibility**: Index of per-parameter edge case test files covering parameter-level behavior.
- **In Scope**: `name::`, `verbosity::`, `format::`, `threshold::`, `dry::` edge cases.
- **Out of Scope**: Command-level tests (→ `command/`), parameter group interactions (→ `param_group/`).

### Responsibility Table

| File | Responsibility |
|------|----------------|
| 01_name.md | Edge cases for `name::` parameter |
| 02_verbosity.md | Edge cases for `verbosity::` / `v::` parameter |
| 03_format.md | Edge cases for `format::` parameter |
| 04_threshold.md | Edge cases for `threshold::` parameter |
| 05_dry.md | Edge cases for `dry::` parameter |
