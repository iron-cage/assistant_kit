# Format: History JSONL

### Scope

- **Purpose**: Specify the `~/.claude/history.jsonl` format — global project access tracking across all sessions.
- **Responsibility**: Authoritative instance for the history.jsonl format — entry structure, field definitions, growth characteristics.
- **In Scope**: File location, entry structure, `display`/`pastedContents`/`timestamp`/`project` fields, growth pattern.
- **Out of Scope**: Session JSONL format for conversations (→ [`../jsonl/`](../jsonl/readme.md)); storage directory context (→ [`../storage/003_root_files.md`](../storage/003_root_files.md)).

### Location

`~/.claude/history.jsonl`

**Format**: Line-delimited JSON — one JSON object per line.
**Mutability**: Append-only.

### Schema

```json
{
  "display": "https://www.youtube-transcript.io/api\nread page...",
  "pastedContents": {},
  "timestamp": 1758992388766,
  "project": "/home/alice/projects/consumer-app/module/reasoner"
}
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `display` | string | ✅ | User query or context preview (first lines of initial prompt) |
| `pastedContents` | object | ✅ | Pasted file contents keyed by filename (usually empty `{}`) |
| `timestamp` | number | ✅ | Unix timestamp in milliseconds |
| `project` | string | ✅ | Filesystem path to the project directory |

### Growth

- Appends one entry per conversation start
- ~4,324 entries observed; ~254 bytes/entry average; ~1.1MB total
- Growth rate: proportional to number of distinct Claude Code sessions started

### Maintenance

Can be truncated to reclaim space — loses project history used for session picker but does not affect conversation data in `projects/`. File is safe to delete entirely (Claude Code recreates it); only loses the project history index.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| entity | [readme.md](readme.md) | Formats master index |
| storage | [`../storage/003_root_files.md`](../storage/003_root_files.md) | Root file context: size, access frequency, security |
