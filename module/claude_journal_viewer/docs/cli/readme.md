# `claude_journal_viewer` CLI Documentation

### Scope

- **Purpose**: Document the `clj` command-line interface for journal event viewing and analysis.
- **Responsibility**: Reference documentation for commands, parameters, types, parameter groups, environment parameters, dictionary, and user stories.
- **In Scope**: CLI commands, parameters, types, parameter groups, dictionary, env params, user stories.
- **Out of Scope**: Journal library internals (-> `claude_journal/docs/`), CLR integration (-> `claude_runner/docs/feature/002_journaling_integration.md`).

All commands use unilang `.command param::value` syntax. The binary is `clj` (standalone) and `ast .journal.*` (super-app).

### Responsibility Table

| File | Responsibility |
|------|----------------|
| `command/` | Per-command detail pages with parameter tables and examples |
| `param/` | Per-parameter detail pages with type, defaults, and command cross-refs |
| `type/` | Per-type constraint and validation reference |
| `param_group/` | Per-group detail pages with membership and interaction rules |
| `dictionary.md` | Domain vocabulary and term definitions |
| `env_param.md` | Environment variable catalog with precedence rules |
| `user_story/` | User story index covering persona goals and acceptance criteria |

### Completion Matrix

| File | L1 | L2 | L3 | L4 | L5 | Status |
|------|----|----|----|----|----|----|
| `readme.md` | Y | Y | Y | -- | -- | L3 |
| `command/readme.md` | Y | Y | Y | -- | -- | L3 |
| `command/*.md` (8 files) | Y | Y | Y | -- | -- | L3 |
| `param/readme.md` | Y | Y | Y | -- | -- | L3 |
| `param/*.md` (28 files) | Y | Y | Y | -- | -- | L3 |
| `type/readme.md` | Y | Y | Y | -- | -- | L3 |
| `type/*.md` (11 files) | Y | Y | Y | -- | -- | L3 |
| `param_group/readme.md` | Y | Y | Y | -- | -- | L3 |
| `param_group/*.md` (5 files) | Y | Y | Y | -- | -- | L3 |
| `dictionary.md` | Y | Y | Y | -- | -- | L3 |
| `env_param.md` | Y | Y | Y | -- | -- | L3 |
| `user_story/readme.md` | Y | Y | Y | -- | -- | L3 |
| `user_story/*.md` (5 files) | Y | Y | Y | -- | -- | L3 |
**Current Level:** L3 (Specification Complete)
**Design Completeness:** 60%
**Implementation Status:** 0% (all commands planned)

### Navigation

- [Commands](command/readme.md) — What operations exist and how to invoke them
- [Parameters](param/readme.md) — What inputs control each command
- [Types](type/readme.md) — Semantic type constraints and validation rules
- [Parameter Groups](param_group/readme.md) — Related parameter sets and their coherence
- [Dictionary](dictionary.md) — Domain vocabulary
- [Environment Parameters](env_param.md) — Environment variables and precedence rules
- [User Stories](user_story/readme.md) — Persona goals, acceptance criteria, and workflows
