# clv CLI Documentation

### Scope

- **Purpose**: Document the clv command-line interface for Claude Code version and settings management.
- **Responsibility**: Reference documentation for commands, parameters, types, output formats, and user stories.
- **In Scope**: commands, params, types, parameter groups, dictionary, user stories, output formats, environment parameters, config parameters, parameter interactions.
- **Out of Scope**: Implementation design and behavioral contracts — validation rules, exit code semantics, pipeline architecture (→ `feature/`), design rationale (→ `collection/001_design_decisions.md`), test planning (→ `tests/docs/cli/`).

Manage Claude Code installation: versions, processes, and settings.

### Usage

```sh
clv <.command> [param::value ...]
```

### Completion Matrix

| File | L1 | L2 | L3 | L4 | L5 | Status |
|------|----|----|----|----|----|--------|
| readme.md | ✅ | ✅ | ✅ | ✅ | ✅ | Complete |
| command/ | ✅ | ✅ | ✅ | ✅ | ✅ | Complete |
| param/ | ✅ | ✅ | ✅ | ✅ | ✅ | Complete |
| type/ | ✅ | ✅ | ✅ | ✅ | ✅ | Complete |
| param_group/ | ✅ | ✅ | ✅ | ✅ | ✅ | Complete |
| dictionary.md | ➖ | ✅ | ➖ | ➖ | ➖ | Complete |
| 004_parameter_interactions.md | ➖ | ➖ | ➖ | ✅ | ✅ | Complete |
| env_param.md | ➖ | ➖ | ✅ | ➖ | ➖ | L3 Entity |
| config_param.md | ➖ | ➖ | ✅ | ➖ | ➖ | L3 Entity |
| user_story/ | ➖ | ➖ | ✅ | ➖ | ➖ | L3 Entity |
| format/ | ➖ | ➖ | ✅ | ➖ | ➖ | L3 Entity |

**Current Level:** L5 (Test Detail Complete)
**Design Completeness:** 100%
**Implementation Status:** Complete

### Navigation

- [Commands](command/readme.md) — command reference
- [Parameters](param/readme.md) — flag reference
- [Types](type/readme.md) — semantic type definitions
- [Parameter Groups](param_group/readme.md) — logical parameter groupings
- [Parameter Interactions](004_parameter_interactions.md) — cross-parameter constraints
- [Dictionary](dictionary.md) — domain vocabulary
- [User Stories](user_story/readme.md) — persona-goal scenarios
- [Formats](format/readme.md) — output format catalog
- [Environment Parameters](env_param.md) — environment variable reference
- [Config Parameters](config_param.md) — config file reference

### See Also

- [feature/001_version_management.md](../feature/001_version_management.md) — version management, architecture, constraints
- [collection/001_design_decisions.md](../collection/001_design_decisions.md) — CLI redesign rationale
- [tests/docs/cli/](../../tests/docs/cli/readme.md) — test case planning (commands, params, groups)

### Exception Records

**Exception to `cli_doc_des.rulebook.md § Architecture : Entity Type Classes` (format test mirror):**
Project retains `tests/docs/cli/format/` as a dedicated testing tier. Rationale: format rendering contracts (verbosity interaction, JSON validity, field presence invariants) are not fully captured by command integration tests alone; a dedicated format test layer makes format contract changes reviewable in isolation.

**Exception to `cli_doc_des.rulebook.md § Completion Levels : Level 3` (`command_noun` / `command_verb` entities):**
The clv CLI uses a hybrid command pattern: namespace-qualified commands (`.version.*`, `.processes.*`, `.settings.*`) coexist with pure operation commands (`.help`, `.status`, `.config`). `command_noun` and `command_verb` entities are deferred because domain noun lifecycle and verb behavioral contracts are already captured within `command/<namespace>.md` files; a separate `command_noun/` layer would duplicate that content without adding structure.
