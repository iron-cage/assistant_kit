# Invariant Doc Entity

### Scope

- **Purpose**: Defines the non-functional constraints that `claude_profile` must maintain at all times.
- **Responsibility**: Documents all quality invariants with their statements, enforcement mechanisms, and violation consequences.
- **In Scope**: NFR-1, NFR-3, NFR-4, NFR-5, NFR-6 — the five non-functional requirements from the original spec.
- **Out of Scope**: Functional requirements (→ feature/), CLI design constraints (→ cli/).

### Overview Table

| ID | Name | NFR | Status |
|----|------|-----|--------|
| 001 | [Zero Third-Party Dependencies](001_zero_third_party_deps.md) | NFR-1 | ✅ |
| 002 | [Cross-Platform Compatibility](002_cross_platform.md) | NFR-3 | ✅ |
| 003 | [Clear Error Messages](003_clear_errors.md) | NFR-4 | ✅ |
| 004 | [No Process Execution](004_no_process_execution.md) | NFR-5 | ✅ |
| 005 | [Atomic Account Switching](005_atomic_switching.md) | NFR-6 | ✅ |
