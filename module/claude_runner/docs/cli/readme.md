# clr CLI Documentation

### Scope

- **Purpose**: Document the clr command-line interface for Claude Code execution.
- **Responsibility**: Reference documentation for commands, parameters, types, and user stories.
- **In Scope**: commands, parameters, types, parameter groups, dictionary, user stories, env parameters.
- **Out of Scope**: Implementation design (-> `feature/001_runner_tool.md`), API contracts (-> `api/001_public_api.md`), test planning (-> `tests/docs/cli/`).

Execute Claude Code with configurable `--flag value` parameters.

### Usage

```sh
clr <command> [OPTIONS] [MESSAGE]
```

### Responsibility Table

| File | Responsibility |
|------|----------------|
| `command/` | Command reference: run, ask, isolated, refresh, help, ps, kill, tools, scope, query (10 commands) |
| `001_dictionary.md` | Domain vocabulary for clr CLI concepts |
| `002_command_defaults.md` | Cross-command parameter default matrix and isolated/refresh design targets |
| `parity/` | Cross-command behavioral parity comparisons (2 instances) |
| `003_env_param.md` | CLR_* env var fallbacks, gate/query runtime config (dir/poll/attempts/config-dir/query-dir), and CLAUDE_CODE_* subprocess vars (93 vars) |
| `config_param.md` | Config-file parameter tier: eligible parameters, TOML key reference, discovery/precedence |
| `param_group/` | Logical parameter groupings (7 groups) |
| `type/` | Semantic type definitions (13 active types; 1 deprecated) |
| `param/` | Individual parameter reference docs (75 active; 1 deprecated) |
| `user_story/` | User goal and usage pattern docs (29 user stories) |

### Completion Matrix

| Entity | L1 | L2 | L3 | L4 | L5 | Status |
|--------|----|----|----|----|----|----|
| readme.md | ✅ | ✅ | ➖ | ➖ | ➖ | Complete |
| command/ (10 instances) | ✅ | ✅ | ✅ | ➖ | ➖ | Complete |
| param/ (75 instances; 1 deprecated) | ✅ | ✅ | ✅ | ➖ | ➖ | Complete |
| 001_dictionary.md | ✅ | ✅ | ➖ | ➖ | ➖ | Complete |
| type/ (14 instances; 1 deprecated) | ✅ | ✅ | ✅ | ➖ | ➖ | Complete |
| param_group/ (7 instances) | ✅ | ✅ | ✅ | ➖ | ➖ | Complete |
| user_story/ (29 instances) | ✅ | ✅ | ✅ | ➖ | ➖ | Complete |
| 003_env_param.md | ✅ | ✅ | ✅ | ➖ | ➖ | Complete |
| parity/ (2 instances) | ✅ | ✅ | ✅ | ➖ | ➖ | Complete |
| config_param.md | ✅ | ✅ | ➖ | ➖ | ➖ | Complete |
| format.md | ➖ | ➖ | ➖ | ➖ | ➖ | N/A — no named output formats |
| tests/docs/cli/readme.md | ➖ | ➖ | ➖ | ✅ | ➖ | Complete |
| tests/docs/cli/dictionary.md | ➖ | ➖ | ➖ | ✅ | ✅ | Complete |
| tests/docs/cli/command/ (10 files) | ➖ | ➖ | ➖ | ✅ | ✅ | Complete |
| tests/docs/cli/param/ (75 files) | ➖ | ➖ | ➖ | ✅ | ✅ | Complete |
| tests/docs/cli/param_group/ (7 files) | ➖ | ➖ | ➖ | ✅ | ✅ | Complete |
| tests/docs/cli/type/ (14 files) | ➖ | ➖ | ➖ | ✅ | ✅ | Complete |
| tests/docs/cli/env_param/ (3 files) | ➖ | ➖ | ➖ | ✅ | ✅ | Complete |
| tests/docs/cli/parity/ (2 files) | ➖ | ➖ | ➖ | ✅ | ✅ | Complete |
| tests/docs/cli/user_story/ (29 files) | ➖ | ➖ | ➖ | ✅ | ✅ | Complete |

**Current Level:** L5
**Design Completeness:** 100%
**Implementation Status:** Complete (`clr tools` filter/projection/inspect flags documented at Level 2 pending implementation — see task tracking under `task/claude_runner/`)

### Navigation

- [Commands](command/readme.md) — command reference (10 commands)
- [Parameters](param/readme.md) — flag and argument reference (75 active; 1 deprecated)
- [Types](type/readme.md) — semantic type definitions (13 active types; 1 deprecated)
- [Parameter Groups](param_group/readme.md) — logical parameter groupings (7 groups)
- [Dictionary](001_dictionary.md) — domain vocabulary
- [User Stories](user_story/readme.md) — user goals and usage patterns (29 user stories)
- [Env Parameters](003_env_param.md) — input, runtime config, and subprocess environment variables (93 variables)
- [Config File Parameters](config_param.md) — TOML config-file parameter tier (38 eligible parameters)
- [Parity](parity/readme.md) — cross-command behavioral parity comparisons (2 instances)

### See Also

- [feature/001_runner_tool.md](../feature/001_runner_tool.md) — architecture, separation of concerns, constraints
- [feature/006_cli_design.md](../feature/006_cli_design.md) — CLI redesign rationale
- [tests/docs/cli/](../../tests/docs/cli/readme.md) — test case planning (commands, params, types, groups, env params)
