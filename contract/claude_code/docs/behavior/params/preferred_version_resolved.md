# preferred_version_resolved

The concrete semver string resolved at the time of the last `cm .version.install` run.

### Forms

| | Value |
|-|-------|
| CLI Flag | — |
| Env Var | — |
| Config Key | `preferredVersionResolved` (in `~/.claude/settings.json`) |

### Type

string/null

### Default

`null`

### Description

The concrete semver string resolved at the time of the last `cm .version.install` run. For example, if `preferredVersionSpec` is `"stable"`, this field stores the actual version that was installed (e.g. `"2.1.78"`). `null` when the spec is `"latest"` (no pinning). Used by `cm .version.guard` to compare against the currently installed version.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |