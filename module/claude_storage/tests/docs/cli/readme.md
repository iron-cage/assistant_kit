# CLI Testing Collection

### Scope

- **Purpose**: Document integration and edge case test plans for all clg commands, parameters, types, and export formats.
- **Responsibility**: Index of per-command, per-parameter, per-group, per-type, per-format, and per-user-story test case planning files.
- **In Scope**: All 12 clg commands, all 27 parameters, all 5 parameter groups, the command_group evaluation (0 qualifying groups), all 13 types, all 3 export formats, and all 6 user stories.
- **Out of Scope**: Automated test implementations (→ `tests/` in crate), spec documentation (→ `docs/feature/`).

6-tier testing organization for `claude_storage` CLI, providing distinct audience focus at each level.

### Responsibility Table

| File | Responsibility |
|------|----------------|
| `command/` | Integration test cases per command |
| `param/` | Edge case tests per parameter |
| `param_group/` | Interaction tests per parameter group |
| `command_group/` | Evidentiary basis for command_group verdicts (0 groups) |
| `type/` | Type constraint validation tests per type |
| `format/` | Export output format verification tests per format |
| `user_story/` | Acceptance criterion tests per user story |
| `pitfall/` | Pitfall contract tests mirroring `docs/cli/pitfall/` |

### Overview

| Tier | Location | Purpose | Audience | Test Types |
|------|----------|---------|----------|-----------|
| Type | `type/*.md` | Validate type parsing, constraints, and error messages | Developers | Type constraint tests |
| Parameter | `param/*.md` | Validate individual parameter parsing and constraints | Developers | Unit tests, edge cases |
| Group | `param_group/*.md` | Test parameter interactions within groups | Developers | Corner cases, dependencies |
| Command Group | `command_group/*.md` | Record evidentiary basis for group verdicts | Developers | Static-dispatch evidence |
| Command | `command/*.md` | End-to-end command integration | QA / Users | Integration tests, workflows |
| Format | `format/*.md` | Verify export output conforms to format specifications | QA / Users | Format verification tests |
| User Story | `user_story/*.md` | Verify acceptance criteria per user story | QA / Users | Real-world scenario tests |

### Navigation

- [Command Tests](command/) — Integration tests per command
- [Parameter Tests](param/) — Edge case tests per parameter
- [Parameter Group Tests](param_group/) — Interaction tests per group
- [Command Group Tests](command_group/) — Evidentiary basis for command_group verdicts (0 groups)
- [Type Tests](type/) — Type constraint tests per type
- [Format Tests](format/) — Export format verification tests per format
- [User Story Tests](user_story/) — Acceptance criterion tests per user story

### Aggregate Counts

| Tier | Files | Tests |
|------|-------|-------|
| Commands | 13 files | 179 tests |
| Parameters | 27 files | 187 tests |
| Parameter Groups | 5 files | 31 tests |
| Command Groups | 1 file | 0 tests (0 groups qualify — evidentiary record only, no `INT-N`/`CC-N` cases) |
| Types | 13 files | 64 tests |
| Formats | 3 files | 15 tests |
| User Stories | 6 files | 29 tests |
| **Total** | **68 files** | **505 tests** |

### Test ID Conventions

| Prefix | Category | Used In |
|--------|----------|---------|
| `INT-N` | Integration test | Command tests (`command/`) |
| `EC-N` | Edge case | Parameter tests (`param/`) |
| `CC-N` | Cross-command interaction | Parameter group tests (`param_group/`) |
| `TC-N` | Type constraint | Type validation tests (`type/`) |
| `FM-N` | Format specification | Export format tests (`format/`) |
| `RWS-N` | Real-world scenario | User story acceptance tests (`user_story/`) |

### Related Documentation

- [command/readme.md](../../../docs/cli/command/readme.md) — Command specifications
- [param/readme.md](../../../docs/cli/param/readme.md) — Parameter specifications
- [param_group/readme.md](../../../docs/cli/param_group/readme.md) — Group specifications
- [command_group/readme.md](../../../docs/cli/command_group/readme.md) — Command group specifications (0 groups)
- [type/readme.md](../../../docs/cli/type/readme.md) — Type specifications
- [format/readme.md](../../../docs/cli/format/readme.md) — Export format specifications
