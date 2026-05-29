# Parameter Groups

Semantic groupings of parameters that share a common behavioral pattern.

| File | Responsibility |
|------|----------------|
| [001_output_control.md](001_output_control.md) | Output Control: `format::`, `get::` — output serialization format and value extraction |
| [002_field_presence.md](002_field_presence.md) | Field Presence: 17 boolean field-inclusion toggles |
| [003_fetch_behavior.md](003_fetch_behavior.md) | Fetch Behavior: `refresh::`, `live::`, `interval::`, `jitter::`, `trace::`, `touch::`, `imodel::`, `effort::` |
| [004_sort_control.md](004_sort_control.md) | Sort Control: `sort::`, `desc::`, `prefer::`, `next::` |
| [005_display_control.md](005_display_control.md) | Display Control: `cols::`, `count::`, `offset::`, `only_active::`, `only_next::`, `min_5h::`, `min_7d::`, `only_valid::`, `exclude_exhausted::`, `abs::`, `no_color::` — column visibility, row filtering, display modifiers |
| [006_account_targeting.md](006_account_targeting.md) | Account Targeting: `host::` — metadata labels attached to saved account profiles |

**Total:** 6 groups

### Overview Table

| Group | Parameters | Used By |
|-------|------------|---------|
| [Output Control](001_output_control.md) | `format::`, `get::` | `format::`: `.accounts`, `.token.status`, `.paths`, `.usage`, `.account.limits`, `.credentials.status`; `get::`: `.usage` only |
| [Field Presence](002_field_presence.md) | `active::`, `account::`, `sub::`, `tier::`, `token::`, `expires::`, `email::`, `file::`, `saved::`, `display_name::`, `role::`, `billing::`, `model::`, `uuid::`, `capabilities::`, `org_uuid::`, `org_name::` | `.accounts`, `.credentials.status` |
| [Fetch Behavior](003_fetch_behavior.md) | `refresh::`, `live::`, `interval::`, `jitter::`, `trace::`, `touch::`, `imodel::`, `effort::` | `.usage` (all 8); `.account.use` (`trace::`, `touch::`, `imodel::`, `effort::`) |
| [Sort Control](004_sort_control.md) | `sort::`, `desc::`, `prefer::`, `next::` | `.usage` only |
| [Display Control](005_display_control.md) | `cols::`, `count::`, `offset::`, `only_active::`, `only_next::`, `min_5h::`, `min_7d::`, `only_valid::`, `exclude_exhausted::`, `abs::`, `no_color::` | `.usage` only |
| [Account Targeting](006_account_targeting.md) | `host::` | `.account.save` only |

### See Also

- [../param/](../param/readme.md) — individual parameter specifications
- [../004_parameter_interactions.md](../004_parameter_interactions.md) — cross-parameter interaction rules
