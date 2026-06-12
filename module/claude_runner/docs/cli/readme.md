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
| `command/` | Command reference: run, ask, isolated, refresh, help, ps (6 commands) |
| `dictionary.md` | Domain vocabulary for clr CLI concepts |
| `command_defaults.md` | Cross-command parameter default matrix and isolated/refresh design targets |
| `env_param.md` | CLR_* env var fallbacks and CLAUDE_CODE_* subprocess vars |
| `param_group/` | Logical parameter groupings (4 groups) |
| `type/` | Semantic type definitions (12 types) |
| `param/` | Individual parameter reference docs (36 parameters) |
| `user_story/` | User goal and usage pattern docs (26 user stories) |

### Completion Matrix

| Entity | L1 | L2 | L3 | L4 | L5 | Status |
|--------|----|----|----|----|----|----|
| readme.md | ✅ | ✅ | ➖ | ➖ | ➖ | Complete |
| command/ (6 instances) | ✅ | ✅ | ✅ | ➖ | ➖ | Complete |
| param/ (36 instances) | ✅ | ✅ | ✅ | ➖ | ➖ | Complete |
| dictionary.md | ✅ | ✅ | ➖ | ➖ | ➖ | Complete |
| type/ (12 instances) | ✅ | ✅ | ✅ | ➖ | ➖ | Complete |
| param_group/ (4 instances) | ✅ | ✅ | ✅ | ➖ | ➖ | Complete |
| user_story/ (26 instances) | ✅ | ✅ | ✅ | ➖ | ➖ | Complete |
| env_param.md | ✅ | ✅ | ✅ | ➖ | ➖ | Complete |
| config_param.md | ➖ | ➖ | ➖ | ➖ | ➖ | N/A — no config file mechanism |
| format.md | ➖ | ➖ | ➖ | ➖ | ➖ | N/A — no named output formats |
| tests/docs/cli/readme.md | ➖ | ➖ | ➖ | ✅ | ➖ | Complete |
| tests/docs/cli/command/ (6 files) | ➖ | ➖ | ➖ | ✅ | ✅ | Complete |
| tests/docs/cli/param/ (36 files) | ➖ | ➖ | ➖ | ✅ | ✅ | Complete |
| tests/docs/cli/param_group/ (4 files) | ➖ | ➖ | ➖ | ✅ | ✅ | Complete |
| tests/docs/cli/type/ (12 files) | ➖ | ➖ | ➖ | ✅ | ✅ | Complete |
| tests/docs/cli/env_param/ (2 files) | ➖ | ➖ | ➖ | ✅ | ✅ | Complete |
| tests/docs/cli/user_story/ (26 files) | ➖ | ➖ | ➖ | ✅ | ✅ | Complete |

**Current Level:** L5
**Design Completeness:** 100%
**Implementation Status:** Complete

### Navigation

- [Commands](command/readme.md) — command reference (6 commands)
- [Parameters](param/readme.md) — flag and argument reference (36 parameters)
- [Types](type/readme.md) — semantic type definitions (12 types)
- [Parameter Groups](param_group/readme.md) — logical parameter groupings (4 groups)
- [Dictionary](dictionary.md) — domain vocabulary
- [User Stories](user_story/readme.md) — user goals and usage patterns (26 user stories)
- [Env Parameters](env_param.md) — input and subprocess environment variables (38 variables)

### See Also

- [feature/001_runner_tool.md](../feature/001_runner_tool.md) — architecture, separation of concerns, constraints
- [001_design_decisions.md](../001_design_decisions.md) — CLI redesign rationale
- [tests/docs/cli/](../../tests/docs/cli/readme.md) — test case planning (commands, params, types, groups, env params)
