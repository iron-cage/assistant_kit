# Invariant Doc Entity

### Scope

- **Purpose**: Document non-functional constraints that claude_runner must always satisfy.
- **Responsibility**: Index of invariant doc instances covering default flag injection and dependency constraints.
- **In Scope**: Default-on flags (`--dangerously-skip-permissions`, `-c`, `--chrome`), zero willbe dependency rule, binary dependency gating.
- **Out of Scope**: Feature behavior (→ `feature/`), API contracts (→ `api/`).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [Default Flags](001_default_flags.md) | Automatic flag injection and opt-out mechanism | ✅ |
| 002 | [Dependency Constraints](002_dep_constraints.md) | Zero willbe deps, binary deps gated by enabled, no routines.rs | ✅ |
