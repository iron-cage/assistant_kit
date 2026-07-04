# Env Param Tests

### Scope

- **Purpose**: Document edge case coverage for individual clr env parameters.
- **Responsibility**: Index of per-env-parameter edge case test files covering env-parameter-level behavior.
- **In Scope**: All clr env parameter edge case entries (CLR_* input vars + subprocess-injected vars).
- **Out of Scope**: Command-level tests (→ `command/`), parameter edge cases (→ `param/`).

Per-env-parameter edge case indices for `clr`. See [env_param.md](../../../../docs/cli/env_param.md) for specification.

### Responsibility Table

| Name | Purpose | Status |
|------|---------|--------|
| `01_max_output_tokens.md` | Edge cases for CLAUDE_CODE_MAX_OUTPUT_TOKENS subprocess env var | ✅ |
| `02_clr_input_vars.md` | Edge cases for all 40 CLR_* input env var fallbacks (E01–E40) | ✅ |
| `03_auto_compact_window.md` | Edge cases for `CLAUDE_CODE_AUTO_COMPACT_WINDOW` injection and `--no-compact-window` opt-out | ⏳ |
