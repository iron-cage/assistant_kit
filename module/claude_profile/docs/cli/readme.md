# clp CLI Documentation

### Scope

- **Purpose**: Document the clp command-line interface for Claude Code account management.
- **Responsibility**: Reference documentation for commands, parameters, types, and workflows.
- **In Scope**: commands, params, types, parameter groups, dictionary, workflows.
- **Out of Scope**: Implementation design (→ `feature/`), quality constraints (→ `invariant/`), test planning (→ `tests/docs/cli/`).

Manage Claude Code account credentials: save, list, switch, and delete named profiles; check token status; discover ~/.claude/ file paths.

### Completion Matrix

| File | L1 | L2 | L3 | L4 | L5 | Status |
|------|----|----|----|----|----|--------|
| readme.md | ✅ | ✅ | ✅ | ✅ | ✅ | Complete |
| commands.md | ✅ | ✅ | ✅ | ✅ | ✅ | Complete |
| params.md | ✅ | ✅ | ✅ | ✅ | ✅ | Complete |
| dictionary.md | ➖ | ✅ | ✅ | ✅ | ✅ | Complete |
| types.md | ➖ | ➖ | ✅ | ✅ | ✅ | Complete |
| parameter_groups.md | ➖ | ➖ | ✅ | ✅ | ✅ | Complete |
| parameter_interactions.md | ➖ | ➖ | ✅ | ➖ | ➖ | Complete |
| workflows.md | ➖ | ➖ | ✅ | ✅ | ✅ | Complete |

**Current Level:** L4 (Workflow Complete)
**Design Completeness:** 100%
**Implementation Status:** 100% (11/11 commands implemented)

### Navigation

- [Commands](commands.md)
- [Parameters](params.md)
- [Dictionary](dictionary.md)
- [Types](types.md)
- [Parameter Groups](parameter_groups.md)
- [Parameter Interactions](parameter_interactions.md)
- [Workflows](workflows.md)

### See Also

- [tests/docs/cli/](../../tests/docs/cli/readme.md) — test case planning (commands, params, groups)
