# Feature Doc Entity

### Scope

- **Purpose**: Document user-facing CLI capabilities of the claude_patch (`clt`) binary for consumers managing Claude Code patch components and inspecting parameter provenance.
- **Responsibility**: Index of feature doc instances covering the `.patch.*` and `.param.*` command subjects.
- **In Scope**: Command signatures, argument semantics, delegation to claude_patch_core and claude_version_core.
- **Out of Scope**: Patch-component domain semantics (→ `claude_patch_core/docs/feature/`), invariant constraints (→ `invariant/`).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [Patch CLI](001_patch_cli.md) | `.patch.*` component lifecycle commands | 🔄 |
| 002 | [Param CLI](002_param_cli.md) | `.param.*` read-only provenance inspection commands | 🔄 |
| — | [procedure.md](procedure.md) | Workflow for creating and updating feature doc instances | ✅ |
