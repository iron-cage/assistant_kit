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

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [`readme.md`](readme.md) | Master parameter table |
| doc | [`../005_settings_format.md`](../005_settings_format.md) | Settings JSON structure |
