# Env Param Tests

### Scope

- **Purpose**: Document edge case coverage for individual clr env parameters.
- **Responsibility**: Index of per-env-parameter edge case test files covering env-parameter-level behavior.
- **In Scope**: All 1 clr env parameter edge case files.
- **Out of Scope**: Command-level tests (→ `command/`), parameter edge cases (→ `param/`).

Per-env-parameter edge case indices for `clr`. See [env_param.md](../../../../docs/cli/env_param.md) for specification.

### Responsibility Table

| File | Responsibility |
|------|----------------|
| 01_max_output_tokens.md | Edge cases for CLAUDE_CODE_MAX_OUTPUT_TOKENS env var injection |

### Index

| Env Parameter | File | Tests |
|---------------|------|-------|
| `CLAUDE_CODE_MAX_OUTPUT_TOKENS` | [01_max_output_tokens.md](01_max_output_tokens.md) | 6 EC |
