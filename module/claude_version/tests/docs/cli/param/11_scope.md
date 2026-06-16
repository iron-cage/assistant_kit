# Test: `scope::`

Edge case coverage for the `scope::` parameter. See [param/11_scope.md](../../../../docs/cli/param/11_scope.md) for specification.

### Scope

- **Purpose**: Edge case tests for the `scope::` parameter.
- **Responsibility**: Boundary values, invalid inputs, and default behavior for `scope::`.
- **Commands:** `.config`
- **In Scope**: Single-parameter edge cases, validation errors, default behavior.
- **Out of Scope**: Command integration (-> `../command/`), group interactions (-> `../param_group/`).

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `scope::user` (default) writes to `~/.claude/settings.json` | Valid: default |
| EC-2 | `scope::project` writes to `{cwd}/.claude/settings.json` | Valid: project |
| EC-3 | `scope::invalid` -> exit 1, unknown scope | Invalid Value |
| EC-4 | `scope::` (empty value) -> exit 1 | Empty Value |
| EC-5 | `scope::user` without `key::` and `value::` -> exit 1, scope only applies to writes | Invalid Combination |
| EC-6 | `scope::project` with `key::K value::V` creates `.claude/` directory if absent | Project Creation |
| EC-7 | `scope::project` in show-all mode (no key::) -> exit 1 | Invalid Combination |

## Test Coverage Summary

- Valid (default scope): 1 test
- Valid (project scope): 1 test
- Invalid Value: 1 test
- Empty Value: 1 test
- Invalid Combination: 2 tests
- Project Creation: 1 test

**Total:** 7 edge cases

**Behavioral Divergence Pair:** EC-1 (`scope::user` -> writes to user config) <-> EC-2 (`scope::project` -> writes to project config)
