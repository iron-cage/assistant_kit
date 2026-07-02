# fileCheckpointingEnabled

Enable automatic file checkpointing before edits.

- **Config Key**: `fileCheckpointingEnabled`
- **Type**: bool
- **Default**: `false`
- **Scope**: G

When `true`, Claude Code saves a checkpoint copy of each file before modifying it,
providing a recovery path if an edit produces an undesired result.

**Example** (`~/.claude/settings.json`):

```json
{ "fileCheckpointingEnabled": true }
```

### Since

pre-v1.0 (unverified)

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [`readme.md`](readme.md) | Master parameter table |
| doc | [`../settings/readme.md`](../settings/readme.md) | Settings JSON structure |
