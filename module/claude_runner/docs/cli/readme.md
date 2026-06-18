# clr CLI Documentation

### Scope

- **Purpose**: Document the clr command-line interface for Claude Code execution.
- **Responsibility**: Reference documentation for commands, parameters, types, and user stories.
- **In Scope**: commands, parameters, types, parameter groups, dictionary, user stories, env parameters.
- **Out of Scope**: Implementation design (-> `feature/001_runner_tool.md`), API contracts (-> `api/001_public_api.md`), test planning (-> `tests/docs/cli/`).

Execute Claude Code with configurable `--flag value` parameters.

### Usage

```sh
clr [OPTIONS] [MESSAGE]
```

### Responsibility Table

| File | Responsibility |
|------|----------------|
| `command/` | Command reference: run, ask, isolated, refresh, help, ps, kill (7 commands) |
| `dictionary.md` | Domain vocabulary for clr CLI concepts |
| `command_defaults.md` | Cross-command parameter default matrix and isolated/refresh design targets |
| `env_param.md` | CLR_* env var fallbacks, CLR_GATE_DIR runtime config, and CLAUDE_CODE_* subprocess vars (58 vars) |
| `param_group/` | Logical parameter groupings (5 groups) |
| `type/` | Semantic type definitions (14 types) |
| `param/` | Individual parameter reference docs (56 parameters) |
| `user_story/` | User goal and usage pattern docs (27 user stories) |

### Completion Matrix

| Entity | L1 | L2 | L3 | L4 | L5 | Status |
|--------|----|----|----|----|----|----|
| readme.md | ✅ | ✅ | ➖ | ➖ | ➖ | Complete |
| command/ (7 instances) | ✅ | ✅ | ✅ | ➖ | ➖ | Complete |
| param/ (56 instances) | ✅ | ✅ | ✅ | ➖ | ➖ | Complete |
| dictionary.md | ✅ | ✅ | ➖ | ➖ | ➖ | Complete |
| type/ (14 instances) | ✅ | ✅ | ✅ | ➖ | ➖ | Complete |
| param_group/ (5 instances) | ✅ | ✅ | ✅ | ➖ | ➖ | Complete |
| user_story/ (27 instances) | ✅ | ✅ | ✅ | ➖ | ➖ | Complete |
| env_param.md | ✅ | ✅ | ✅ | ➖ | ➖ | Complete |
| config_param.md | ➖ | ➖ | ➖ | ➖ | ➖ | N/A — no config file mechanism |
| format.md | ➖ | ➖ | ➖ | ➖ | ➖ | N/A — no named output formats |
| tests/docs/cli/readme.md | ➖ | ➖ | ➖ | ✅ | ➖ | Complete |
| tests/docs/cli/command/ (7 files) | ➖ | ➖ | ➖ | ✅ | ✅ | Complete |
| tests/docs/cli/param/ (56 files) | ➖ | ➖ | ➖ | ✅ | ✅ | Complete |
| tests/docs/cli/param_group/ (5 files) | ➖ | ➖ | ➖ | ✅ | ✅ | Complete |
| tests/docs/cli/type/ (14 files) | ➖ | ➖ | ➖ | ✅ | ✅ | Complete |
| tests/docs/cli/env_param/ (2 files) | ➖ | ➖ | ➖ | ✅ | ✅ | Complete |
| tests/docs/cli/user_story/ (27 files) | ➖ | ➖ | ➖ | ✅ | ✅ | Complete |

**Current Level:** L5
**Design Completeness:** 100%
**Implementation Status:** Complete

### Navigation

- [Commands](command/readme.md) — command reference (7 commands)
- [Parameters](param/readme.md) — flag and argument reference (56 parameters)
- [Types](type/readme.md) — semantic type definitions (14 types)
- [Parameter Groups](param_group/readme.md) — logical parameter groupings (5 groups)
- [Dictionary](dictionary.md) — domain vocabulary
- [User Stories](user_story/readme.md) — user goals and usage patterns (27 user stories)
- [Env Parameters](env_param.md) — input, runtime config, and subprocess environment variables (58 variables)

### See Also

- [feature/001_runner_tool.md](../feature/001_runner_tool.md) — architecture, separation of concerns, constraints
- [001_design_decisions.md](../001_design_decisions.md) — CLI redesign rationale
- [tests/docs/cli/](../../tests/docs/cli/readme.md) — test case planning (commands, params, types, groups, env params)
