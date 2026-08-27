# API Doc Entity

### Scope

- **Purpose**: Pin the public library contract of `claude_version_core` so Layer 2 consumers can depend on it without reading the source.
- **Responsibility**: Index of API doc instances, one per surface group.
- **In Scope**: Public types, constants, and functions across the five modules; error contracts; platform variance; purity and side-effect notes.
- **Out of Scope**: Structural constraints (→ `invariant/`), CLI behavior and algorithms (→ `../../../claude_version/docs/`).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [Core Surface](001_core_surface.md) | `CoreError`, `paths`, `config_catalog`, `config_resolve`, `params_catalog` | ✅ |
| 002 | [Version Surface](002_version_surface.md) | The `version` module: detection, markers, install, lock state, preferences | ✅ |
| — | [procedure.md](procedure.md) | Workflow for creating and updating API doc instances | ✅ |
