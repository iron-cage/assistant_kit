# claude_storage Documentation

### Responsibility Table

| File | Responsibility |
|------|----------------|
| `cli/` | CLI command, parameter, and type specifications |
| `feature/` | CLI tool functional design and scope |
| `operation/` | Operational procedures for users and maintainers |
| `advanced_topics.md` | Agent sessions, command system, advanced search |
| `cli_design.md` | CLI design decisions (superseded by `cli/`) |
| `doc_graph.yml` | Cross-reference graph for navigability analysis |
| `entities.md` | Entity tree and cross-entity index of all doc instances |

### Overview

This directory contains comprehensive documentation for the `claude_storage` crate.

### Documents

### Claude Code Knowledge (shared)

Core specifications for Claude Code's storage layout, file formats, and JSONL schema have moved
to the workspace-level [`docs/claude_code/`](../../docs/claude_code/) directory, since they are
consumed by multiple crates (`claude_storage`, `claude_version`, `claude_runner`, `claude_profile`).

Key doc instances there: `001_session_behaviors.md`, `002_storage_organization.md`, `003_filesystem_layout.md`,
`004_jsonl_format.md`, `005_settings_format.md`, `006_ancillary_formats.md`.

---

### Implementation Guides

#### [CLI Documentation](cli/)

**Complete CLI reference** for all claude_storage commands, parameters, and types.

**Contents**:
- [commands.md](cli/commands.md) — All 13 commands with syntax, parameters, examples
- [params.md](cli/params.md) — All parameters with types, validation, bidirectional cross-refs
- [types.md](cli/types.md) — Semantic type system with validation rules
- [dictionary.md](cli/dictionary.md) — Domain vocabulary (project, session, entry, scope, etc.)
- [parameter_groups.md](cli/parameter_groups.md) — Shared parameter groups (Output Control, Project Scope, Session Filter, etc.)

**Use this when**:
- Using or implementing CLI commands
- Looking up parameter formats (`param::value`)
- Understanding type constraints and valid values
- Designing new commands

---

### Advanced Topics

#### [Advanced Topics](advanced_topics.md)

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

1. **`../../docs/claude_code/`** - Claude Code storage architecture, file formats, JSONL schema
2. **`advanced_topics.md`** - Understand agent sessions, commands, history, search
3. **`cli/commands.md`** - Understand CLI commands and parameters
5. **`feature/001_cli_tool.md`** - Understand overall crate architecture and scope
6. **`../readme.md`** - User-facing documentation

### Implementation Order

**Phase 2: Read-Only Parsing**

1. ✅ JSON parser foundation (`src/json.rs`)
2. ✅ Entry type parsing (`src/entry.rs`)
3. ✅ User message parsing
4. ✅ Assistant message parsing
5. ✅ Integration tests (`tests/`)
6. ✅ Error handling
7. ✅ Enhanced read API
8. ✅ Documentation & examples

---

### Current Status

**Phase 1: Foundation** ✅ Complete
- Core types implemented
- Path encoding working
- 18 unit tests passing
- Zero dependencies maintained

**Phase 2: Read-Only Parsing** ✅ Complete
- JSON parser implemented
- Entry type parsing (user, assistant)
- Integration tests passing
- Enhanced read API with ProjectStats

**Phase 3: Integration** 🔜 Future
- Optional integration with `claude_profile`
- Used by `wplan_agent`
- Example tools

---

### Testing Data

### Real Storage Location

```bash
~/.claude/projects/
```

### Sample Project (Current Directory)

```bash
~/.claude/projects/-home-user1-pro/
```

### Examining Real Data

```bash
# List projects
ls ~/.claude/projects/

# List sessions in current project
ls ~/.claude/projects/-home-user1-pro/

# View first entry
head -1 ~/.claude/projects/-home-user1-pro/SESSION_ID.jsonl | jq .

# View user message
head -1 FILE.jsonl | jq '.message'

# View assistant message
head -2 FILE.jsonl | tail -1 | jq '.message.content[]'
```

---

### Format Examples

### Minimal User Message

```json
{
  "uuid": "a6f3bd8c-5575-4eab-82b0-b856f7a02833",
  "parentUuid": null,
  "timestamp": "2025-11-08T23:30:10.039Z",
  "type": "user",
  "cwd": "/home/user",
  "sessionId": "8d795a1c-c81d-4010-8d29-b4e678272419",
  "version": "2.0.31",
  "gitBranch": null,
  "userType": "external",
  "isSidechain": false,
  "message": {
    "role": "user",
    "content": "Hello"
  },
  "thinkingMetadata": {
    "level": "low",
    "disabled": true,
    "triggers": []
  }
}
```

### Minimal Assistant Message

```json
{
  "uuid": "56a226b5-0ec6-4214-af16-b13cc326f8dc",
  "parentUuid": "a6f3bd8c-5575-4eab-82b0-b856f7a02833",
  "timestamp": "2025-11-08T23:30:21.913Z",
  "type": "assistant",
  "cwd": "/home/user",
  "sessionId": "8d795a1c-c81d-4010-8d29-b4e678272419",
  "version": "2.0.31",
  "gitBranch": null,
  "userType": "external",
  "isSidechain": false,
  "message": {
    "model": "claude-sonnet-4-5-20250929",
    "id": "msg_01ABC",
    "type": "message",
    "role": "assistant",
    "content": [
      {
        "type": "text",
        "text": "Hi there!"
      }
    ],
    "stop_reason": "end_turn",
    "stop_sequence": null,
    "usage": {
      "input_tokens": 5,
      "output_tokens": 3,
      "cache_creation_input_tokens": 0,
      "cache_read_input_tokens": 0,
      "service_tier": "standard"
    }
  },
  "requestId": "req_01ABC"
}
```

---

### Implementation Notes

### Zero Dependencies Strategy

**Rationale**: Hand-written JSON parser instead of `serde_json`

**Pros**:
- Fast compilation
- Minimal attack surface
- No version conflicts
- Full control over error messages

**Cons**:
- More implementation work (+8-10 hours)

**Decision**: Worth the tradeoff for long-term maintainability

### Error Handling Approach

**Graceful degradation**:
- Parse as many entries as possible
- Skip malformed entries with warnings
- Return both successes and errors
- Never panic on user data

**Error messages include**:
- What failed
- Why it failed
- Where it failed (line number)
- Context (snippet of problematic content)

### Performance Targets

- Parse 1000+ entries/second
- <100ms for typical session (100 entries)
- <1 second for large session (10,000 entries)
- Memory usage scales linearly
- Streaming available for huge sessions

---

### API Examples

### List Projects

```rust
use claude_storage::Storage;

let storage = Storage::new()?;
for project in storage.list_projects()? {
  println!("Project: {:?}", project.id());
}
```

### Read Session

```rust
use claude_storage::Storage;

let storage = Storage::new()?;
let project = storage.load_project_for_cwd()?;

for mut session in project.sessions()? {
  println!("Session: {}", session.id());

  for entry in session.entries()? {
    println!("  [{}] {}", entry.entry_type, entry.uuid);
  }
}
```

### Search History (Future)

```rust
use claude_storage::Storage;

let storage = Storage::new()?;
let results = storage.search("version_bump")?;

for result in results {
  println!("Found in {}: {}", result.session_id, result.snippet);
}
```

---

### Contributing

### Before Starting Implementation

1. ✅ Read [`docs/claude_code/004_jsonl_format.md`](../../docs/claude_code/004_jsonl_format.md) - Understand data format
2. ✅ Read `development_plan.md` - Understand approach
3. ✅ Examine real data - Run jq commands above
4. ✅ Review existing code - Understand current structure

### During Implementation

1. Follow task order from development plan
2. Write tests first (TDD)
3. Test against real Claude Code data
4. Keep zero dependencies
5. Document as you go

### Testing Checklist

- ✅ Unit tests for each module
- ✅ Integration tests with real storage
- ✅ Doc tests for all public APIs
- ✅ Error handling tests
- ✅ Performance benchmarks
- ✅ No panics on any Claude Code data

---

### Resources

- **JSONL Spec**: https://jsonlines.org/
- **Claude API**: https://docs.anthropic.com/en/api/
- **ISO 8601**: https://www.iso.org/iso-8601-date-and-time-format.html
- **UUID v4**: RFC 4122

---

### Questions?

See the main [readme.md](../readme.md) for more information.
