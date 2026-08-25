# Invariant Doc Entity

### Scope

**Responsibilities:** Non-negotiable structural constraints for the `claude_journal_viewer` crate.
**In Scope:** Read-only journal access, web server security boundaries, agreement between the documented and the accepted CLI surface.
**Out of Scope:** Journal write constraints (-> `claude_journal/docs/invariant/`).

### Responsibility Table

| # | File | Responsibility |
|---|------|----------------|
| 001 | `001_read_only.md` | Viewer never modifies journal data (except `.prune` which deletes whole files) |
| 002 | `002_localhost_only.md` | Web server binds to localhost by default — remote access requires explicit `bind::` override |
| 003 | `003_cli_surface_consistency.md` | Every description of which command takes which parameter agrees with the binary |
