# Parameter Groups

Semantic groupings of parameters that share a common behavioral pattern.

| File | Responsibility |
|------|----------------|
| [001_output_control.md](001_output_control.md) | Output Control: `format::` — output serialization format |
| [002_field_presence.md](002_field_presence.md) | Field Presence: 13 boolean field-inclusion toggles |
| [003_fetch_behavior.md](003_fetch_behavior.md) | Fetch Behavior: `refresh::`, `live::`, `interval::`, `jitter::`, `trace::` |
| [004_sort_control.md](004_sort_control.md) | Sort Control: `sort::`, `desc::`, `prefer::` |

**Total:** 4 groups

### Overview Table

| Group | Parameters | Used By |
|-------|------------|---------|
| [Output Control](001_output_control.md) | `format::` | `.accounts`, `.token.status`, `.paths`, `.usage`, `.account.limits`, `.credentials.status` |
| [Field Presence](002_field_presence.md) | `active::`, `account::`, `sub::`, `tier::`, `token::`, `expires::`, `email::`, `file::`, `saved::`, `display_name::`, `role::`, `billing::`, `model::` | `.accounts`, `.credentials.status` |
| [Fetch Behavior](003_fetch_behavior.md) | `refresh::`, `live::`, `interval::`, `jitter::`, `trace::` | `.usage` only |
| [Sort Control](004_sort_control.md) | `sort::`, `desc::`, `prefer::` | `.usage` only |

### See Also

- [../param/](../param/readme.md) — individual parameter specifications
- [../004_parameter_interactions.md](../004_parameter_interactions.md) — cross-parameter interaction rules
