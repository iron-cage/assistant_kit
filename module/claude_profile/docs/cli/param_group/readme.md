# Parameter Groups

Semantic groupings of parameters that share a common behavioral pattern.

| File | Responsibility |
|------|----------------|
| [001_output_control.md](001_output_control.md) | Output Control: `format::`, `get::` — output serialization format and value extraction |
| [002_field_presence.md](002_field_presence.md) | Field Presence: 16 boolean field-inclusion toggles |
| [003_fetch_behavior.md](003_fetch_behavior.md) | Fetch Behavior: `refresh::`, `live::`, `interval::`, `jitter::`, `trace::`, `touch::`, `imodel::`, `effort::`, `solo::` |
| [004_sort_control.md](004_sort_control.md) | Sort Control: `sort::`, `desc::`, `prefer::` |
| [005_display_control.md](005_display_control.md) | Display Control: `cols::`, `count::`, `offset::`, `only_active::`, `only_next::`, `min_5h::`, `min_7d::`, `only_valid::`, `exclude_exhausted::`, `abs::`, `no_color::` — column visibility, row filtering, display modifiers |
| [006_account_targeting.md](006_account_targeting.md) | Account Targeting: `host::`, `role::`, `inference_provider::` — metadata labels attached to saved account profiles |
| [007_redirect_backend_config.md](007_redirect_backend_config.md) | Redirect Backend Config: `backend::`, `preset::`, `base_url::`, `api_key::`, `redirect_model::` — foreign-backend account creation fields |

**Total:** 7 groups

### Overview Table

| Group | Parameters | Used By |
|-------|------------|---------|
| [Output Control](001_output_control.md) | `format::`, `get::` | `format::`: `.accounts`, `.paths`, `.usage`, `.account.limits`, `.credentials.status`, `.account.inspect`; `get::`: `.usage` only |
| [Field Presence](002_field_presence.md) | `account::`, `sub::`, `tier::`, `token::`, `expires::`, `email::`, `file::`, `saved::`, `display_name::`, `role::`, `billing::`, `model::`, `uuid::`, `capabilities::`, `org_uuid::`, `org_name::` | `.credentials.status` (all 16) |
| [Fetch Behavior](003_fetch_behavior.md) | `refresh::`, `live::`, `interval::`, `jitter::`, `trace::`, `touch::`, `imodel::`, `effort::`, `solo::` | `.usage` (all 9); `.accounts` (8, excl. `solo::`); `.account.use` (`trace::`, `touch::`, `imodel::`, `effort::`); `.account.inspect` (`refresh::`, `trace::`); `.account.save`, `.account.delete`, `.account.limits`, `.account.relogin`, `.account.renewal`, `.credentials.status`, `.paths` (`trace::`) |
| [Sort Control](004_sort_control.md) | `sort::`, `desc::`, `prefer::` | `.usage`, `.accounts` (all 3) |
| [Display Control](005_display_control.md) | `cols::`, `count::`, `offset::`, `only_active::`, `only_next::`, `min_5h::`, `min_7d::`, `only_valid::`, `exclude_exhausted::`, `abs::`, `no_color::` | `.usage`, `.accounts` (all 11) |
| [Account Targeting](006_account_targeting.md) | `host::`, `role::`, `inference_provider::` | `.account.save` (`host::`, `role::`, `inference_provider::`); `.accounts` (`host::` display toggle; `inference_provider` default identity column) |
| [Redirect Backend Config](007_redirect_backend_config.md) | `backend::`, `preset::`, `base_url::`, `api_key::`, `redirect_model::` | `.account.save` (all 5) |

### See Also

- [../param/](../param/readme.md) — individual parameter specifications
- [../004_parameter_interactions.md](../004_parameter_interactions.md) — cross-parameter interaction rules
- [../user_story/](../user_story/readme.md) — user stories that exercise these parameter groups
