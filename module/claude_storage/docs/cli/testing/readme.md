# Testing Structure

3-tier testing organization for `claude_storage` CLI, providing distinct audience focus at each level.

## Overview

| Tier | Location | Purpose | Audience | Test Types |
|------|----------|---------|----------|-----------|
| Parameter | `testing/param/*.md` | Validate individual parameter parsing and constraints | Developers | Unit tests, edge cases |
| Group | `testing/param_group/*.md` | Test parameter interactions within groups | Developers | Corner cases, dependencies |
| Command | `testing/command/*.md` | End-to-end command integration | QA / Users | Integration tests, workflows |

## Navigation

- [Command Tests](command/) — Integration tests per command
- [Parameter Tests](param/) — Edge case tests per parameter
- [Parameter Group Tests](param_group/) — Interaction tests per group

## Aggregate Counts

| Tier | Files | Tests |
|------|-------|-------|
| Commands | 12 files | 149 tests |
| Parameters | 21 files | 148 tests |
| Parameter Groups | 6 files | 31 tests |
| **Total** | **39 files** | **328 tests** |

## Test ID Conventions

| Prefix | Category | Used In |
|--------|----------|---------|
| `IT-N` | Integration test | Command tests |
| `EC-N` | Edge case | Parameter tests |
| `CC-N` | Conditional case | Parameter group tests |
| `CD-N` | Dependency test | Parameter group tests |

## Related Documentation

- [commands.md](../commands.md) — Command specifications
- [params.md](../params.md) — Parameter specifications
- [parameter_groups.md](../parameter_groups.md) — Group specifications
- [types.md](../types.md) — Type validation rules (informs edge case design)
