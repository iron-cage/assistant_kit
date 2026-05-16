# Invariant Doc Entity

### Scope

- **Purpose**: Defines the non-functional constraints that `claude_profile` must maintain at all times.
- **Responsibility**: Documents all quality invariants with their statements, enforcement mechanisms, and violation consequences.
- **In Scope**: NFR-1, NFR-3, NFR-4, NFR-5, NFR-6 — the five non-functional requirements from the original spec.
- **Out of Scope**: Functional requirements (→ feature/), CLI design constraints (→ cli/).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [Zero Third-Party Dependencies](001_zero_third_party_deps.md) | Library path must have zero third-party crates.io dependencies | ✅ |
| 002 | [Cross-Platform Compatibility](002_cross_platform.md) | All path operations work correctly on Linux, macOS, and Windows | ✅ |
| 003 | [Clear Error Messages](003_clear_errors.md) | All errors name the relevant resource and state corrective action | ✅ |
| 004 | [No Process Execution](004_no_process_execution.md) | `std::process::Command` is forbidden anywhere in the library | ✅ |
| 005 | [Atomic Account Switching](005_atomic_switching.md) | Account switching uses write-then-rename to prevent credential corruption | ✅ |
| — | [procedure.md](procedure.md) | Workflow for creating and updating invariant doc instances | ✅ |
