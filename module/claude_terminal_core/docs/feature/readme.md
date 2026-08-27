# Feature Doc Entity

### Scope

- **Purpose**: Document user-facing capabilities of the `claude_terminal_core` library for consumers that hold raw terminal output and need to read it.
- **Responsibility**: Index of feature doc instances covering the escape-sequence scanner and the plain-text it produces.
- **In Scope**: `render::to_plain_text`, `render::MAX_ESCAPE_PARAM_CHARS`, the in-line cursor model, and trimming.
- **Out of Scope**: Producing the stream — terminal allocation and child spawning (→ `claude_pty_core/docs/feature/`), retaining and serving a hosted session's output (→ `claude_daemon_core/docs/feature/`), invariant constraints (→ `invariant/`).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [Readable Output](001_readable_output.md) | Render a terminal's byte stream as the text a reader would have seen | ✅ |
| — | [procedure.md](procedure.md) | Workflow for creating and updating feature doc instances | ✅ |
