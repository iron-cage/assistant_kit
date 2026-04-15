# Feature Doc Entity

### Scope

- **Purpose**: Document user-facing capabilities of assistant as the Layer 3 super-app that surfaces all Layer 2 commands under one `clt` binary.
- **Responsibility**: Index of feature doc instances covering programmatic command aggregation and the unified CLI entry point.
- **In Scope**: `register_commands()` aggregation pattern, `build_registry()`, static YAML command inclusion, feature-gated compilation.
- **Out of Scope**: Individual Layer 2 command behavior (→ each Layer 2 crate's `docs/`), unilang pipeline mechanics (→ unilang docs).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [Super-App Aggregation](001_super_app_aggregation.md) | Programmatic command registration from five Layer 2 crates into clt | ✅ |
