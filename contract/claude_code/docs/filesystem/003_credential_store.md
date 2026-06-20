# Filesystem: Credential Store

### Scope

- **Purpose**: Document the credential store paths — per-account credential files and the active account marker.
- **Responsibility**: Authoritative instance for the `{credential_store}/` cluster — `_active` file and per-account `.credentials.json` files; path resolution.
- **In Scope**: `{credential_store}/_active` (active account name), `{credential_store}/{name}.credentials.json` (per-account snapshots), resolution to `$PRO/.persistent/` or `$HOME/.persistent/`.
- **Out of Scope**: `~/.claude/.credentials.json` (active OAuth token, → [001_claude_home.md](001_claude_home.md)); settings format (→ [`../settings/`](../settings/readme.md)).

### Paths

| Path | Type | Access | Used By | Purpose |
|------|------|--------|---------|---------|
| `{credential_store}/` | dir | R/W | `.account.*` | Credential store directory; created on first save; absent = no saved accounts |
| `{credential_store}/_active` | file | R/W | `.account.switch`, `.account.status`, `.status` | Active account name (single line, plain text) |
| `{credential_store}/{name}.credentials.json` | file | R/W/del | `.account.save`, `.account.switch`, `.account.delete`, `.account.list` | Per-account credential snapshot; `name` is an email address |

### Resolution

`{credential_store}` resolves based on environment:

```
$PRO is a directory → {credential_store} = $PRO/.persistent/claude/credential/
otherwise           → {credential_store} = $HOME/.persistent/claude/credential/
```

Implemented in `PersistPaths::credential_store()`. The directory is created on first `.account.save` operation if it does not exist.

### _active File

`_active` is a single-line plain text file containing the active account name (typically an email address). Written by `.account.switch`, read by `.account.status` and `.status`.

- Absent: no active account selected
- Present: content is the name of the active account (matches a `{name}.credentials.json` file)

### Per-Account Credential Files

`{name}.credentials.json` stores a full credential snapshot for one account. Operations:
- **save** (`.account.save`): writes current `~/.claude/.credentials.json` content
- **switch** (`.account.switch`): copies this file to `~/.claude/.credentials.json` + updates `_active`
- **delete** (`.account.delete`): removes the file
- **list** (`.account.list`): reads filenames to enumerate saved accounts

### Since

pre-v1.0 (unverified)

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| entity | [readme.md](readme.md) | Filesystem master index: full directory tree, path reference table |
| filesystem | [001_claude_home.md](001_claude_home.md) | `~/.claude/.credentials.json` (active OAuth token, separate from this store) |
| formats | [`../formats/002_credentials.md`](../formats/002_credentials.md) | Credential JSON structure |
| source | `../../../../module/claude_profile/src/paths.rs` | `PersistPaths::credential_store()` implementation |
