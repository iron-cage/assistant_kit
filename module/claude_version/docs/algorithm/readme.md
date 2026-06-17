# Algorithm Collection

### Scope

- **Purpose**: Document algorithms used in the claude_version implementation.
- **Responsibility**: Index of algorithm doc instances covering settings type inference and config resolution.
- **In Scope**: value:: type inference rules, precedence order, edge cases; 4-layer config resolution algorithm.
- **Out of Scope**: Settings JSON I/O design (→ `feature/003_settings_management.md`), CLI parsing (→ `feature/005_cli_design.md`).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [Settings Type Inference](001_settings_type_inference.md) | Type inference rules for value:: parameter in settings.set | ✅ |
| 002 | [Config Resolution](002_config_resolution.md) | 4-layer resolution: env → project → user → catalog default | ✅ |
