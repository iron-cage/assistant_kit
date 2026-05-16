# Invariant Doc Entity

### Scope

- **Purpose**: Document non-negotiable behavioral constraints of the claude_assets CLI that must never be violated.
- **Responsibility**: Index of invariant doc instances covering the source root resolution requirement.
- **In Scope**: Environment variable resolution rules, error conditions for missing configuration.
- **Out of Scope**: Feature design (→ `feature/`), domain install constraints (→ `claude_assets_core/docs/invariant/`).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [Source Root Resolution](001_source_root_resolution.md) | $PRO_CLAUDE must resolve before any install or list operation | ✅ |
| — | [procedure.md](procedure.md) | Workflow for creating and updating invariant doc instances | ✅ |
