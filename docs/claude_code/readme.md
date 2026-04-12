# Claude Code Doc Entity

### Scope

- **Purpose**: Document the external behavior, storage format, and runtime filesystem conventions of the Claude Code binary for all workspace crates that interact with it.
- **Responsibility**: Master file for the `claude_code/` doc entity — lists all instances, defines type-specific requirements, and declares scope boundaries.
- **In Scope**: Session behavior catalog, storage organization, filesystem layout, JSONL entry format, settings format, ancillary file formats, and complete runtime parameter reference (CLI flags, env vars, config keys).
- **Out of Scope**: Internal implementation details of workspace crates (→ respective crate `docs/`); Claude API protocol (→ Anthropic documentation); builder-API defaults and Rust `with_*()` methods (→ `module/claude_runner_core/docs/claude_params/`).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [Session Behaviors](001_session_behaviors.md) | Observed external behaviors (B1–B16h) with evidence and invalidation tests | ✅ |
| 002 | [Storage Organization](002_storage_organization.md) | `~/.claude/` directory architecture, containment hierarchy, access patterns | ✅ |
| 003 | [Filesystem Layout](003_filesystem_layout.md) | Runtime paths accessed by claude_manager; directory tree and path resolution | ✅ |
| 004 | [JSONL Format](004_jsonl_format.md) | Session entry schema: fields, content blocks, usage, threading model | ✅ |
| 005 | [Settings Format](005_settings_format.md) | settings.json structure, atomic write protocol, version lock, type inference | ✅ |
| 006 | [Ancillary Formats](006_ancillary_formats.md) | history.jsonl, credentials, debug logs, shell snapshots, todos, commands | ✅ |
| params/ | [Parameters](params/readme.md) | All 65 runtime parameters — CLI flags, env vars, config keys; one file per parameter | ✅ |

### Type-Specific Requirements

All `claude_code/` doc instances must include:

1. **Title**: `# Claude Code: {Name}` — using `Claude Code` as the type prefix
2. **Scope** (H3): 4 required bullets — Purpose, Responsibility, In Scope, Out of Scope
3. **Content sections** (H3/H4 only): At least one section documenting the specific facet
4. **Cross-References** (H3): Flat table with `Type | File | Responsibility` columns

Each doc instance covers exactly one facet of the external Claude Code binary's behavior or storage contract.

### Cross-Doc Entity Dependencies

**This entity depends on**:
- `error/` — Claude Code API error catalog (error/001–005)

**This entity consumed by**:
- `module/claude_storage/docs/` — storage implementation docs reference this entity extensively
- `module/claude_manager/docs/` — manager docs reference filesystem and settings format
- `module/claude_runner_core/docs/` — runner core docs reference behavior and params
