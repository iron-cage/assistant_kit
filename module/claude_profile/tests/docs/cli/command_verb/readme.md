# Test: verb:: contracts

BV-N behavioral contract test specs for clp CLI verbs. Each spec covers idempotency,
state transition, and pre-condition enforcement properties defined in `docs/cli/command_verb/`.

**BV- extension note:** BV- is a project-local element type extension not registered in
`test_surface.rulebook.md` (that file is outside the `module/claude_profile/` package scope).
Task `task/claude_profile/completed/286_command_verb_contract_test_specs.md` is the authorizing source
for the BV- prefix until a separate rulebook-update task is filed.

### Responsibility Table

| File | Verb | Idempotency | State Pattern | BV-N Cases |
|------|------|-------------|---------------|-----------|
| `01_save.md` | save | Conditional | Creates | BV-1/2/3 |
| `02_use.md` | use | Conditional | Transitions | BV-1/2/3 |
| `03_delete.md` | delete | Conditional | Removes | BV-1/2/3 |
| `04_limits.md` | limits | Yes | Reads | BV-1/2/3 |
| `05_relogin.md` | relogin | No | Transitions (in-place) | BV-1/2/3 |
| `06_rotate.md` | rotate | No | Transitions | BV-1/2/3 |
| `07_renewal.md` | renewal | Yes | Accumulates | BV-1/2/3 |
| `08_inspect.md` | inspect | Yes | Reads | BV-1/2/3 |
| `09_assign.md` | assign | Yes | Accumulates | BV-1/2/3 |
| `10_status.md` | status | Yes | Reads | BV-1/2/3/4 (2 nouns) |
| `11_unclaim.md` | unclaim | Yes | Metadata mutation | BV-1/2/3 |

### Coverage Summary

| Verb Files | Min Cases | Total |
|------------|-----------|-------|
| 11 | ≥ 3 each | 34 (10 × 3 + 1 × 4) |

### See Also

- [docs/cli/command_verb/](../../../../docs/cli/command_verb/readme.md) — behavioral contract source docs
