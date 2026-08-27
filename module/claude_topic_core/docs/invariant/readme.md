# Invariant Doc Entity

### Scope

- **Purpose**: Record the constraints that must hold for every build of `claude_topic_core`, each with a mechanical check that fails the test suite when it stops holding.
- **Responsibility**: Index of invariant doc instances covering what the registry means and how a topic must be addressed.
- **In Scope**: The authority relationship between the registry and the session file; the `( name, mode )` pair as the unit of addressing.
- **Out of Scope**: Behavioral capabilities (→ `feature/`), signature-level contracts (→ `api/`).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [Registry Non-Authoritative](001_registry_non_authoritative.md) | The registry is an index; the session file is the authority | ✅ |
| 002 | [Mode Travels With Name](002_mode_travels_with_name.md) | A topic is a `( name, mode )` pair, never a name | ✅ |
| — | [procedure.md](procedure.md) | Workflow for creating and updating invariant doc instances | ✅ |
