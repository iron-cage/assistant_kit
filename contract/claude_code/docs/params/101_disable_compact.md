# Parameter: disable_compact

### Forms

| Form | Value |
|------|-------|
| Env Var | `DISABLE_COMPACT` |

### Type

boolean (presence-activated)

### Default

Not set (compaction enabled)

### Description

Disables ALL compaction — both automatic and manual `/compact`. Stronger than
`DISABLE_AUTO_COMPACT` which only blocks automatic triggers.

### Since

pre-v1.0 (unverified)

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |
| doc | [100_disable_auto_compact.md](100_disable_auto_compact.md) | Disable auto-compaction only |
