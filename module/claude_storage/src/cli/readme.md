# src/cli/

CLI command routines for `claude_storage`. Each file owns one command (or a
closely related cluster of commands), keeping individual files focused and
navigable.

## Responsibility Table

| File | Responsibility |
|------|----------------|
| `mod.rs` | Module declarations and public re-exports |
| `storage.rs` | Shared storage factory, verbosity validation, path + project + session helpers |
| `format.rs` | Entry content formatting, timestamp formatting, safe UTF-8 truncation |
| `status.rs` | `.status` command — project stats for a path |
| `list.rs` | `.list` command — session/conversation listing with verbosity levels |
| `show.rs` | `.show` command — session and project content viewer |
| `count.rs` | `.count` command — fast entry/session/project/conversation counters |
| `search.rs` | `.search` command — full-text search across session content |
| `export.rs` | `.export` command — session export to markdown/JSON/text |
| `projects.rs` | `.projects` command — agent-aware session-first view; family/conversation types |
| `session.rs` | `.project.path`, `.project.exists`, `.session.dir`, `.session.ensure` commands |
