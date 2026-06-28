# Pitfall Test Surface

### Scope

- **Purpose**: Test case specifications for claude_version pitfall doc instances.
- **Responsibility**: Per-pitfall PF- test specs verifying the documented trap is avoided by the implementation.
- **In Scope**: Trap avoidance verification, mitigation behavior confirmation (PF- prefix, min 3 cases per spec).
- **Out of Scope**: Pattern tests (→ `../pattern/`), feature tests (→ `../feature/`).

### Overview Table

| Name | Purpose | Status |
|------|---------|--------|
| 001_version_lock_chmod.md | PF- test cases verifying chmod side-effect trap is avoided by install/guard | ✅ |
| 002_symlink_retarget.md | PF- test cases verifying symlink retarget bypass is neutralized by Layer 4 purge | ✅ |
| procedure.md | Workflow for creating and updating pitfall test specs | ✅ |
