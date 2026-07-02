# claude_storage Documentation

### Scope

- **Purpose:** Documentation hub for the `claude_storage` CLI crate.
- **Responsibility:** Behavioral requirements, CLI reference, and operational procedures.
- **In Scope:** CLI command and parameter reference (`cli/`), feature requirements (`feature/`), operational procedures (`operation/`), architecture guides (`guide/`), system invariants (`invariant/`), and doc cross-reference graph.
- **Out of Scope:** Source code (→ `src/`), automated tests (→ `tests/`), build scripts (→ `verb/`).

### Responsibility Table

| File | Responsibility |
|------|----------------|
| `cli/` | CLI command, parameter, and type specifications |
| `feature/` | CLI tool functional design and scope |
| `operation/` | Operational procedures for users and maintainers |
| `guide/` | Architecture internals reference guides |
| `invariant/` | System behavioral contracts |
| `entity.md` | Master doc entity and instance tables |
| `doc_graph.yml` | Cross-reference graph for navigability analysis |

### Overview

This directory contains comprehensive documentation for the `claude_storage` crate.

### Documents

### Claude Code Knowledge (shared)

Core specifications for Claude Code's storage layout, file formats, and JSONL schema have moved
to the repo-level [`contract/claude_code/docs/`](../../../contract/claude_code/docs/) directory, since they are
consumed by multiple crates (`claude_storage`, `claude_version`, `claude_runner`, `claude_profile`).

Key entity directories there: `behavior/` (25 hypotheses), `storage/`, `filesystem/`, `jsonl/`, `settings/`, `formats/`, `taxonomy/`, `params/` (73 params), `endpoint/`.

---

### Implementation Guides

#### [CLI Documentation](cli/)

**Complete CLI reference** for all claude_storage commands, parameters, and types.

**Contents**:
- [command/readme.md](cli/command/readme.md) — All 11 commands with syntax, parameters, examples
- [param/readme.md](cli/param/readme.md) — All parameters with types, validation, bidirectional cross-refs
- [type/readme.md](cli/type/readme.md) — Semantic type system with validation rules
- [dictionary.md](cli/dictionary.md) — Domain vocabulary (project, conversation, session, entry, scope, etc.)
- [param_group/readme.md](cli/param_group/readme.md) — Shared parameter groups (Output Control, Project Scope, Session Filter, etc.)

**Use this when**:
- Using or implementing CLI commands
- Looking up parameter formats (`param::value`)
- Understanding type constraints and valid values
- Designing new commands

---

### Architecture Guides

#### [Advanced Storage Topics](guide/001_advanced_storage_topics.md)

**Deep dive into advanced Claude Code storage features**.

**Contents**:
- **Agent sessions** - Sub-agent conversations (`agent-*.jsonl` format)
  - Flat and hierarchical storage layouts (B7, B13)
  - agentId field, isSidechain flag, and slug field
  - Agent metadata sidecars (`.meta.json` with agentType)
  - Parent session tracking and Session Family concept
  - Detection and discovery algorithms (both formats)
- **Command system** - Slash command definitions (46 commands)
  - YAML frontmatter + markdown structure
  - Command categories (audit, test, dev, etc.)
  - Role/Objective/Scope/Procedures pattern
- **History tracking** - Global project index details
  - Display field patterns (truncated messages, pasted content indicators)
  - pastedContents field usage
  - Timestamp precision (milliseconds)
- **Session environment** - session-env/ directory purpose
  - 549 empty directories (session markers)
  - Future use considerations
- **Advanced search** - Cross-project search, agent tracking, history-based discovery

**Use this when**:
- Implementing agent session support
- Understanding command system integration
- Building search features
- Exploring session metadata

---

### Quick Reference

### Key Files to Read First

1. **`../../../contract/claude_code/docs/behavior/`** - Claude Code storage architecture, file formats, JSONL schema
2. **`guide/001_advanced_storage_topics.md`** - Understand agent sessions, commands, history, search
3. **`cli/command/readme.md`** - Understand CLI commands and parameters
4. **`feature/001_cli_tool.md`** - Understand overall crate architecture and scope
5. **`../readme.md`** - User-facing documentation

