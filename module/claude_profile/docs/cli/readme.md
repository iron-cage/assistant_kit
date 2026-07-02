# clp CLI Documentation

### Scope

- **Purpose**: Document the clp command-line interface for Claude Code account management.
- **Responsibility**: Reference documentation for commands, parameters, types, and workflows.
- **In Scope**: commands, params, types, parameter groups, output formats, env params, user stories, dictionary.
- **Out of Scope**: Implementation design (→ `feature/`), quality constraints (→ `invariant/`), test planning (→ `tests/docs/cli/`).

> **Dual purpose.** This directory serves two functions: (1) container for CLI sub-entity directories (command/, param/, type/, param_group/, user_story/, command_noun/, command_verb/, format/); (2) standalone collection for cross-cutting CLI reference docs (001–004). Governing authority for sub-entities: individual sub-entity readme.md files. Governing authority for standalone docs: this readme.md.

Manage Claude Code account credentials: save, list, use, and delete named profiles; check token status; discover ~/.claude/ file paths.

### Responsibility Table

| Entry | Responsibility |
|-------|----------------|
| [command/](command/readme.md) | Per-namespace command specifications (account, token, credentials, usage, paths, meta) |
| [param/](param/readme.md) | Individual parameter specifications (58 active params, 63 files; 5 REMOVED: 013, 032, 053, 056, 057) |
| [type/](type/readme.md) | CLI type definitions (AccountName, OutputFormat, WarningThreshold, AccountSelector) |
| [param_group/](param_group/readme.md) | Parameter group semantics (Output Control, Field Presence, Fetch Behavior, Sort Control, Display Control, Account Targeting) |
| [user_story/](user_story/readme.md) | Five canonical user stories mapping personas and goals to commands |
| [workflow_scenario/](workflow_scenario/readme.md) | Legacy composed workflows (eliminated in rulebook v1.7; content migrated to user_story/) |
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
| command/ | ✅ | ✅ | ✅ | ✅ | ✅ | Complete |
| param/ | ✅ | ✅ | ✅ | ✅ | ✅ | Complete |
| type/ | ➖ | ➖ | ✅ | ✅ | ✅ | Complete |
| param_group/ | ➖ | ➖ | ✅ | ✅ | ✅ | Complete |
| user_story/ | ➖ | ➖ | ✅ | ✅ | ✅ | Complete |
| workflow_scenario/ | ➖ | ➖ | ➖ | ➖ | ➖ | Eliminated (v1.7) — 0 instances |
| command_noun/ | ✅ | ✅ | ✅ | ➖ | ➖ | Complete |
| command_verb/ | ✅ | ✅ | ✅ | ➖ | ➖ | Complete |
| format/ | ➖ | ➖ | ✅ | ➖ | ➖ | Complete |
| 003_env_param.md | ➖ | ➖ | ✅ | ➖ | ➖ | Complete |
| 001_config_param.md | ➖ | ➖ | ✅ | ➖ | ➖ | Complete |
| 002_dictionary.md | ➖ | ✅ | ✅ | ✅ | ✅ | Complete |
| 004_parameter_interactions.md | ➖ | ➖ | ✅ | ➖ | ➖ | Complete |

**Current Level:** L5 (Test Detail Complete)
**Design Completeness:** 100%
**Implementation Status:** 100% (18/18 active commands; 2 REMOVED: .account.assign, .account.unclaim)

### Navigation

- [Commands](command/readme.md) — 18 commands (16 visible + 2 hidden) across 6 namespaces
- [Parameters](param/readme.md) — 58 active parameter specifications (63 files; 5 REMOVED: 013, 032, 053, 056, 057)
- [Types](type/readme.md) — 4 CLI types (AccountName, OutputFormat, WarningThreshold, AccountSelector)
- [Parameter Groups](param_group/readme.md) — 6 groups (Output Control, Field Presence, Fetch Behavior, Sort Control, Display Control, Account Targeting)
- [User Stories](user_story/readme.md) — 5 user stories (Account Rotation, Onboarding, Quota Monitoring, Scripted Automation, Credential Diagnostics)
- [Command Nouns](command_noun/readme.md) — 3 domain nouns (account, token, credentials)
- [Command Verbs](command_verb/readme.md) — 8 active domain verbs (save, use, delete, limits, relogin, renewal, inspect, status; 1 DEPRECATED: rotate Feature 038; 2 REMOVED: assign Feature 037, unclaim Feature 064)
- [Output Formats](format/readme.md) — 3 formats (text, json, table)
- [Environment Parameters](003_env_param.md) — $PRO, $HOME, $USERPROFILE path resolution
- [Configuration Parameters](001_config_param.md) — not applicable (no config file)
- [Parameter Interactions](004_parameter_interactions.md) — incompatibility and override rules
- [Dictionary](002_dictionary.md) — domain term definitions

### See Also

- [tests/docs/cli/](../../tests/docs/cli/readme.md) — test case planning (commands, params, groups)
