# API Doc Entity

### Scope

- **Purpose**: Document the programmatic interface of the `claude_terminal_core` library surface.
- **Responsibility**: Index of API doc instances covering the exported function and constant.
- **In Scope**: `to_plain_text`, `MAX_ESCAPE_PARAM_CHARS`, and the `render` module they live in.
- **Out of Scope**: The private scanner internals (`put`, `apply_csi`, `tidy`); behavioral rationale (→ [feature/001_readable_output.md](../feature/001_readable_output.md)); CLI behavior (this crate has no binary).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [Terminal Surface](001_terminal_surface.md) | Signature contract for every exported item | ✅ |
| — | [procedure.md](procedure.md) | Workflow for creating and updating API doc instances | ✅ |
