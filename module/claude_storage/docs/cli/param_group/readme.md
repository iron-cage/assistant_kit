# Parameter Groups

### Scope

- **Purpose**: Document shared parameter sets reused across command roots.
- **Responsibility**: Per-group detail pages with membership, examples, and cross-refs.
- **In Scope**: All 5 parameter groups with member parameters and command usage.
- **Out of Scope**: Individual parameter specs (→ `param/`), type constraints (→ `type/`).

Shared parameters reused across command roots. Groups emerge from semantic coherence — parameters that together control the same operational concern.

See [param/readme.md](../param/readme.md) for individual parameter specs and [command/readme.md](../command/readme.md) for per-command usage.

### Responsibility Table

| File | Responsibility |
|------|----------------|
| `01_output_control.md` | Output Control — show_stat, show_tokens, show_tree toggles |
| `02_project_scope.md` | Project Scope — project:: identifier scoping |
| `03_session_identification.md` | Session Identification — session_id:: direct access |
| `04_session_filter.md` | Session Filter — session/agent/min_entries listing filters |
| `05_scope_configuration.md` | Scope Configuration — scope:: and path:: discovery boundary |

### Overview

| # | Group | Parameters | Used By |
|---|-------|-----------|---------|
| 1 | [Output Control](01_output_control.md) | `show_stat::`, `show_tokens::`, `show_tree::` | 3 commands |
| 2 | [Project Scope](02_project_scope.md) | `project::` | 5 commands |
| 3 | [Session Identification](03_session_identification.md) | `session_id::` | 2 commands |
| 4 | [Session Filter](04_session_filter.md) | `session::`, `agent::`, `min_entries::` | 2 commands |
| 5 | [Scope Configuration](05_scope_configuration.md) | `scope::`, `path::` | 6 commands |
