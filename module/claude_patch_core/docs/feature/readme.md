# Feature Doc Entity

### Scope

- **Purpose**: Document user-facing capabilities of the claude_patch_core library for consumers building patch-management tooling.
- **Responsibility**: Index of feature doc instances covering the patch component model and its install/uninstall/pin/unpin semantics.
- **In Scope**: PatchKind taxonomy, component state machine, operation semantics.
- **Out of Scope**: CLI command design (→ `claude_patch/docs/feature/`), invariant constraints (→ `invariant/`).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [Patch Component Model](001_patch_component_model.md) | Kind taxonomy and install/uninstall/pin/unpin state machine | 🔄 |
| — | [procedure.md](procedure.md) | Workflow for creating and updating feature doc instances | ✅ |
