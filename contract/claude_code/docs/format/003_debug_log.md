# Format: Debug Log

### Scope

- **Purpose**: Specify the `~/.claude/debug/*.txt` format — plain text debug output from Claude Code operations.
- **Responsibility**: Authoritative instance for debug log format — line structure, content types, growth characteristics, maintenance guidance.
- **In Scope**: File location pattern, `[DEBUG]` line format, content types, growth, maintenance.
- **Out of Scope**: Debug log storage directory context (→ [`../storage/002_support_directories.md`](../storage/002_support_directories.md)).

### Location

`~/.claude/debug/*.txt`

**Format**: Plain text, line-oriented.
**Mutability**: Continuous append during Claude Code operation.

### Schema

**Line format**: `[DEBUG] message`

```
[DEBUG] Watching for changes in setting files...
[DEBUG] Found 0 plugins (0 enabled, 0 disabled)
[DEBUG] Total LSP servers loaded: 0
[DEBUG] Creating shell snapshot at /home/alice/.claude/shell-snapshots/...
[DEBUG] Starting session 8d795a1c-c81d-4010-8d29-b4e678272419
```

### Content Types

- Setting file watching
- Plugin loading and registration
- LSP server initialization
- Shell snapshot creation
- Process lifecycle events (start, stop)

### Growth

- Continuous append during Claude Code operation
- Can grow to 100MB+ per file over extended use
- Multiple files may exist (one per debug session or rotation boundary)

### Maintenance

Safe to delete entirely — no impact on conversations, settings, or credentials. Can be deleted to reclaim disk space at any time. Claude Code recreates new log files on next run.

### Since

pre-v1.0 (unverified)

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| entity | [readme.md](readme.md) | Formats master index |
| storage | [`../storage/002_support_directories.md`](../storage/002_support_directories.md) | `debug/` directory: size, maintenance guidance |
