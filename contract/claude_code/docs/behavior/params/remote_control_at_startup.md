# remoteControlAtStartup

Enable remote-control connection on startup.

- **Config Key**: `remoteControlAtStartup`
- **Type**: bool
- **Default**: `false`
- **Scope**: G

When `true`, Claude Code opens a remote-control channel at startup, allowing
external processes (e.g. IDE extensions, orchestration tools) to send commands
to the running instance.

**Example** (`~/.claude/settings.json`):

```json
{ "remoteControlAtStartup": true }
```

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [`readme.md`](readme.md) | Master parameter table |
| doc | [`../005_settings_format.md`](../005_settings_format.md) | Settings JSON structure |
