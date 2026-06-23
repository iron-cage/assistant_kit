# clp CLI Documentation

### Scope

- **Purpose**: Document the clp command-line interface for Claude Code account management.
- **Responsibility**: Reference documentation for commands, parameters, types, and workflows.
- **In Scope**: commands, params, types, parameter groups, output formats, env params, workflow scenarios, dictionary.
- **Out of Scope**: Implementation design (→ `feature/`), quality constraints (→ `invariant/`), test planning (→ `tests/docs/cli/`).

> **Dual purpose.** This directory serves two functions: (1) container for CLI sub-entity directories (command/, param/, type/, param_group/, user_story/, workflow_scenario/, command_noun/, command_verb/, format/); (2) standalone collection for cross-cutting CLI reference docs (001–004). Governing authority for sub-entities: individual sub-entity readme.md files. Governing authority for standalone docs: this readme.md.

Manage Claude Code account credentials: save, list, use, and delete named profiles; check token status; discover ~/.claude/ file paths.

### Responsibility Table

| Entry | Responsibility |
|-------|----------------|
| [command/](command/readme.md) | Per-namespace command specifications (account, token, credentials, usage, paths, meta) |
| [param/](param/readme.md) | Individual parameter specifications (62 params, one file each) |
| [type/](type/readme.md) | CLI type definitions (AccountName, OutputFormat, WarningThreshold, AccountSelector) |
| [param_group/](param_group/readme.md) | Parameter group semantics (Output Control, Field Presence, Fetch Behavior, Sort Control, Display Control, Account Targeting) |
| [user_story/](user_story/readme.md) | Five canonical user stories mapping personas and goals to commands |
| [workflow_scenario/](workflow_scenario/readme.md) | Legacy composed workflows (eliminated in rulebook v1.7; content migrating to user_story/) |
| [command_noun/](command_noun/readme.md) | Domain noun documentation (account, token, credentials) |
| [command_verb/](command_verb/readme.md) | Domain verb documentation (save, use, delete, status, …) |
| [format/](format/readme.md) | Output format specifications (text, json, table) |
| [003_env_param.md](003_env_param.md) | Environment variable mechanism ($PRO, $HOME, $USERPROFILE) |
| [001_config_param.md](001_config_param.md) | Configuration file mechanism (not applicable) |
| [004_parameter_interactions.md](004_parameter_interactions.md) | Cross-parameter interaction and incompatibility rules |
| [002_dictionary.md](002_dictionary.md) | Domain term definitions |

### Completion Matrix

| Directory / File | L1 | L2 | L3 | L4 | L5 | Status |
|------------------|----|----|----|----|----|--------|
| readme.md | ✅ | ✅ | ✅ | ✅ | ✅ | Complete |
| command/ | ✅ | ✅ | ✅ | ✅ | ➖ | In Progress |
| param/ | ✅ | ✅ | ✅ | ✅ | ➖ | In Progress |
| type/ | ➖ | ➖ | ✅ | ✅ | ✅ | Complete |
| param_group/ | ➖ | ➖ | ✅ | ✅ | ✅ | Complete |
| user_story/ | ➖ | ➖ | ✅ | ✅ | ✅ | Complete |
| workflow_scenario/ | ➖ | ➖ | ⚠️ | ➖ | ➖ | Legacy (eliminated in v1.7) |
| command_noun/ | ✅ | ✅ | ✅ | ➖ | ➖ | Complete |
| command_verb/ | ✅ | ✅ | ✅ | ➖ | ➖ | Complete |
| format/ | ➖ | ➖ | ✅ | ➖ | ➖ | Complete |
| 003_env_param.md | ➖ | ➖ | ✅ | ➖ | ➖ | Complete |
| 001_config_param.md | ➖ | ➖ | ✅ | ➖ | ➖ | Complete |
| 002_dictionary.md | ➖ | ✅ | ✅ | ✅ | ✅ | Complete |
| 004_parameter_interactions.md | ➖ | ➖ | ✅ | ➖ | ➖ | Complete |

**Current Level:** L4 (Test Structure Complete)
**Design Completeness:** 100%
**Implementation Status:** 100% (18/18 commands implemented)

### Navigation

- [Commands](command/readme.md) — 18 commands (17 visible + 1 hidden) across 6 namespaces
- [Parameters](param/readme.md) — 61 individual parameter specifications
- [Types](type/readme.md) — 4 CLI types (AccountName, OutputFormat, WarningThreshold, AccountSelector)
- [Parameter Groups](param_group/readme.md) — 6 groups (Output Control, Field Presence, Fetch Behavior, Sort Control, Display Control, Account Targeting)
- [User Stories](user_story/readme.md) — 5 user stories (Account Rotation, Onboarding, Quota Monitoring, Scripted Automation, Credential Diagnostics)
- [Command Nouns](command_noun/readme.md) — 3 domain nouns (account, token, credentials)
- [Command Verbs](command_verb/readme.md) — 11 domain verbs (save, use, delete, limits, relogin, rotate, renewal, inspect, assign, status, unclaim)
- [Workflow Scenarios](workflow_scenario/readme.md) — 10 composed workflows (legacy; content migrating to user_story/)
- [Output Formats](format/readme.md) — 3 formats (text, json, table)
- [Environment Parameters](003_env_param.md) — $PRO, $HOME, $USERPROFILE path resolution
- [Configuration Parameters](001_config_param.md) — not applicable (no config file)
- [Parameter Interactions](004_parameter_interactions.md) — incompatibility and override rules
- [Dictionary](002_dictionary.md) — domain term definitions

### See Also

- [tests/docs/cli/](../../tests/docs/cli/readme.md) — test case planning (commands, params, groups)
