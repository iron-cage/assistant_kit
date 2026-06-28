# Type :: 12. `TopicName`

### Scope

- **Purpose**: Specify the `TopicName` semantic type.
- **Responsibility**: Validation rules, fundamental type, and parameter mapping for `TopicName`.
- **In Scope**: Parsing rules, valid/invalid values, error messages.
- **Out of Scope**: Parameter usage (→ `param/`), command context (→ `command/`).

**Purpose:** Session topic identifier appended as `-{name}` to a base directory path. Represents the human-readable name for a Claude Code session working directory.

**Fundamental Type:** Wrapper around string

**Constants:**
- DEFAULT_TOPIC = `"default_topic"` (used by `.session.dir` and `.session.ensure` when `topic::` is absent)

**Constraints:**
- Non-empty string when provided
- Must not contain `/` — error: `"topic must not contain path separators"`
- Supplied without leading `-`; the handler prepends `-` when constructing the directory path
- Error on empty: `"topic must be non-empty"`

**Parsing:**
```
Validate non-empty, no slashes:
  Input: "default_topic" → TopicName("default_topic") → dir suffix "-default_topic"
  Input: "work"          → TopicName("work")          → dir suffix "-work"
  Input: ""              → Error("topic must be non-empty")
  Input: "my/topic"      → Error("topic must not contain path separators")
```

**Methods:**
- `get() -> string` — Raw topic name (without leading `-`)
- `as_dir_suffix() -> string` — Returns `-{name}` for appending to base path

**Commands:** `.project.path`, `.project.exists`, `.session.dir`, `.session.ensure`

### Referenced Commands

| # | Command | Via Parameter |
|---|---------|---------------|
| 8 | [`.project.path`](../command/08_project_path.md) | `topic::` |
| 9 | [`.project.exists`](../command/09_project_exists.md) | `topic::` |
| 10 | [`.session.dir`](../command/10_session_dir.md) | `topic::` |
| 11 | [`.session.ensure`](../command/11_session_ensure.md) | `topic::` |

### Referenced Parameters

| # | Parameter | Commands |
|---|-----------|----------|
| 17 | [`topic::`](../param/17_topic.md) | 4 |
