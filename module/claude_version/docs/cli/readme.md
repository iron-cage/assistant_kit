# cm CLI Documentation

### Scope

- **Purpose**: Document the cm command-line interface for Claude Code version and settings management.
- **Responsibility**: Reference documentation for commands, parameters, types, and workflows.
- **In Scope**: commands, params, types, parameter groups, dictionary, workflows, parameter interactions.
- **Out of Scope**: Implementation design (→ `feature/`), design rationale (→ `design_decisions.md`), test planning (→ `tests/doc/cli/`).

Manage Claude Code installation: versions, processes, and settings.

### Usage

```sh
cm <.command> [param::value ...]
```

### Completion Matrix

| File | L1 | L2 | L3 | L4 | L5 | Status |
|------|----|----|----|----|----|--------|
| readme.md | ✅ | ✅ | ✅ | ✅ | ✅ | Complete |
| commands.md | ✅ | ✅ | ✅ | ✅ | ✅ | Complete |
| params.md | ✅ | ✅ | ✅ | ✅ | ✅ | Complete |
| dictionary.md | ➖ | ✅ | ➖ | ➖ | ➖ | Complete |
| types.md | ➖ | ➖ | ✅ | ✅ | ✅ | Complete |
| parameter_groups.md | ➖ | ➖ | ✅ | ✅ | ✅ | Complete |
| parameter_interactions.md | ➖ | ➖ | ➖ | ✅ | ✅ | Complete |
| workflows.md | ➖ | ➖ | ✅ | ➖ | ➖ | Complete |

**Current Level:** L4 (Specification Complete)
**Design Completeness:** 100%
**Implementation Status:** Complete (12 commands implemented)

### Navigation

- [Commands](commands.md) — command reference
- [Parameters](params.md) — flag reference
- [Types](types.md) — semantic type definitions
- [Parameter Groups](parameter_groups.md) — logical parameter groupings
- [Parameter Interactions](parameter_interactions.md) — cross-parameter constraints
- [Dictionary](dictionary.md) — domain vocabulary
- [Workflows](workflows.md) — common usage patterns

### See Also

- [feature/001_version_management.md](../feature/001_version_management.md) — version management, architecture, constraints
- [design_decisions.md](../design_decisions.md) — CLI redesign rationale
- [tests/doc/cli/](../../tests/doc/cli/readme.md) — test case planning (commands, params, groups)
