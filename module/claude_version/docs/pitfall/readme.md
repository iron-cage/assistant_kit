# Pitfall Doc Entity

### Scope

- **Purpose**: Document confirmed design traps discovered through implementation of claude_version.
- **Responsibility**: Index of pitfall doc instances covering failure modes with concrete trap, failure, and mitigation sections.
- **In Scope**: Version lock side effects, auto-updater bypass vectors.
- **Out of Scope**: Design pattern rationale (-> `pattern/`), feature behavior (-> `feature/`).

### Overview Table

| ID | Name | Status |
|----|------|--------|
| 001 | [Version Lock chmod Side Effects](001_version_lock_chmod.md) | ✅ |
| 002 | [Auto-Updater Symlink Retarget](002_symlink_retarget.md) | ✅ |
