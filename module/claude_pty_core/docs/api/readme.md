# API Doc Entity

### Scope

- **Purpose**: Document the programmatic interface of the `claude_pty_core` library surface.
- **Responsibility**: Index of API doc instances covering the exported types, functions, and error variants.
- **In Scope**: `Pty`, `WinSize`, `SessionConfig`, `PtySession`, `WriterHandle`, `Error`, `Result`, and the `env_scrub` module.
- **Out of Scope**: The `ffi` module, which is private (→ [invariant/001_unsafe_containment.md](../invariant/001_unsafe_containment.md)); CLI behavior (this crate has no binary).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [PTY Surface](001_pty_surface.md) | Signature contract for every exported item | ✅ |
| — | [procedure.md](procedure.md) | Workflow for creating and updating API doc instances | ✅ |
