# Parameter: disable_auto_compact

### Forms

| Form | Value |
|------|-------|
| Env Var | `DISABLE_AUTO_COMPACT` |

### Type

boolean (presence-activated)

### Default

Not set (auto-compaction enabled)

### Description

Disables automatic context compaction. Manual `/compact` still works. For
disabling all compaction (auto + manual), use `DISABLE_COMPACT` instead.

### Since

pre-v1.0 (unverified)

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |
| doc | [101_disable_compact.md](101_disable_compact.md) | Disable all compaction |
