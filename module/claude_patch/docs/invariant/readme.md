# Invariant Doc Entity

### Scope

- **Purpose**: Document non-negotiable behavioral constraints of the claude_patch CLI that must never be violated.
- **Responsibility**: Index of invariant doc instances covering the read-only nature of the `.param.*` subject.
- **In Scope**: `.param.*` read-only constraint and its enforcement.
- **Out of Scope**: Feature design (→ `feature/`), patch-component domain semantics (→ `claude_patch_core/docs/`).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [No Param Mutation](001_no_param_mutation.md) | `.param.*` commands must never write or mutate settings | 🔄 |
| — | [procedure.md](procedure.md) | Workflow for creating and updating invariant doc instances | ✅ |
