# Parameter Groups

Semantic groupings of parameters that share a common behavioral pattern.

| File | Responsibility |
|------|----------------|
| [01_output_control.md](01_output_control.md) | Output Control: `format::` — output serialization format |
| [02_field_presence.md](02_field_presence.md) | Field Presence: 13 boolean field-inclusion toggles |
| [03_fetch_behavior.md](03_fetch_behavior.md) | Fetch Behavior: `refresh::`, `live::`, `interval::`, `jitter::`, `trace::` |

**Total:** 3 groups

### Overview Table

| Group | Parameters | Used By |
|-------|------------|---------|
| [Output Control](01_output_control.md) | `format::` | `.accounts`, `.token.status`, `.paths`, `.usage`, `.account.limits`, `.credentials.status` |
| [Field Presence](02_field_presence.md) | `active::`, `account::`, `sub::`, `tier::`, `token::`, `expires::`, `email::`, `file::`, `saved::`, `display_name::`, `role::`, `billing::`, `model::` | `.accounts`, `.credentials.status` |
| [Fetch Behavior](03_fetch_behavior.md) | `refresh::`, `live::`, `interval::`, `jitter::`, `trace::` | `.usage` only |

### See Also

- [../param/](../param/readme.md) — individual parameter specifications
- [../parameter_interactions.md](../parameter_interactions.md) — cross-parameter interaction rules
