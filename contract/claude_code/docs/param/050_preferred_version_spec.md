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

### Since

pre-v1.0 (unverified)

### Description

The user's preferred version alias or semver constraint written by `cm .version.install`. Examples: `"stable"`, `"latest"`, `"2.1.78"`. `null` means no preference is pinned. Used by `cm .version.guard` to detect and restore version drift. Not read by the `claude` binary itself at runtime — it is metadata for `claude_version`'s version management commands.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |
| doc | [049_preferred_version_resolved.md](049_preferred_version_resolved.md) | Resolved concrete version (companion) |
| doc | [../settings/003_version_lock.md](../settings/003_version_lock.md) | Actual reader/writer: this repo's own `cm .version.install`/`.version.guard` — verified absent from the `claude` binary itself, so the real `claude install`/`claude update` subcommands neither read nor write this key |
| doc | [../pattern/001_version_pinning.md](../pattern/001_version_pinning.md) | Synthesis: full version-pinning landscape |