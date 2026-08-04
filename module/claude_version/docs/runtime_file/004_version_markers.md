# Runtime File: Version Markers

### Scope

- **Purpose**: Document the on-disk store persisting user-defined custom version marker entries for the `.version.mark` command.
- **Responsibility**: Describe the markers file path, format, owner module, lifecycle triggers, and crash durability classification.
- **In Scope**: File path, JSON schema, creation and mutation triggers, read consumers, durability.
- **Out of Scope**: Built-in alias resolution (→ `feature/001_version_management.md`), `.version.mark` command behavior (→ `cli/command/version.md#command-17-versionmark`).

### Abstract

Persists user-defined custom version markers created via `.version.mark`. Each entry maps a short name to a version spec and an optional description. Read by `.version.list` (aliases mode), `.version.install`, and `.version.guard` when resolving a `version::` spec.

### Path

`~/.claude/version-markers.json`

Resolution:
1. `$HOME/.claude/version-markers.json` — primary path
2. If `HOME` is unset, custom marker resolution is skipped and only built-in aliases are available.

### Format

JSON object with a top-level `"markers"` array. Each element is an object with three string fields:

```json
{
  "markers": [
    {
      "name": "team-pin",
      "value": "2.1.220",
      "description": ""
    }
  ]
}
```

- `name` — valid marker name (`[a-z][a-z0-9-]*`, max 32 chars, cannot shadow built-in aliases)
- `value` — stored version spec (semver or built-in alias such as `stable`)
- `description` — optional human-readable note; empty string when not provided

### Owner

`claude_version_core/src/version.rs` — `save_custom_marker()` and `remove_custom_marker()`. Both functions use an atomic write strategy: write to a temporary sibling file, then rename over the target to avoid partial-write corruption. The parent directory `~/.claude/` is created via `std::fs::create_dir_all` on first write.

### Lifecycle

- **Created:** On the first successful `.version.mark name::N version::V` invocation when no markers file exists.
- **Updated (upsert):** On any `.version.mark name::N version::V` call; the named entry is added or its value replaced.
- **Pruned:** On `.version.mark name::N unset::1`; the named entry is removed; the file is rewritten without it.
- **Read:** On `.version.list` (aliases mode), `.version.install`, and `.version.guard` when resolving a `version::` spec that is not a built-in alias.
- **Never deleted by clv:** The file persists until the user removes it manually or via `unset::1` for each entry.
- **Graceful degradation:** An absent or malformed markers file is treated as an empty marker set; no error is returned to the caller.

### Durability

**Classification:** recoverable

Custom markers represent user intent (pinned versions for team workflows). A missing file causes all custom marker names to be unresolvable until they are re-created via `.version.mark`. Deletion of the file is safe at the system level (no crash) but loses user-defined pinning data.

### Features

| File | Relationship |
|------|-------------|
| [feature/010_custom_markers.md](../feature/010_custom_markers.md) | Custom markers feature: CRUD, validation, and resolution rules |
| [feature/001_version_management.md](../feature/001_version_management.md) | `.version.install` and `.version.guard` that read this file for resolution |
| [cli/command/version.md](../cli/command/version.md#command-17-versionmark) | `.version.mark` command that writes this file |
