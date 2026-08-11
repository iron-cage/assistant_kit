# Invariant Doc Entity

### Scope

- **Purpose**: Document non-negotiable behavioral constraints of the claude_patch_core library that must never be violated.
- **Responsibility**: Index of invariant doc instances covering the pin/uninstall interaction.
- **In Scope**: Pin-blocks-uninstall constraint and its enforcement.
- **Out of Scope**: Feature design (→ `feature/`), CLI behavior (→ `claude_patch/docs/`).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [Pin Blocks Uninstall](001_pin_blocks_uninstall.md) | uninstall() must reject Pinned components until explicitly unpinned | 🔄 |
| — | [procedure.md](procedure.md) | Workflow for creating and updating invariant doc instances | ✅ |
