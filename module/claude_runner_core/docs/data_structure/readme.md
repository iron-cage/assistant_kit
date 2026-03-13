# Data Structure Doc Entity

### Scope

- **Purpose**: Document the type-safe enum types used to configure Claude Code command parameters.
- **Responsibility**: Index of data structure doc instances covering configuration enum semantics and valid values.
- **In Scope**: ActionMode, LogLevel, OutputFormat, InputFormat, PermissionMode, EffortLevel — values and CLI string mappings.
- **Out of Scope**: Builder methods that accept these types (→ `api/`), execution flow (→ `feature/`).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [Command Types](001_command_types.md) | Configuration enums: ActionMode, LogLevel, OutputFormat, etc. | ✅ |
