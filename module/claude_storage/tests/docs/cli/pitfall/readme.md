# CLI Pitfall Testing Collection

### Scope

- **Purpose**: Document test plans for CLI implementation pitfalls in `docs/cli/pitfall/`.
- **Responsibility**: Index of per-pitfall test case planning files.
- **In Scope**: All 3 CLI implementation pitfalls (parameter validation, cross-command propagation, test data format).
- **Out of Scope**: Automated test implementations (→ `tests/` in crate), feature specs (→ `docs/feature/`).

Direct contract verification for the pitfalls defined in `docs/cli/pitfall/01-03`.

### Responsibility Table

| Name | Purpose | Status |
|------|---------|--------|
| `01_parameter_validation.md` | PF- test cases for parameter validation pitfall (`docs/cli/pitfall/01_parameter_validation.md`) | ⏳ |
| `02_cross_command_propagation.md` | PF- test cases for cross-command propagation pitfall (`docs/cli/pitfall/02_cross_command_propagation.md`) | ⏳ |
| `03_test_data_format.md` | PF- test cases for test data format pitfall (`docs/cli/pitfall/03_test_data_format.md`) | ⏳ |
