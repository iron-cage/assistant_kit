# Feature: Custom Version Markers

### Scope

- **Purpose**: Document the `.version.mark` command that creates, updates, and removes named custom version alias markers.
- **Responsibility**: Specify marker CRUD semantics, name constraints, storage format, integration with `version::` resolution, and `.version.list` rendering.
- **In Scope**: `.version.mark` command, `CustomMarker` struct, `version-markers.json` storage, name validation, integration with `.version.install`/`.version.guard`/`.version.list`/`.version.show` (reverse-lookup labeling).
- **Out of Scope**: Built-in alias resolution (→ `feature/001_version_management.md`), settings management (→ `feature/003_settings_management.md`).

### Design

Custom markers extend the two built-in version aliases (`stable`, `latest`) with user- or team-defined names that resolve to arbitrary semver strings or other built-in aliases. They are stored at runtime in `~/.claude/version-markers.json` and accepted anywhere `version::` is accepted (`.version.install`, `.version.guard`), appearing in `.version.list` output alongside built-in aliases tagged as `(custom)`.

**Storage format (`~/.claude/version-markers.json`):**

```json
{
  "markers": [
    {"name": "team-stable", "value": "2.1.220", "description": "Team-approved baseline"},
    {"name": "qa-pin",      "value": "2.1.200", "description": "QA environment lock"}
  ]
}
```

Writes are atomic: the file is written to a temp path then renamed, so a crash during write leaves the previous state intact.

**Name constraints:**
- Must match `[a-z][a-z0-9-]*` (lowercase letter start; then lowercase letters, digits, or hyphens only)
- Maximum 32 characters
- Must not shadow a built-in alias (`stable`, `latest`)

**Resolution order in `resolve_version_spec()`:**
1. Check built-in aliases (`stable`, `latest`) — exact match
2. Check custom markers loaded from `version-markers.json` — exact name match
3. Treat as semver string — validate and pass through

**`.version.list` integration:**
- `mode::aliases` (default) now renders built-in aliases then custom markers
- Text format (v::1+): custom markers tagged `(custom)` or show their description
- JSON format: each entry has `"kind":"builtin"` or `"kind":"custom"`

**`.version.show` integration:**
- `.version.show` performs a reverse-lookup: it collects all custom markers whose `value` equals the installed semver and displays them as inline labels
- Text format (v::1): labels rendered as bracketed names, e.g. `2.1.220  [team-pin]`; omitted when no markers match
- Text format (v::2): structured block — `version:` / `labels:` with per-label kind annotation
- JSON format: `"labels"` array where each entry carries `"name"`, `"kind": "custom"`, and optional `"description"`
- Only exact semver matches are included; markers with a `value` that is itself a named alias are not transitively resolved

**Error handling:**
- `version-markers.json` absent or empty → treated as no custom markers (not an error)
- `version-markers.json` contains invalid JSON → treated as no custom markers (graceful degradation, same as malformed settings)
- I/O error writing → exit 1 with error message (only on set/unset paths, not on list/read)

### Sources

| File | Relationship |
|------|-------------|
| `../../src/commands/version.rs` | `.version.mark` command handler (`version_mark_routine`) |
| `../../../claude_version_core/src/version.rs` | `CustomMarker`, `load_custom_markers()`, `save_custom_marker()`, `remove_custom_marker()`, `validate_marker_name()` |
| `../../../claude_core/src/paths.rs` | `ClaudePaths::markers_file()` → `~/.claude/version-markers.json` |

### Tests

| File | Relationship |
|------|-------------|
| [tests/docs/feature/010_custom_markers.md](../../tests/docs/feature/010_custom_markers.md) | Feature test spec |
| [tests/docs/cli/command/17_version_mark.md](../../tests/docs/cli/command/17_version_mark.md) | Integration test spec |
