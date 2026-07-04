# Invariant Doc Entity

### Scope

- **Purpose**: Document non-negotiable behavioral constraints of the claude_assets_core library that must never be violated.
- **Responsibility**: Index of invariant doc instances covering the symlink-only install mechanism.
- **In Scope**: Install operation constraints, data-loss prevention rules.
- **Out of Scope**: Feature design (→ `feature/`), CLI behavior (→ `claude_assets/docs/`).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [Symlink Only](001_symlink_only.md) | install() must use symlink(); never copy files | ✅ |
| — | [procedure.md](procedure.md) | Workflow for creating and updating invariant doc instances | ✅ |
