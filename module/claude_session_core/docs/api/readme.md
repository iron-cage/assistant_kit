# API Doc Entity

### Scope

- **Purpose**: Document the programmatic interface of the `claude_session_core` library surface.
- **Responsibility**: Index of API doc instances covering the registry, liveness, and turn-detection exports.
- **In Scope**: `scan`, `scan_live`, `SessionRecord`, `SessionStatus`, `pid_alive`, `proc_starttime`, `TurnWatcher`, `TurnEvent`, `BackgroundReporting`, `Error`, `Result`.
- **Out of Scope**: CLI behavior (this crate has no binary); the `claude_storage_core` transcript surface it depends on.

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [Session Surface](001_session_surface.md) | Signature contract for every exported item | ✅ |
| — | [procedure.md](procedure.md) | Workflow for creating and updating API doc instances | ✅ |
