# Invariant Doc Entity

### Scope

- **Purpose**: Record the constraints that must hold for every build of `claude_daemon_core`, each with a mechanical check that fails the test suite when it stops holding.
- **Responsibility**: Index of invariant doc instances covering the protocol line cap and the session-addressing key.
- **In Scope**: How much a single protocol line may consume; what identifies a session across a re-host.
- **Out of Scope**: Behavioral capabilities (→ `feature/`), signature-level contracts (→ `api/`).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [Capped Line Reads](001_capped_line_reads.md) | No protocol read may grow without bound | ✅ |
| 002 | [Conversation Id Key](002_conversation_id_key.md) | Sessions are addressed by conversation id, never PID | ✅ |
| — | [procedure.md](procedure.md) | Workflow for creating and updating invariant doc instances | ✅ |
