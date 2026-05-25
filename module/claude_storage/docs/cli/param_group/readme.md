# Parameter Groups

Shared parameters reused across command roots. Groups emerge from semantic coherence — parameters that together control the same operational concern.

See [param/readme.md](../param/readme.md) for individual parameter specs and [command/readme.md](../command/readme.md) for per-command usage.

## Responsibility Table

| File | Responsibility |
|------|----------------|
| `01_output_control.md` | Output Control — verbosity and detail level |
| `02_project_scope.md` | Project Scope — project:: identifier scoping |
| `03_session_identification.md` | Session Identification — session_id:: direct access |
| `04_session_filter.md` | Session Filter — session/agent/min_entries listing filters |
| `05_scope_configuration.md` | Scope Configuration — scope:: and path:: discovery boundary |

## Overview

| # | Group | Parameters | Used By |
|---|-------|-----------|---------|
| 1 | [Output Control](01_output_control.md) | `verbosity::` | 5 commands |
| 2 | [Project Scope](02_project_scope.md) | `project::` | 5 commands |
| 3 | [Session Identification](03_session_identification.md) | `session_id::` | 2 commands |
| 4 | [Session Filter](04_session_filter.md) | `session::`, `agent::`, `min_entries::` | 2 commands |
| 5 | [Scope Configuration](05_scope_configuration.md) | `scope::`, `path::` | 6 commands |
