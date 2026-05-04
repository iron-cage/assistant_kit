# Parameter Tests

### Scope

- **Purpose**: Document edge case coverage for individual cm parameters.
- **Responsibility**: Index of per-parameter edge case test files covering parameter-level behavior.
- **In Scope**: All 10 cm parameter test files.
- **Out of Scope**: Command-level tests (→ `command/`), parameter group interactions (→ `param_group/`).

### Responsibility Table

| File | Responsibility |
|------|----------------|
| 01_version.md | Edge case tests for `version::` parameter |
| 02_dry.md | Edge case tests for `dry::` parameter |
| 03_force.md | Edge case tests for `force::` parameter |
| 04_verbosity.md | Edge case tests for `v::` / `verbosity::` parameter |
| 05_format.md | Edge case tests for `format::` parameter |
| 06_key.md | Edge case tests for `key::` parameter |
| 07_value.md | Edge case tests for `value::` parameter |
| 08_interval.md | Edge case tests for `interval::` parameter |
| 09_count.md | Edge case tests for `count::` parameter |
| 10_help_param.md | Edge case tests for `.help` parameter |
