# Pattern Doc Entity

### Scope

- **Purpose**: Document reusable SDK-integration patterns relevant to this workspace, mirroring `contract/claude_code`'s `pattern/` collection style (design-pattern documentation, not per-symbol reference).
- **Responsibility**: Master file for the `pattern` collection — lists both pattern instances.
- **In Scope**: The in-process custom-tool pattern; the architectural options for bridging a Rust caller to the SDK's protocol.
- **Out of Scope**: Per-symbol API reference (→ [`../api/`](../api/readme.md)); per-field parameter reference (→ [`../param/`](../param/readme.md)).

### Responsibility Table

| File | Responsibility |
|------|-----------------|
| readme.md | Master pattern table (this file) |
| 001_in_process_custom_tool.md | Combining `tool()` + `createSdkMcpServer()` into a registered in-process tool |
| 002_rust_bridge_strategies.md | Architectural options for a Rust caller to drive SDK-mode, given no official Rust binding exists |
