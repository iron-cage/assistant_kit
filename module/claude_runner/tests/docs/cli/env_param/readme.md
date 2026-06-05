# Env Param Tests

### Scope

- **Purpose**: Document edge case coverage for individual clr env parameters.
- **Responsibility**: Index of per-env-parameter edge case test files covering env-parameter-level behavior.
- **In Scope**: All 30 clr env parameter edge case files (29 CLR_* input vars + 1 subprocess var).
- **Out of Scope**: Command-level tests (→ `command/`), parameter edge cases (→ `param/`).

Per-env-parameter edge case indices for `clr`. See [env_param.md](../../../../docs/cli/env_param.md) for specification.

### Responsibility Table

| Name | Purpose | Status |
|------|---------|--------|
| `01_max_output_tokens.md` | Edge cases for CLAUDE_CODE_MAX_OUTPUT_TOKENS subprocess env var | ✅ |
| `02_clr_input_vars.md` | Edge cases for all 29 CLR_* input env var fallbacks (E01–E29) | ✅ |
