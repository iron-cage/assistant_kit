# Feature: CLI Tool

### Scope

- **Purpose**: Provide an interactive and scriptable command-line interface for querying Claude Code's conversation storage at `~/.claude/`.
- **Responsibility**: Documents the CLI tool's design scope, backward-compatibility policy, and relationship to the core library.
- **In Scope**: Tool responsibility, in/out scope boundaries, backward-compatibility non-goal, CLI design principles.
- **Out of Scope**: Core storage library design (→ `claude_storage_core/docs/feature/001_core_library.md`), command parameters (→ `cli/`).

### Design

`claude_storage` (binary `cls`) is a CLI wrapper around the `claude_storage_core` library. It provides two invocation modes: an interactive REPL for exploratory use and one-shot command execution for scripting.

**Separation from core.** All storage access logic lives in `claude_storage_core`. This crate provides command parsing, REPL interface, output formatting, and routing to the core library. The separation means library consumers (e.g. automation tools) can depend on `claude_storage_core` without pulling in CLI framework dependencies.

**Backward compatibility is a non-goal.** Command names, parameter syntax, and output formats can change freely between versions. The tool is designed for developers who can adapt scripts to changes. There are no deprecated-alias commands, no output-format versioning, and no migration shims.

**Data model.** The tool exposes a four-level hierarchy: Storage Root → Project → Session → Entry. Projects are either UUID-based (web/IDE sessions) or path-encoded (CLI sessions). Two session storage layouts coexist: flat layout (older projects, B7) and hierarchical layout (newer projects with sub-agent directories, B13/B14/B15). The tool handles both transparently.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | `../../module/claude_storage_core/docs/feature/001_core_library.md` | Core library this CLI wraps |
| doc | `../cli/commands.md` | All CLI commands with syntax and examples |
| doc | `../advanced_topics.md` | Agent sessions and advanced storage topics |

### Sources

| File | Notes |
|------|-------|
| `spec.md` (deleted — migrated here) | Combined specification; responsibility, design principles, and scope sections extracted here; data model extracted to `claude_storage_core/docs/data_structure/001_storage_hierarchy.md` |
