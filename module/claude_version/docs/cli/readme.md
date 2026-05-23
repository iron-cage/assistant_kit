# cm CLI Documentation

### Scope

- **Purpose**: Document the cm command-line interface for Claude Code version and settings management.
- **Responsibility**: Reference documentation for commands, parameters, types, and workflows.
- **In Scope**: commands, params, types, parameter groups, dictionary, workflows, parameter interactions.
- **Out of Scope**: Implementation design and behavioral contracts — validation rules, exit code semantics, pipeline architecture (→ `feature/`), design rationale (→ `001_design_decisions.md`), test planning (→ `tests/docs/cli/`).

Manage Claude Code installation: versions, processes, and settings.

### Usage

```sh
cm <.command> [param::value ...]
```

### Completion Matrix

| File | L1 | L2 | L3 | L4 | L5 | Status |
|------|----|----|----|----|----|--------|
| readme.md | ✅ | ✅ | ✅ | ✅ | ✅ | Complete |
| 001_commands.md | ✅ | ✅ | ✅ | ✅ | ✅ | Complete |
| 005_params.md | ✅ | ✅ | ✅ | ✅ | ✅ | Complete |
| 002_dictionary.md | ➖ | ✅ | ➖ | ➖ | ➖ | Complete |
| 006_types.md | ➖ | ➖ | ✅ | ✅ | ✅ | Complete |
| 003_parameter_groups.md | ➖ | ➖ | ✅ | ✅ | ✅ | Complete |
| 004_parameter_interactions.md | ➖ | ➖ | ➖ | ✅ | ✅ | Complete |
| 007_workflows.md | ➖ | ➖ | ✅ | ➖ | ➖ | Complete |
| procedure.md | ➖ | ➖ | ➖ | ➖ | ➖ | Entity Infrastructure |

**Current Level:** L4 (Specification Complete)
**Design Completeness:** 100%
**Implementation Status:** Complete (12 commands implemented)

### Navigation

- [Commands](001_commands.md) — command reference
- [Parameters](005_params.md) — flag reference
- [Types](006_types.md) — semantic type definitions
- [Parameter Groups](003_parameter_groups.md) — logical parameter groupings
- [Parameter Interactions](004_parameter_interactions.md) — cross-parameter constraints
- [Dictionary](002_dictionary.md) — domain vocabulary
- [Workflows](007_workflows.md) — common usage patterns

### See Also

- [feature/001_version_management.md](../feature/001_version_management.md) — version management, architecture, constraints
- [001_design_decisions.md](../001_design_decisions.md) — CLI redesign rationale
- [tests/docs/cli/](../../tests/docs/cli/readme.md) — test case planning (commands, params, groups)
