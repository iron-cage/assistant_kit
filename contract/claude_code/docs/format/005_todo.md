# Format: Todo

### Scope

- **Purpose**: Specify the `~/.claude/todos/{session-uuid}.json` format — JSON arrays of task objects for conversation sessions.
- **Responsibility**: Authoritative instance for todo JSON format — array structure, task object fields, status values, file organization.
- **In Scope**: File location, JSON array structure, `content`/`status`/`activeForm` fields, status values.
- **Out of Scope**: Todo directory context (→ [`../storage/002_support_directories.md`](../storage/002_support_directories.md)).

### Location

`~/.claude/todos/{session-uuid}.json`

**Format**: JSON array of task objects.
**Mutability**: Overwritten on task status changes.

### Schema

```json
[
  {
    "content": "Plan comprehensive manual testing strategy",
    "status": "completed",
    "activeForm": "Planning comprehensive manual testing strategy"
  },
  {
    "content": "Implement manual test runner",
    "status": "in_progress",
    "activeForm": "Implementing manual test runner"
  },
  {
    "content": "Write test documentation",
    "status": "pending",
    "activeForm": "Writing test documentation"
  }
]
```

### Task Object Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `content` | string | ✅ | Task description in imperative form (e.g., "Run tests") |
| `status` | string | ✅ | Task state: `"pending"`, `"in_progress"`, `"completed"` |
| `activeForm` | string | ✅ | Task description in present continuous form (e.g., "Running tests") |

### File Organization

One file per session UUID. Updated on every task status change (full array overwrite). File is created when the first todo is written for a session; absent if no todos were created.

### Since

pre-v1.0 (unverified)

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| entity | [readme.md](readme.md) | Formats master index |
| storage | [`../storage/002_support_directories.md`](../storage/002_support_directories.md) | `todos/` directory: size, organization |
| tool | [`../tool/027_todo_write.md`](../tool/027_todo_write.md) | TodoWrite tool that produces this format |
