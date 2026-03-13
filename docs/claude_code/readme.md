# Claude Code Knowledge Base

Shared knowledge about the external Claude Code binary — its storage layout, file formats,
runtime behavior, and filesystem conventions. This knowledge is consumed by multiple crates
in the workspace (`claude_storage`, `claude_manager`, `claude_runner`, `claude_profile`).

## Responsibility Table

| File | Responsibility |
|------|----------------|
| `behavior.md` | Observed Claude Code session behaviors and invalidation tests (B1-B15) |
| `storage_organization.md` | Storage directory architecture under `~/.claude/` |
| `jsonl_format.md` | JSONL conversation entry format specification |
| `file_formats.md` | All Claude Code file format specifications |
| `filesystem.md` | Runtime filesystem paths and directory layout |
| `settings_format.md` | Settings file structure and atomic write protocols |

## Related (in-crate)

- **CLI parameters** — `module/claude_runner_core/docs/claude_params/` (59 parameter docs; dual-purpose with builder API mapping)
