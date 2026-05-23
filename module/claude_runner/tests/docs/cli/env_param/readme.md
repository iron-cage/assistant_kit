# Env Param Tests

### Scope

- **Purpose**: Document edge case coverage for individual clr env parameters.
- **Responsibility**: Index of per-env-parameter edge case test files covering env-parameter-level behavior.
- **In Scope**: All 29 clr env parameter edge case files (28 CLR_* input vars + 1 subprocess var).
- **Out of Scope**: Command-level tests (→ `command/`), parameter edge cases (→ `param/`).

Per-env-parameter edge case indices for `clr`. See [003_env_param.md](../../../../docs/cli/003_env_param.md) for specification.

### Responsibility Table

| File | Responsibility |
|------|----------------|
| 001_max_output_tokens.md | Edge cases for CLAUDE_CODE_MAX_OUTPUT_TOKENS subprocess env var |
| 002_clr_input_vars.md | Edge cases for all 28 CLR_* input env var fallbacks (E01–E28) |

### Index

| Env Parameter | File | Tests |
|---------------|------|-------|
| `CLAUDE_CODE_MAX_OUTPUT_TOKENS` | [001_max_output_tokens.md](001_max_output_tokens.md) | 6 EC |
| CLR_* (28 vars) | [002_clr_input_vars.md](002_clr_input_vars.md) | 28 EC |
