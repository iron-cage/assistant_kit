# `claude_storage` CLI Documentation

### Scope

- **Purpose**: Document the clg command-line interface for Claude Code conversation storage exploration.
- **Responsibility**: Reference documentation for commands, parameters, types, workflows, and test planning.
- **In Scope**: commands, params, types, parameter groups, dictionary, workflows, format/.
- **Out of Scope**: CLI tool design (→ `feature/001_cli_tool.md`), quality constraints (→ `invariant/`).

Reference documentation for the `claude_storage` CLI binary — a tool for exploring Claude Code conversation storage. All commands are read-only except `.session.ensure`, which creates the session working directory on disk.

### Responsibility Table

| File | Responsibility |
|------|----------------|
| `001_commands.md` | All command specs, syntax, parameters, exit codes, examples |
| `004_params.md` | Parameter definitions, types, validation rules, cross-refs |
| `005_types.md` | Semantic type system with constants, parsing, methods |
| `002_dictionary.md` | Domain vocabulary and term definitions |
| `003_parameter_groups.md` | Shared parameter groups with coherence tests |
| `006_workflows.md` | Usage scenarios, best practices, complexity matrix |
| `format/` | Output format catalog for export rendering modes |

### Completion Matrix

| File | L1 | L2 | L3 | L4 | L5 | Status |
|------|----|----|----|----|----|----|
| `readme.md` | ✅ | ✅ | ✅ | ✅ | ➖ | Complete |
| `001_commands.md` | ✅ | ✅ | ✅ | ✅ | ➖ | Complete |
| `004_params.md` | ✅ | ✅ | ✅ | ✅ | ➖ | Complete |
| `002_dictionary.md` | ➖ | ✅ | ✅ | ✅ | ➖ | Complete |
| `005_types.md` | ➖ | ➖ | ✅ | ✅ | ➖ | Complete |
| `003_parameter_groups.md` | ➖ | ➖ | ✅ | ✅ | ➖ | Complete |
| `006_workflows.md` | ➖ | ➖ | ✅ | ✅ | ➖ | Complete |
| `format/readme.md` | ➖ | ➖ | ✅ | ✅ | ➖ | Complete |
| `format/*.md` (3 files) | ➖ | ➖ | ✅ | ✅ | ➖ | Complete |
**Current Level:** L4 (Specification Complete)
**Design Completeness:** 100% (4/4 levels passed)
**Implementation Status:** 100% (11/11 commands implemented; 0 deprecated)

### Navigation

- [Commands](001_commands.md) — What operations exist and how to invoke them
- [Parameters](004_params.md) — What inputs control each command
- [Types](005_types.md) — Semantic type constraints and validation rules
- [Dictionary](002_dictionary.md) — Domain vocabulary
- [Parameter Groups](003_parameter_groups.md) — Related parameter sets and their coherence
- [Workflows](006_workflows.md) — Common usage patterns and best practices
- [Formats](format/readme.md) — Export output format rendering specifications
### Related Documentation

- [`../feature/001_cli_tool.md`](../feature/001_cli_tool.md) — Crate architecture and overall design
- `../readme.md` — User-facing crate overview
- [`contract/claude_code/docs/taxonomy/readme.md`](../../../../contract/claude_code/docs/taxonomy/readme.md) — Four-level taxonomy (Project / Conversation / Session / Entry) — authoritative terminology reference
- [`contract/claude_code/docs/storage/readme.md`](../../../../contract/claude_code/docs/storage/readme.md) — Storage layout (`~/.claude/projects/`)
- `../../unilang.commands.yaml` — Machine-readable command definitions
- [tests/doc/cli/](../../tests/doc/cli/readme.md) — test case planning (commands, params, groups)
