# clr CLI Documentation

### Scope

- **Purpose**: Document the clr command-line interface for Claude Code execution.
- **Responsibility**: Reference documentation for commands, parameters, types, and workflows.
- **In Scope**: commands, parameters, types, parameter groups, dictionary, workflows, env parameters.
- **Out of Scope**: Implementation design (→ `feature/001_runner_tool.md`), API contracts (→ `api/001_public_api.md`), test planning (→ `tests/docs/cli/`).

Execute Claude Code with configurable `--flag value` parameters.

### Usage

```sh
clr [OPTIONS] [MESSAGE]
```

### Completion Matrix

| File | L1 | L2 | L3 | L4 | L5 | Status |
|------|----|----|----|----|----|----|
| readme.md | ✅ | ✅ | ➖ | ➖ | ➖ | Complete |
| command.md | ✅ | ✅ | ✅ | ➖ | ➖ | Complete |
| param/readme.md | ✅ | ✅ | ✅ | ➖ | ➖ | Complete |
| dictionary.md | ✅ | ✅ | ➖ | ➖ | ➖ | Complete |
| type.md | ✅ | ➖ | ✅ | ➖ | ➖ | Complete |
| param_group.md | ✅ | ➖ | ✅ | ➖ | ➖ | Complete |
| workflow_scenario.md | ✅ | ➖ | ✅ | ➖ | ➖ | Complete |
| env_param.md | ✅ | ➖ | ✅ | ➖ | ➖ | Complete |
| config_param.md | ➖ | ➖ | ➖ | ➖ | ➖ | N/A |
| format.md | ➖ | ➖ | ➖ | ➖ | ➖ | N/A |
| tests/docs/cli/readme.md | ➖ | ➖ | ➖ | ✅ | ➖ | Complete |
| tests/docs/cli/command/*.md (3 files) | ➖ | ➖ | ➖ | ✅ | ✅ | Complete |
| tests/docs/cli/param/*.md (24 files) | ➖ | ➖ | ➖ | ✅ | ✅ | Complete |
| tests/docs/cli/param_group/*.md (4 files) | ➖ | ➖ | ➖ | ✅ | ✅ | Complete |
| tests/docs/cli/type/*.md (11 files) | ➖ | ➖ | ➖ | ✅ | ✅ | Complete |
| tests/docs/cli/env_param/*.md (2 files) | ➖ | ➖ | ➖ | ✅ | ✅ | Complete |

**Current Level:** L5
**Design Completeness:** 100%
**Implementation Status:** 100% (3/3 commands, 24/24 params, 24 CLR_* env vars implemented)

### Navigation

- [Commands](command.md) — command reference (3 commands)
- [Parameters](param/readme.md) — flag and argument reference (24 parameters)
- [Types](type.md) — semantic type definitions (11 types)
- [Parameter Groups](param_group.md) — logical parameter groupings (4 groups)
- [Dictionary](dictionary.md) — domain vocabulary
- [Workflows](workflow_scenario.md) — common usage patterns (10 workflows)
- [Env Parameters](env_param.md) — input and subprocess environment variables (25 variables)

### See Also

- [feature/001_runner_tool.md](../feature/001_runner_tool.md) — architecture, separation of concerns, constraints
- [design_decisions.md](../design_decisions.md) — CLI redesign rationale
- [tests/docs/cli/](../../tests/docs/cli/readme.md) — test case planning (commands, params, types, groups, env params)
