# CLI Test Documentation

### Scope

- **Purpose**: Test planning and case indexes for the `claude_storage` CLI, organized parallel to `docs/cli/`.
- **Responsibility**: Index of CLI testing doc entities covering commands, parameters, and parameter groups.
- **In Scope**: CLI command test cases, parameter edge cases, and parameter group interaction tests.
- **Out of Scope**: CLI reference documentation (→ `docs/cli/`), implementation tests (→ `tests/*.rs`).

| Directory | Responsibility |
|-----------|----------------|
| `testing/` | Test case index files for commands, parameters, and parameter groups |

### Completion Matrix

| File | L1 | L2 | L3 | L4 | L5 | Status |
|------|----|----|----|----|----|--------|
| `readme.md` | ✅ | ✅ | ✅ | ✅ | ✅ | Complete |
| `testing/readme.md` | ➖ | ➖ | ➖ | ✅ | ✅ | Index only |
| `testing/command/*.md` (11 files) | ➖ | ➖ | ➖ | ✅ | ✅ | Complete |
| `testing/param/*.md` (20 files) | ➖ | ➖ | ➖ | ✅ | ✅ | Complete |
| `testing/param_group/*.md` (5 files) | ➖ | ➖ | ➖ | ✅ | ✅ | Complete |

**Current Level:** L5 (Test Detail Complete)

### Navigation

- [Testing](testing/readme.md)
- [CLI Design Reference](../../../docs/cli/readme.md)
