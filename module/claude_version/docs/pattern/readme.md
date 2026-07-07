# Pattern Doc Entity

### Scope

- **Purpose**: Document architectural patterns applied in the claude_version design.
- **Responsibility**: Index of pattern doc instances covering version lock strategy and parameter-trace instrumentation.
- **In Scope**: 8-layer version lock problem/solution/applicability/consequences; parameter-trace convention for mutating functions.
- **Out of Scope**: Feature behavior (→ `feature/`), type inference algorithm (→ `algorithm/`).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [Version Lock](001_version_lock.md) | 8-layer protection for pinned Claude Code versions | ✅ |
| 002 | [Parameter Trace](002_parameter_trace.md) | Unconditional stderr trace on all 10 mutating functions | ✅ |
