# `claude_storage` CLI Documentation

### Scope

- **Purpose**: Document the clg command-line interface for Claude Code conversation storage exploration.
- **Responsibility**: Reference documentation for commands, parameters, types, workflows, and test planning.
- **In Scope**: commands, params, types, parameter groups, dictionary, workflows, format/, user_story/.
- **Out of Scope**: CLI tool design (→ `feature/001_cli_tool.md`), quality constraints (→ `invariant/`).

Reference documentation for the `claude_storage` CLI binary — a tool for exploring Claude Code conversation storage. All commands are read-only except `.session.ensure`, which creates the session working directory on disk.

### Responsibility Table

| File | Responsibility |
|------|----------------|
| `command/` | Per-command detail pages with full parameter tables and cross-refs |
| `param/` | Per-parameter detail pages with type, defaults, and command cross-refs |
| `type/` | Per-type constraint and parsing reference (14 type pages) |
| `002_dictionary.md` | Domain vocabulary and term definitions |
| `param_group/` | Per-group detail pages with membership, examples, and cross-refs |
| `006_workflows.md` | Usage scenarios, best practices, complexity matrix |
| `format/` | Output format catalog for export rendering modes |
| `env_param.md` | Environment variable catalog with precedence rules |
| `user_story/` | User story index covering persona goals and acceptance criteria |

### Completion Matrix

| File | L1 | L2 | L3 | L4 | L5 | Status |
|------|----|----|----|----|----|----|
| `readme.md` | ✅ | ✅ | ✅ | ✅ | ✅ | L5 |
| `command/readme.md` | ➖ | ➖ | ✅ | ✅ | ✅ | L5 |
| `command/*.md` (11 files) | ➖ | ➖ | ✅ | ✅ | ✅ | L5 |
| `param/readme.md` | ➖ | ➖ | ✅ | ✅ | ✅ | L5 |
| `param/*.md` (22 files) | ➖ | ➖ | ✅ | ✅ | ✅ | L5 |
| `002_dictionary.md` | ➖ | ✅ | ✅ | ✅ | ✅ | L5 |
| `type/readme.md` | ➖ | ➖ | ✅ | ✅ | ✅ | L5 |
| `type/*.md` (14 files) | ➖ | ➖ | ✅ | ✅ | ✅ | L5 |
| `param_group/readme.md` | ➖ | ➖ | ✅ | ✅ | ✅ | L5 |
| `param_group/*.md` (5 files) | ➖ | ➖ | ✅ | ✅ | ✅ | L5 |
| `006_workflows.md` | ➖ | ➖ | ✅ | ✅ | ✅ | L5 |
| `format/readme.md` | ➖ | ➖ | ✅ | ✅ | ✅ | L5 |
| `format/*.md` (3 files) | ➖ | ➖ | ✅ | ✅ | ✅ | L5 |
| `env_param.md` | ✅ | ✅ | ✅ | ➖ | ➖ | L3 |
| `user_story/readme.md` | ✅ | ✅ | ✅ | ✅ | ✅ | L5 |
| `user_story/*.md` (5 files) | ✅ | ✅ | ✅ | ✅ | ✅ | L5 |
**Current Level:** L5 (Implementation Complete)
**Design Completeness:** 94% (env_param.md pending test mirror at L4)
**Implementation Status:** 100% (11/11 commands implemented)

### Navigation

- [Commands](command/readme.md) — What operations exist and how to invoke them
- [Parameters](param/readme.md) — What inputs control each command
- [Types](type/readme.md) — Semantic type constraints and validation rules
- [Dictionary](002_dictionary.md) — Domain vocabulary
- [Parameter Groups](param_group/readme.md) — Related parameter sets and their coherence
- [Workflows](006_workflows.md) — Common usage patterns and best practices
- [Formats](format/readme.md) — Export output format rendering specifications
- [Environment Parameters](env_param.md) — Environment variables and precedence rules
- [User Stories](user_story/readme.md) — Persona goals, acceptance criteria, and workflows

### Related Documentation

- [`../feature/001_cli_tool.md`](../feature/001_cli_tool.md) — Crate architecture and overall design
- `../readme.md` — User-facing crate overview
- [`contract/claude_code/docs/taxonomy/readme.md`](../../../../contract/claude_code/docs/taxonomy/readme.md) — Four-level taxonomy (Project / Conversation / Session / Entry) — authoritative terminology reference
- [`contract/claude_code/docs/storage/readme.md`](../../../../contract/claude_code/docs/storage/readme.md) — Storage layout (`~/.claude/projects/`)
- `../../unilang.commands.yaml` — Machine-readable command definitions
- [tests/docs/cli/](../../tests/docs/cli/readme.md) — test case planning (commands, params, groups)
