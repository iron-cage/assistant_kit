# clp CLI Documentation

### Scope

- **Purpose**: Document the clp command-line interface for Claude Code account management.
- **Responsibility**: Reference documentation for commands, parameters, types, and workflows.
- **In Scope**: commands, params, types, parameter groups, output formats, env params, workflow scenarios, dictionary.
- **Out of Scope**: Implementation design (→ `feature/`), quality constraints (→ `invariant/`), test planning (→ `tests/docs/cli/`).

Manage Claude Code account credentials: save, list, use, and delete named profiles; check token status; discover ~/.claude/ file paths.

### Responsibility Table

| Entry | Responsibility |
|-------|----------------|
| [command/](command/readme.md) | Per-namespace command specifications (account, token, credentials, usage, paths, meta) |
| [param/](param/readme.md) | Individual parameter specifications (24 params, one file each) |
| [type/](type/readme.md) | CLI type definitions (AccountName, OutputFormat, WarningThreshold, AccountSelector) |
| [param_group/](param_group/readme.md) | Parameter group semantics (Output Control, Field Presence, Fetch Behavior) |
| [workflow_scenario/](workflow_scenario/readme.md) | Composed command workflows for real operational tasks |
| [format/](format/readme.md) | Output format specifications (text, json, table) |
| [env_param.md](env_param.md) | Environment variable mechanism ($PRO, $HOME, $USERPROFILE) |
| [config_param.md](config_param.md) | Configuration file mechanism (not applicable) |
| [parameter_interactions.md](parameter_interactions.md) | Cross-parameter interaction and incompatibility rules |
| [dictionary.md](dictionary.md) | Domain term definitions |

### Completion Matrix

| Directory / File | L1 | L2 | L3 | L4 | L5 | Status |
|------------------|----|----|----|----|----|--------|
| readme.md | ✅ | ✅ | ✅ | ✅ | ✅ | Complete |
| command/ | ✅ | ✅ | ✅ | ✅ | ✅ | Complete |
| param/ | ✅ | ✅ | ✅ | ✅ | ✅ | Complete |
| type/ | ➖ | ➖ | ✅ | ✅ | ✅ | Complete |
| param_group/ | ➖ | ➖ | ✅ | ✅ | ✅ | Complete |
| workflow_scenario/ | ➖ | ➖ | ✅ | ✅ | ✅ | Complete |
| format/ | ➖ | ➖ | ✅ | ➖ | ➖ | Complete |
| env_param.md | ➖ | ➖ | ✅ | ➖ | ➖ | Complete |
| config_param.md | ➖ | ➖ | ✅ | ➖ | ➖ | Complete |
| dictionary.md | ➖ | ✅ | ✅ | ✅ | ✅ | Complete |
| parameter_interactions.md | ➖ | ➖ | ✅ | ➖ | ➖ | Complete |

**Current Level:** L4 (Workflow Complete)
**Design Completeness:** 100%
**Implementation Status:** 100% (13/13 commands implemented)

### Navigation

- [Commands](command/readme.md) — 13 commands across 6 namespaces
- [Parameters](param/readme.md) — 24 individual parameter specifications
- [Types](type/readme.md) — 4 CLI types (AccountName, OutputFormat, WarningThreshold, AccountSelector)
- [Parameter Groups](param_group/readme.md) — 3 groups (Output Control, Field Presence, Fetch Behavior)
- [Workflow Scenarios](workflow_scenario/readme.md) — 10 composed workflows
- [Output Formats](format/readme.md) — 3 formats (text, json, table)
- [Environment Parameters](env_param.md) — $PRO, $HOME, $USERPROFILE path resolution
- [Configuration Parameters](config_param.md) — not applicable (no config file)
- [Parameter Interactions](parameter_interactions.md) — incompatibility and override rules
- [Dictionary](dictionary.md) — domain term definitions

### See Also

- [tests/docs/cli/](../../tests/docs/cli/readme.md) — test case planning (commands, params, groups)
