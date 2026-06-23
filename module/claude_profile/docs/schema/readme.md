# Schema

### Scope

- **Purpose**: Define the on-disk file formats written and read by `claude_profile`, including field names, types, and default values.
- **Responsibility**: Authoritative structural reference for every JSON/text file clp touches — credential snapshots, account metadata, settings, and state markers.
- **In Scope**: All files under the credential store (`{name}.credentials.json`, `{name}.json`, `_active_*`), all global Claude files clp reads or writes (`~/.claude/.credentials.json`, `~/.claude/settings.json`, `~/.claude.json`), and path resolution schemas (`PersistPaths`, `ClaudePaths`).
- **Out of Scope**: HTTP API request/response payloads; in-memory type definitions (see `claude_profile_core/src/account.rs`); read-only paths owned by `claude` binary.

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [Credential Snapshot (`{name}.credentials.json`)](001_credentials_json.md) | Per-account OAuth credential snapshot | ✅ |
| 002 | [Account Metadata (`{name}.json`)](002_account_json.md) | Per-account supplementary metadata — unified field table | ✅ |
| 003 | [File Topology (`ClaudePaths`)](003_file_topology.md) | Canonical `~/.claude/` paths exposed via `ClaudePaths` | ✅ |
| 004 | [Storage Root (`PersistPaths`)](004_storage_root.md) | Persistent storage root resolution via `$PRO`/`$HOME` | ✅ |
| 005 | [Active Marker (`_active_{host}_{user}`)](005_active_marker.md) | Per-machine active-account marker file format | ✅ |
| 006 | [Session Settings (`~/.claude/settings.json`)](006_settings_json.md) | Fields in `settings.json` that `clp` reads or writes | ✅ |
| 007 | [Claude State (`~/.claude.json`)](007_claude_json.md) | Fields in `~/.claude.json` that `clp` reads (read-only) | ✅ |
