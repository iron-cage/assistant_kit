# clv CLI Documentation

### Scope

- **Purpose**: Document the clv command-line interface for Claude Code version and settings management.
- **Responsibility**: Reference documentation for commands, parameters, types, output formats, and user stories.
- **In Scope**: commands, params, types, parameter groups, dictionary, user stories, output formats, environment parameters, config parameters, parameter interactions.
- **Out of Scope**: Implementation design and behavioral contracts — validation rules, exit code semantics, pipeline architecture (→ `feature/`), design rationale (→ `001_design_decisions.md`), test planning (→ `tests/docs/cli/`).

Manage Claude Code installation: versions, processes, and settings.

### Usage

```sh
clv <.command> [param::value ...]
```

### Completion Matrix

| File | L1 | L2 | L3 | L4 | L5 | Status |
|------|----|----|----|----|----|--------|
| readme.md | ✅ | ✅ | ✅ | ✅ | ✅ | Complete |
| command/ | ✅ | ✅ | ✅ | ✅ | ✅ | Complete (6 files) |
| param/ | ✅ | ✅ | ✅ | ✅ | ✅ | Complete (12 files) |
| type/ | ✅ | ✅ | ✅ | ✅ | ✅ | Complete (7 files) |
| param_group/ | ✅ | ✅ | ✅ | ✅ | ✅ | Complete (5 files) |
| 002_dictionary.md | ➖ | ✅ | ➖ | ➖ | ➖ | Complete |
| 004_parameter_interactions.md | ➖ | ➖ | ➖ | ✅ | ✅ | Complete |
| env_param.md | ➖ | ➖ | ✅ | ➖ | ➖ | L3 Entity |
| config_param.md | ➖ | ➖ | ✅ | ➖ | ➖ | L3 Entity |
| user_story/ | ➖ | ➖ | ✅ | ➖ | ➖ | L3 Entity (5 instances) |
| format/ | ➖ | ➖ | ✅ | ➖ | ➖ | L3 Entity (2 instances) |
| format/procedure.md | ➖ | ➖ | ➖ | ➖ | ➖ | Entity Infrastructure |

**Current Level:** L5 (Bidirectionally Complete)
**Design Completeness:** 100%
**Implementation Status:** Complete (12 commands implemented)

### Navigation

- [Commands](command/readme.md) — command reference
- [Parameters](param/readme.md) — flag reference
- [Types](type/readme.md) — semantic type definitions
- [Parameter Groups](param_group/readme.md) — logical parameter groupings
- [Parameter Interactions](004_parameter_interactions.md) — cross-parameter constraints
- [Dictionary](002_dictionary.md) — domain vocabulary
- [User Stories](user_story/readme.md) — persona-goal scenarios
- [Formats](format/readme.md) — output format catalog
- [Environment Parameters](env_param.md) — environment variable reference
- [Config Parameters](config_param.md) — config file reference

### See Also

- [feature/001_version_management.md](../feature/001_version_management.md) — version management, architecture, constraints
- [001_design_decisions.md](../001_design_decisions.md) — CLI redesign rationale
- [tests/docs/cli/](../../tests/docs/cli/readme.md) — test case planning (commands, params, groups)
