# clr CLI Documentation

### Scope

- **Purpose**: Document the clr command-line interface for Claude Code execution.
- **Responsibility**: Reference documentation for commands, parameters, types, and workflows.
- **In Scope**: commands, parameters, types, parameter groups, dictionary, workflows, parameter interactions.
- **Out of Scope**: Implementation design (→ `feature/001_runner_tool.md`), API contracts (→ `api/001_public_api.md`), test planning (→ `tests/doc/cli/`).

Execute Claude Code with configurable `--flag value` parameters.

### Usage

```sh
clr [OPTIONS] [MESSAGE]
```

### Completion Matrix

| File | L1 | L2 | L3 | L4 | L5 | Status |
|------|----|----|----|----|----|----|
| readme.md | done | done | done | done | — | Complete |
| commands.md | done | done | done | done | — | Complete |
| params.md | done | done | done | done | — | Complete |
| dictionary.md | — | done | — | — | — | Complete |
| types.md | — | — | done | done | — | Complete |
| parameter_groups.md | — | — | done | done | — | Complete |
| workflows.md | — | — | done | done | — | Complete |
| parameter_interactions.md | — | — | — | done | — | Complete |
**Current Level:** L4 (Specification Complete)

### Navigation

- [Commands](commands.md) — command reference (2 commands)
- [Parameters](params.md) — flag and argument reference (18 parameters)
- [Types](types.md) — semantic type definitions (7 types)
- [Parameter Groups](parameter_groups.md) — logical parameter groupings (3 groups)
- [Dictionary](dictionary.md) — domain vocabulary
- [Workflows](workflows.md) — common usage patterns (9 workflows)
- [Parameter Interactions](parameter_interactions.md) — flag interaction rules and precedence
### See Also

- [feature/001_runner_tool.md](../feature/001_runner_tool.md) — architecture, separation of concerns, constraints
- [design_decisions.md](../design_decisions.md) — CLI redesign rationale
- [tests/doc/cli/](../../tests/doc/cli/readme.md) — test case planning (commands, params, groups)
