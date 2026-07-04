# Feature: CLI Tool

### Scope

- **Purpose**: Provide an interactive and scriptable command-line interface for querying Claude Code's conversation storage at `~/.claude/`.
- **Responsibility**: Documents the CLI tool's design scope, backward-compatibility policy, and relationship to the core library.
- **In Scope**: Tool responsibility, in/out scope boundaries, backward-compatibility non-goal, CLI design principles.
- **Out of Scope**: Core storage library design (→ `claude_storage_core/docs/feature/001_core_library.md`), command parameters (→ `cli/`).

### Design

`claude_storage` (binary `clg`) is a CLI wrapper around the `claude_storage_core` library. It provides three invocation modes: help output (empty argv or `.help`), an interactive REPL (`--repl`), and one-shot command execution for scripting.

**Help rendering:** When invoked with no arguments, `.help`, `--help`, or `-h`, the binary renders grouped command output via `cli_fmt::CliHelpTemplate` to stdout and exits 0. Commands are displayed in groups matching their functional domain (Status, Session, Project, Query). Help is rendered before any pipeline initialization.

**Separation from core.** All storage access logic lives in `claude_storage_core`. This crate provides command parsing, REPL interface, output formatting, and routing to the core library. The separation means library consumers (e.g. automation tools) can depend on `claude_storage_core` without pulling in CLI framework dependencies.

**Backward compatibility is a non-goal.** Command names, parameter syntax, and output formats can change freely between versions. The tool is designed for developers who can adapt scripts to changes. There are no deprecated-alias commands, no output-format versioning, and no migration shims.

**Data model.** The tool exposes a four-level hierarchy: Storage Root → Project → Session → Entry. Projects are either UUID-based (web/IDE sessions) or path-encoded (CLI sessions). Two session storage layouts coexist: flat layout (older projects, B7) and hierarchical layout (newer projects with sub-agent directories, B13/B14/B15). The tool handles both transparently.

### Algorithms

| File | Relationship |
|------|-------------|
| `../algorithm/001_agent_session_tracking.md` | Agent session discovery algorithm used by this CLI |

### Features

| File | Relationship |
|------|-------------|
| `../../../claude_storage_core/docs/feature/001_core_library.md` | Core library this CLI wraps |

### Operations

| File | Relationship |
|------|-------------|
| `../operation/001_migration_guide.md` | Migration procedure for storage layout upgrades |

### Provenance

| File | Notes |
|------|-------|
| `spec.md` (deleted) | Combined specification; responsibility, design principles, and scope sections extracted here; data model extracted to `claude_storage_core/docs/data_structure/001_storage_hierarchy.md` |

### Sources

| File | Relationship |
|------|-------------|
| `../../src/lib.rs` | Crate root; re-exports and top-level documentation |
| `../../src/cli/` | CLI command implementations |

### Tests

| File | Relationship |
|------|-------------|
| `../../tests/cli_commands.rs` | Integration tests for all commands |
| `../../tests/cli_sanity.rs` | Sanity checks for CLI output |
