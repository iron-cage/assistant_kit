# API Doc Entity

### Scope

- **Purpose**: Document the programmatic interface for constructing and executing Claude Code commands.
- **Responsibility**: Index of API doc instances covering execution methods, return types, and error contracts.
- **In Scope**: execute(), execute_interactive(), ExecutionOutput, error handling, method parameter contracts.
- **Out of Scope**: Builder pattern design (→ `pattern/`), enum type definitions (→ `data_structure/`).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [Execution API](001_execution_api.md) | execute() and execute_interactive() contracts | ✅ |
| — | [procedure.md](procedure.md) | Workflow for creating and updating API doc instances | ✅ |
