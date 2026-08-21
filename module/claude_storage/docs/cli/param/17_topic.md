# Parameter :: 17. `topic::`

### Scope

- **Purpose**: Specify the `topic::` CLI parameter.
- **Responsibility**: Type, defaults, valid values, and command usage for `topic::`.
- **In Scope**: Value constraints, default behavior, command interactions.
- **Out of Scope**: Type definitions (→ `type/`), command behavior (→ `command/`).

Session topic name appended as a `-{name}` suffix to the base directory path.

**Type:** [`TopicName`](../type/12_topic_name.md)

**Fundamental Type:** String (identifier)

**Constraints:**
- Must be non-empty when provided
- Must not contain `/`
- Do NOT include a leading `-` in the value — it is added automatically
- Error on empty: `"topic must be non-empty"`
- Error on slash: `"topic must not contain path separators"`

**Default:** unset (no suffix applied) for `.project.path`, `.project.exists`; `default_topic` for `.session.dir`, `.session.ensure`; unset (falls back to the most recently modified non-agent session — BUG-488) for `.tail`; unset (selector — `latest` behavior applies instead) for `.session.path`

**Commands:** `.project.path`, `.project.exists`, `.session.dir`, `.session.ensure`, `.tail`, `.session.path`

**Purpose:** Identifies a named session topic within a base directory. Claude Code uses hyphen-prefixed directories (`-default_topic`, `-work`, `-commit`) as session working directories. `topic::` takes the name without the leading hyphen and appends it as `{base}/-{topic}`.

**Sense collision (deliberate):** the paragraph above is the legacy dir-suffix sense, shared by every command EXCEPT `.session.path`. In [`.session.path`](../command/16_session_path.md), `topic::` instead names a fork-mode topic — the value selects the deterministic session file `{storage}/{UUIDv5( canonical base, name )}.jsonl` inside the BASE directory's own storage, never a `-{topic}` sibling directory. Same name, same value constraints, disjoint resolution rule; see `16_session_path.md § Topic Sense Collision` for the rationale.

**Examples:**
```bash
# Valid values
topic::default_topic    # → appended as /-default_topic
topic::work             # → appended as /-work
topic::commit           # → appended as /-commit

# Invalid values
topic::                 # "topic must be non-empty"
topic::my/topic         # "topic must not contain path separators"
topic::-default_topic   # (legal — creates /-default_topic... but convention is without leading -)
```

### Referenced Type
| Type | Kind | Fundamental | Key Constraint |
|------|------|-------------|----------------|
| [`TopicName`](../type/12_topic_name.md) | String (identifier) | String | Non-empty; no `/` characters; no leading `-` |

### Referenced Commands
| # | Command | Default | Notes |
|---|---------|---------|-------|
| 8 | [`.project.path`](../command/08_project_path.md) | unset | Appends `-{topic}` suffix to computed storage path |
| 9 | [`.project.exists`](../command/09_project_exists.md) | unset | Checks storage path with topic suffix |
| 10 | [`.session.dir`](../command/10_session_dir.md) | `default_topic` | Appends `-{topic}` to base directory |
| 11 | [`.session.ensure`](../command/11_session_ensure.md) | `default_topic` | Appends `-{topic}` to base directory |
| 12 | [`.tail`](../command/12_tail.md) | unset | Session topic suffix for the resolved session; omitted falls back to the most recently modified non-agent session |
| 16 | [`.session.path`](../command/16_session_path.md) | unset | **Fork-mode sense** — UUIDv5 session file in the base's storage, not a `-{topic}` dir suffix |

### Referenced User Stories
| # | User Story | Persona |
|---|------------|---------|
| 2 | [Find Past Conversation](../user_story/002_find_past_conversation.md) | developer |
| 5 | [Resume Claude Session](../user_story/005_resume_claude_session.md) | developer |
| 6 | [Quick Context Refresh](../user_story/006_quick_context_refresh.md) | developer |
