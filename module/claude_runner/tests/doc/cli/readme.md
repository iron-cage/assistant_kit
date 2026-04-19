# tests/doc/cli

### Scope

- **Purpose**: Test-lens documentation for the `clr` CLI — extends [`docs/cli/`](../../../docs/cli/readme.md) with test planning.
- **Responsibility**: Per-command, per-parameter, and per-group test case indices covering the `clr` binary.
- **In Scope**: Integration test specifications (`testing/`), coverage summaries, edge case catalogues.
- **Out of Scope**: CLI design reference (→ [`docs/cli/`](../../../docs/cli/readme.md)).

| Directory | Responsibility |
|-----------|----------------|
| `testing/` | Test case index files for commands, parameters, and parameter groups |

### Completion Matrix

| File | L1 | L2 | L3 | L4 | L5 | Status |
|------|----|----|----|----|----|--------|
| `readme.md` | ✅ | ✅ | ✅ | ✅ | ✅ | Complete |
| `testing/readme.md` | ➖ | ➖ | ➖ | ✅ | ✅ | Complete |
| `testing/command/*.md` (2 files) | ➖ | ➖ | ➖ | ✅ | ✅ | Complete |
| `testing/param/*.md` (5 files) | ➖ | ➖ | ➖ | ✅ | ✅ | Complete |
| `testing/param_group/*.md` (1 file) | ➖ | ➖ | ➖ | ✅ | ✅ | Complete |

**Current Level:** L5 (Test Detail Complete)

### Navigation

- [Testing](testing/readme.md)
- [CLI Design Reference](../../../docs/cli/readme.md)
