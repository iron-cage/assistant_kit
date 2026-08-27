# API Doc Entity

### Scope

- **Purpose**: Document the programmatic interface of the `claude_topic_core` library surface.
- **Responsibility**: Index of API doc instances covering every exported type, function, and constant.
- **In Scope**: Items re-exported from `lib.rs`, plus the six module paths they are also reachable through.
- **Out of Scope**: Private helpers (`mix`, `claim`, `reclaim`, `parse_owner`, `collect_dir_topics`, `collect_fork_topics`, `registry_root`, `registry_file`); behavioral rationale (→ `../feature/`); CLI behavior (this crate has no binary).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [Topic Surface](001_topic_surface.md) | Signature contract for every exported item | ✅ |
| — | [procedure.md](procedure.md) | Workflow for creating and updating API doc instances | ✅ |
