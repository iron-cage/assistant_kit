# Algorithm: Agent Session Tracking

### Scope

- **Purpose**: Discover and enumerate agent sessions spawned from a root session, across both flat and hierarchical storage layouts.
- **Responsibility**: Agent session naming/entry markers and the discovery procedure for both storage formats.
- **In Scope**: Filename/entry detection, flat and hierarchical discovery procedures, agent metadata sidecars.
- **Out of Scope**: Session Family membership contract (→ `../invariant/02_session_family.md`), CLI display of session families (→ `../cli/command/07_projects.md`).

### Abstract

**Agent sessions** are sub-conversations spawned by Claude Code using the Task tool. Two storage formats coexist per-project (neither deprecated):

- **Flat** (older projects, B7): `agent-{id}.jsonl` alongside the main session at project root.
- **Hierarchical** (newer projects, B13): `{session-uuid}/subagents/agent-{id}.jsonl`.

Both formats form a **Session Family** — a root session and all its agents. See [`../invariant/02_session_family.md`](../invariant/02_session_family.md) for the formal membership contract.

**File naming**: `agent-{id}.jsonl` where `{id}` is a variable-length identifier. Two patterns observed:
- **Pure hex** (68%): 7 or 17 hex characters (e.g., `aec970f`, `a6061d6e2a0c37a78`)
- **Typed prefix** (32%): `compact-hex`, `prompt_suggestion-hex`, `side_question-hex` (e.g., `acompact-629548848068aaa6`)

**Entry format**: agent JSONL uses the same entry structure as regular sessions, with key differences:

| Field | Value | Description |
|-------|-------|--------------|
| `isSidechain` | `true` | Marks entry as part of an agent session |
| `agentId` | e.g. `"64bdad98"` | Short identifier matching the filename suffix |
| `sessionId` | e.g. `"9425242b..."` | References the PARENT session (not this agent session) |

### Algorithm

**Detection**:
```rust
fn is_agent_session(filename: &str) -> bool {
  filename.starts_with("agent-") && filename.ends_with(".jsonl")
}

fn is_agent_entry(entry: &Entry) -> bool {
  entry.is_sidechain && entry.agent_id.is_some()
}
```

**Discovery (flat format)**: list the project directory; every `agent-*.jsonl` file is a candidate; parse its first entry and compare `sessionId` against the root session's UUID to confirm family membership.

```rust
pub fn find_agent_sessions( session : &Session ) -> Result< Vec< AgentSession > >
{
  let project_dir = session.project_dir();
  let main_session_id = session.id();
  let mut agents = Vec::new();

  for entry in fs::read_dir(project_dir)? {
    let entry = entry?;
    let filename = entry.file_name().to_string_lossy().to_string();

    if filename.starts_with("agent-") && filename.ends_with(".jsonl") {
      let agent_id = &filename[6..filename.len() - 6];
      let agent_session = AgentSession::load(project_dir, agent_id)?;

      if agent_session.parent_session_id() == main_session_id {
        agents.push(agent_session);
      }
    }
  }

  Ok(agents)
}
```

**Discovery (hierarchical format)**: family membership is established by directory structure, not by field matching — every `agent-*.jsonl` under `{uuid}/subagents/` belongs to that root session by construction (see [`../invariant/02_session_family.md`](../invariant/02_session_family.md) Violation Conditions — using `sessionId` here instead of directory structure is a contract violation).

```rust
fn find_hierarchical_agents( project_dir : &Path ) -> Result< Vec< PathBuf > >
{
  let mut agents = Vec::new();
  for entry in fs::read_dir( project_dir )?
  {
    let entry = entry?;
    if !entry.file_type()?.is_dir() { continue; }
    let subagents = entry.path().join( "subagents" );
    if subagents.is_dir()
    {
      for agent in fs::read_dir( &subagents )?
      {
        let agent = agent?;
        let name = agent.file_name().to_string_lossy().to_string();
        if name.starts_with( "agent-" ) && name.ends_with( ".jsonl" )
        {
          agents.push( agent.path() );
        }
      }
    }
  }
  Ok( agents )
}
```

**Agent metadata sidecars (meta.json)**: in hierarchical format (B13), each `agent-{id}.jsonl` may have a sibling `agent-{id}.meta.json` containing agent metadata (B14).

Schema:
```json
{"agentType": "Explore", "description": "Read organizational principles rulebook"}
```

| Field | Required | Description |
|-------|----------|-------------|
| `agentType` | yes | Agent type: `Explore`, `general-purpose`, `Plan`, or `claude-code-guide` |
| `description` | no | Human-readable task description (present on some Explore agents) |

**Edge case**: some `.meta.json` files are empty (0 bytes) — parsers must handle this gracefully (see [`../invariant/02_session_family.md`](../invariant/02_session_family.md)).

**Considerations**:
- **Session listing**: include agent sessions in project session lists (with an `is_agent` flag); must discover both flat and hierarchical agents.
- **Parent tracking**: store `parent_session_id` for agent sessions (from the `sessionId` field, B12).
- **Entry parsing**: handle `agentId`, `isSidechain`, and `slug` fields.
- **Metadata**: parse `.meta.json` sidecars for `agentType` and `description` (hierarchical format only, B14).
- **Display**: mark agent sessions distinctly in CLI output (e.g., "agent-64bdad98 (sub-agent)").

### Referenced Commands

| # | Command | Context |
|---|---------|---------|
| 7 | [`.projects`](../cli/command/07_projects.md) | CLI display of session families |

### Features

| File | Relationship |
|------|-------------|
| `../feature/001_cli_tool.md` | Overall crate scope and design |

### Invariants

| File | Relationship |
|------|-------------|
| `../invariant/02_session_family.md` | Session Family membership contract — threading, meta.json edge case, slug field |

### Sources

| File | Notes |
|------|-------|
| `../../../../contract/claude_code/docs/storage/readme.md` | directory structure and storage model |
| `../../../../contract/claude_code/docs/format/readme.md` | ancillary file format specifications |
| `../../../../contract/claude_code/docs/jsonl/readme.md` | conversation entry format details |

### Tests

| File | Relationship |
|------|-------------|
| `../../tests/invariant_contracts_test.rs` | AL-1, AL-2 — agent session format contract tests |
