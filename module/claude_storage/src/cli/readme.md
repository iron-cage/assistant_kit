# src/cli/

CLI command routines for `claude_storage`. Each file owns one command (or a
closely related cluster of commands), keeping individual files focused and
navigable.

## Responsibility Table

| File | Responsibility |
|------|----------------|
| `mod.rs` | Module declarations and public re-exports |
| `storage.rs` | Shared storage factory, path resolution, project + session address helpers |
| `scope.rs` | Shared `scope::`/`path::` validation and project resolution |
| `color.rs` | Hand-rolled ANSI color helpers for `.show`/`.tail` output |
| `format.rs` | Content-block rendering, relative/absolute time formatting, width-aware eliding |
| `field_selector.rs` | `FieldSelector` type — `.show`'s `fields::` attribute-projection parsing/validation |
| `status.rs` | `.status` command — project stats for a path |
| `list.rs` | `.list` command — session/conversation listing with filters |
| `show.rs` | `.show` command — session and project content viewer |
| `count.rs` | `.count` command — fast entry/session/project/conversation counters |
| `search.rs` | `.search` command — full-text search across session content |
| `export.rs` | `.export` command — session export to markdown/JSON/text |
| `projects.rs` | `.projects` command — agent-aware session-first view; family/conversation types |
| `projects_overview.rs` | `.projects` terse rendering — flat recency table and directory tree |
| `session.rs` | `.project.path`, `.project.exists`, `.session.dir`, `.session.ensure` commands |
| `tail.rs` | `.tail` command — last N conversation turns of a session |
| `usage.rs` | `.usage` command — per-session usage table (turns, tokens, duration, dir) |
| `rollup.rs` | `.rollup` command — flexible grouped/filtered/sorted/projected token-usage rollup |
| `cost.rs` | `.cost` command — per-conversation cost table with agent fold-in and pricing |
