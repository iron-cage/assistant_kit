# Feature Collection

### Scope

- **Purpose**: Document user-facing capabilities of the claude_runner crate for CLI users and automation consumers.
- **Responsibility**: Index of feature doc instances covering the clr binary tool design.
- **In Scope**: Execution modes, default flags, YAML library surface, CLI flag behavior.
- **Out of Scope**: Dependency constraints (→ `invariant/`), public API contracts (→ `api/`).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [Runner Tool](001_runner_tool.md) | clr binary design: modes, default flags, and YAML library | ✅ |
| 002 | [Journaling Integration](002_journaling_integration.md) | Integration with claude_journal for automatic event journaling | ✅ |
| 003 | [Retry Hierarchy](003_retry_hierarchy.md) | 3-tier retry resolution for 6 error classes | ✅ |
| 004 | [JSON Config Loading](004_json_config.md) | JSON file and stdin pipe loading for all clr parameters | ✅ |
| — | [procedure.md](procedure.md) | Workflow for creating and updating feature doc instances | ✅ |
