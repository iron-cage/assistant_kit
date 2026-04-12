# preferred_version_spec

The user's preferred version alias or semver constraint written by `cm .version.install`.

### Forms

| | Value |
|-|-------|
| CLI Flag | — |
| Env Var | — |
| Config Key | `preferredVersionSpec` (in `~/.claude/settings.json`) |

### Type

string/null

### Default

`null`

### Description

The user's preferred version alias or semver constraint written by `cm .version.install`. Examples: `"stable"`, `"latest"`, `"2.1.78"`. `null` means no preference is pinned. Used by `cm .version.guard` to detect and restore version drift. Not read by the `claude` binary itself at runtime — it is metadata for `claude_manager`'s version management commands.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |