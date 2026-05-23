# clr CLI Documentation

### Scope

- **Purpose**: Document the clr command-line interface for Claude Code execution.
- **Responsibility**: Reference documentation for commands, parameters, types, and user stories.
- **In Scope**: commands, parameters, types, parameter groups, dictionary, user stories, env parameters.
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
| type.md | ✅ | ➖ | ➖ | ➖ | ➖ | Partial |
| param_group.md | ✅ | ➖ | ➖ | ➖ | ➖ | Partial |
| user_story/readme.md | ✅ | ✅ | ➖ | ➖ | ➖ | Complete |
| user_story/ (13 instances) | ✅ | ✅ | ➖ | ➖ | ➖ | Partial |
| env_param.md | ✅ | ➖ | ✅ | ➖ | ➖ | Complete |
| config_param.md | ➖ | ➖ | ➖ | ➖ | ➖ | N/A |
| format.md | ➖ | ➖ | ➖ | ➖ | ➖ | N/A |
| tests/docs/cli/readme.md | ➖ | ➖ | ➖ | ✅ | ➖ | Complete |
| tests/docs/cli/command/*.md (3 files) | ➖ | ➖ | ➖ | ✅ | ✅ | Complete |
| tests/docs/cli/param/*.md (27 files) | ➖ | ➖ | ➖ | ✅ | ✅ | Complete |
| tests/docs/cli/param_group/*.md (4 files) | ➖ | ➖ | ➖ | ✅ | ✅ | Complete |
| tests/docs/cli/type/*.md (12 types; 12 test specs) | ➖ | ➖ | ➖ | ✅ | ✅ | Complete |
| tests/docs/cli/env_param/*.md (2 files) | ➖ | ➖ | ➖ | ✅ | ✅ | Complete |

**Current Level:** L3
**Design Completeness:** 100%
**Implementation Status:** 27/27 params implemented; 25 CLR_* run env vars

### Navigation

- [Commands](command.md) — command reference (3 commands)
- [Parameters](param/readme.md) — flag and argument reference (27 parameters)
- [Types](type.md) — semantic type definitions (12 types)
- [Parameter Groups](param_group.md) — logical parameter groupings (4 groups)
- [Dictionary](dictionary.md) — domain vocabulary
- [User Stories](user_story/readme.md) — user goals and usage patterns (13 user stories)
- [Env Parameters](env_param.md) — input and subprocess environment variables (28 variables)

### See Also

- [feature/001_runner_tool.md](../feature/001_runner_tool.md) — architecture, separation of concerns, constraints
- [design_decisions.md](../design_decisions.md) — CLI redesign rationale
- [tests/docs/cli/](../../tests/docs/cli/readme.md) — test case planning (commands, params, types, groups, env params)
