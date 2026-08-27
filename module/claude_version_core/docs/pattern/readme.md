# Pattern Doc Entity

### Scope

- **Purpose**: Document architectural patterns applied in the `claude_version_core` crate.
- **Responsibility**: Index of pattern doc instances covering the parameter-trace instrumentation convention.
- **In Scope**: Unconditional stderr parameter-trace convention applied to every public mutating function in this crate and the shared `claude_core::settings_io` module.
- **Out of Scope**: Feature/CLI-command behavior (→ `claude_version/docs/feature/`), config resolution algorithm (→ `algorithm/`).

### Responsibility Table

| File | Responsibility |
|------|---------------|
| readme.md | Index and navigation for pattern instances |
| 002_parameter_trace.md | Unconditional stderr trace on all 11 mutating functions |

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 002 | [Parameter Trace](002_parameter_trace.md) | Unconditional stderr trace on all 11 mutating functions | ✅ |
