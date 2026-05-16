# API Doc Entity

### Scope

- **Purpose**: Document the programmatic interface of the claude_runner library surface.
- **Responsibility**: Index of API doc instances covering COMMANDS_YAML, VerbosityLevel, and register_commands contracts.
- **In Scope**: COMMANDS_YAML constant, VerbosityLevel newtype, register_commands no-op function.
- **Out of Scope**: CLI binary behavior (→ `feature/`), dependency rules (→ `invariant/`).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [Public API](001_public_api.md) | COMMANDS_YAML, VerbosityLevel, and register_commands contracts | ✅ |
| — | [procedure.md](procedure.md) | Workflow for creating and updating API doc instances | ✅ |
