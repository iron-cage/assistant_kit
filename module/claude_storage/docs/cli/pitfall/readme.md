# `claude_storage` CLI Pitfall Documentation

### Scope

- **Purpose**: Document CLI implementation pitfalls to prevent recurring mistakes.
- **Responsibility**: Parameter validation gaps, cross-command bug propagation, test data format.
- **In Scope**: Pitfalls discovered through bug fixes, manual testing, and code review.
- **Out of Scope**: System invariants (→ `../../invariant/`), command specs (→ `../command/`).

### Responsibility Table

| File | Responsibility |
|------|----------------|
| `01_parameter_validation.md` | Explicit validation required despite defaults |
| `02_cross_command_propagation.md` | Bug patterns propagate across all commands |
| `03_test_data_format.md` | Test JSONL must match production format |
