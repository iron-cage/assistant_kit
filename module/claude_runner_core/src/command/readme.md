# src/command/

ClaudeCommand builder split into focused per-tier parameter modules.

## Responsibility Table

| File | Responsibility |
|------|----------------|
| `mod.rs` | ClaudeCommand struct, execution methods, describe helpers |
| `params_core.rs` | Tier 1 critical parameters with non-standard defaults |
| `params_security.rs` | Tier 2 security-sensitive parameters and tool permission lists |
| `params_extended.rs` | Tier 3+ optional I/O, session, MCP, model, debug, and IDE parameters |
