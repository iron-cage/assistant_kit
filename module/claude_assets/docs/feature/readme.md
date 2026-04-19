# Feature Doc Entity

### Scope

- **Purpose**: Document user-facing capabilities of the claude_assets CLI for operators managing Claude Code artifact installations.
- **Responsibility**: Index of feature doc instances covering the four CLI commands and adapter preprocessing behavior.
- **In Scope**: `.list`, `.install`, `.uninstall`, `.kinds` command design; bool normalisation; 5-phase unilang pipeline.
- **Out of Scope**: Domain install logic (→ `claude_assets_core/docs/feature/`), invariant constraints (→ `invariant/`).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [Asset CLI](001_asset_cli.md) | Four CLI commands for listing, installing, and uninstalling artifacts | ✅ |
