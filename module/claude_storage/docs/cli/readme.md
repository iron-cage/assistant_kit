# `claude_storage` CLI Documentation

Reference documentation for the `claude_storage` CLI binary — a read-only tool for exploring Claude Code conversation storage.

## Responsibility Table

| File | Responsibility |
|------|----------------|
| `commands.md` | All command specs, syntax, parameters, exit codes, examples |
| `params.md` | Parameter definitions, types, validation rules, cross-refs |
| `types.md` | Semantic type system with constants, parsing, methods |
| `dictionary.md` | Domain vocabulary and term definitions |
| `parameter_groups.md` | Shared parameter groups with coherence tests |
| `workflows.md` | Usage scenarios, best practices, complexity matrix |
| `format/` | Output format catalog for export rendering modes |
| `testing/` | Test case indexes for all commands, params, and groups |

## Completion Matrix

| File | L1 | L2 | L3 | L4 | L5 | Status |
|------|----|----|----|----|----|----|
| `readme.md` | ✅ | ✅ | ✅ | ✅ | ➖ | Complete |
| `commands.md` | ✅ | ✅ | ✅ | ✅ | ➖ | Complete |
| `params.md` | ✅ | ✅ | ✅ | ✅ | ➖ | Complete |
| `dictionary.md` | ➖ | ✅ | ✅ | ✅ | ➖ | Complete |
| `types.md` | ➖ | ➖ | ✅ | ✅ | ➖ | Complete |
| `parameter_groups.md` | ➖ | ➖ | ✅ | ✅ | ➖ | Complete |
| `workflows.md` | ➖ | ➖ | ✅ | ✅ | ➖ | Complete |
| `format/readme.md` | ➖ | ➖ | ✅ | ✅ | ➖ | Complete |
| `format/*.md` (3 files) | ➖ | ➖ | ✅ | ✅ | ➖ | Complete |
| `testing/readme.md` | ➖ | ➖ | ➖ | ✅ | ✅ | Index only |
| `testing/command/*.md` (13 files) | ➖ | ➖ | ➖ | ✅ | ✅ | Complete |
| `testing/param/*.md` (20 files) | ➖ | ➖ | ➖ | ✅ | ✅ | Complete |
| `testing/param_group/*.md` (5 files) | ➖ | ➖ | ➖ | ✅ | ✅ | Complete |

**Current Level:** L5 (Tests Complete for existing commands)
**Design Completeness:** 100% (5/5 levels passed)
**Implementation Status:** 100% (13/13 commands implemented; 1 deprecated)

## Navigation

- [Commands](commands.md) — What operations exist and how to invoke them
- [Parameters](params.md) — What inputs control each command
- [Types](types.md) — Semantic type constraints and validation rules
- [Dictionary](dictionary.md) — Domain vocabulary
- [Parameter Groups](parameter_groups.md) — Related parameter sets and their coherence
- [Workflows](workflows.md) — Common usage patterns and best practices
- [Formats](format/readme.md) — Export output format rendering specifications
- [Testing](testing/readme.md) — Test case indexes for all commands, params, and groups

## Related Documentation

- [`../feature/001_cli_tool.md`](../feature/001_cli_tool.md) — Crate architecture and overall design
- `../readme.md` — User-facing crate overview
- [`docs/claude_code/storage_organization.md`](../../../../docs/claude_code/storage_organization.md) — Storage layout (`~/.claude/projects/`)
- `../../unilang.commands.yaml` — Machine-readable command definitions
