# Invariant Doc Entity

### Scope

- **Purpose**: Record the constraints that must hold for every build of `claude_session_core`, each with a mechanical check that fails the test suite when it stops holding.
- **Responsibility**: Index of invariant doc instances covering the four-clause liveness predicate and the edge-triggered turn-boundary rule.
- **In Scope**: What "alive" means for a recorded PID; when a status observation may be reported as a turn boundary.
- **Out of Scope**: Behavioral capabilities (→ `feature/`), signature-level contracts (→ `api/`).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [Liveness Four Clauses](001_liveness_four_clauses.md) | A PID is live only if all four clauses hold | ✅ |
| 002 | [First Sighting Never Settles](002_first_sighting_never_settles.md) | The first observation of a session is never a turn boundary | ✅ |
| — | [procedure.md](procedure.md) | Workflow for creating and updating invariant doc instances | ✅ |
