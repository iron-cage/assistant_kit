# CLI Testing Doc Entity

### Scope

- **Purpose**: Document integration and edge case test plans for all clg commands and parameters.
- **Responsibility**: Index of per-command, per-parameter, and per-group test case planning files.
- **In Scope**: All 11 clg commands, all 20 parameters, and all 5 parameter groups.
- **Out of Scope**: Automated test implementations (→ `tests/` in crate), spec documentation (→ `docs/feature/`).

3-tier testing organization for `claude_storage` CLI, providing distinct audience focus at each level.

### Responsibility Table

| File | Responsibility |
|------|----------------|
| `command/` | Integration test cases per command |
| `param/` | Edge case tests per parameter |
| `param_group/` | Interaction tests per parameter group |

### Overview

| Tier | Location | Purpose | Audience | Test Types |
|------|----------|---------|----------|-----------|
| Parameter | `testing/param/*.md` | Validate individual parameter parsing and constraints | Developers | Unit tests, edge cases |
| Group | `testing/param_group/*.md` | Test parameter interactions within groups | Developers | Corner cases, dependencies |
| Command | `testing/command/*.md` | End-to-end command integration | QA / Users | Integration tests, workflows |

### Navigation

- [Command Tests](command/) — Integration tests per command
- [Parameter Tests](param/) — Edge case tests per parameter
- [Parameter Group Tests](param_group/) — Interaction tests per group

### Aggregate Counts

| Tier | Files | Tests |
|------|-------|-------|
| Commands | 11 files | 149 tests |
| Parameters | 20 files | 148 tests |
| Parameter Groups | 5 files | 31 tests |
| **Total** | **36 files** | **328 tests** |

### Test ID Conventions

| Prefix | Category | Used In |
|--------|----------|---------|
| `IT-N` | Integration test | Command tests |
| `EC-N` | Edge case | Parameter tests |
| `CC-N` | Conditional case | Parameter group tests |
| `CD-N` | Dependency test | Parameter group tests |

### Related Documentation

- [commands.md](../../../../docs/cli/commands.md) — Command specifications
- [params.md](../../../../docs/cli/params.md) — Parameter specifications
- [parameter_groups.md](../../../../docs/cli/parameter_groups.md) — Group specifications
- [types.md](../../../../docs/cli/types.md) — Type validation rules (informs edge case design)
