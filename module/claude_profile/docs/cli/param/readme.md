# Parameters

All `clp` CLI parameters with type, default, and command coverage.

| File | Responsibility |
|------|----------------|
| [001_name.md](001_name.md) | `name::` — target account identifier |
| [002_format.md](002_format.md) | `format::` / `fmt::` — output serialization format |
| [003_threshold.md](003_threshold.md) | `threshold::` — token expiry warning boundary |
| [004_dry.md](004_dry.md) | `dry::` — dry-run simulation mode |
| [005_account.md](005_account.md) | `account::` — account name field toggle |
| [006_sub.md](006_sub.md) | `sub::` — subscription type field toggle |
| [007_tier.md](007_tier.md) | `tier::` — rate-limit tier field toggle |
| [008_token.md](008_token.md) | `token::` — token status field toggle |
| [009_expires.md](009_expires.md) | `expires::` — token expiry field toggle |
| [010_email.md](010_email.md) | `email::` — email address field toggle |
| [011_file.md](011_file.md) | `file::` — credentials file path field toggle |
| [012_saved.md](012_saved.md) | `saved::` — saved account count field toggle |
| [013_active.md](013_active.md) | `active::` — active/inactive status field toggle |
| [014_display_name.md](014_display_name.md) | `display_name::` — display name field toggle |
| [015_role.md](015_role.md) | `role::` — organisation role field toggle |
| [016_billing.md](016_billing.md) | `billing::` — billing type field toggle |
| [017_model.md](017_model.md) | `model::` — active model field toggle |
| [018_current.md](018_current.md) | `current::` — current live account field toggle |
| [019_refresh.md](019_refresh.md) | `refresh::` — expired token auto-refresh on auth error |
| [020_live.md](020_live.md) | `live::` — continuous quota refresh loop |
| [021_interval.md](021_interval.md) | `interval::` — live mode cycle duration |
| [022_jitter.md](022_jitter.md) | `jitter::` — live mode cycle timing variance |
| [023_trace.md](023_trace.md) | `trace::` — diagnostic trace output to stderr |
| [024_field.md](024_field.md) | `field::` — single-path output selector |
| [025_sort.md](025_sort.md) | `sort::` — row ordering strategy for quota table |
| [026_desc.md](026_desc.md) | `desc::` — sort direction with context-sensitive default |
| [027_prefer.md](027_prefer.md) | `prefer::` — weekly quota column for sort heuristics |
| [028_uuid.md](028_uuid.md) | `uuid::` — stable user ID field toggle (opt-in) |
| [029_capabilities.md](029_capabilities.md) | `capabilities::` — product capabilities list toggle (opt-in) |
| [030_org_uuid.md](030_org_uuid.md) | `org_uuid::` — organisation UUID field toggle (opt-in) |
| [031_org_name.md](031_org_name.md) | `org_name::` — organisation display name field toggle (opt-in) |
| [032_next.md](032_next.md) | `next::` — recommendation strategy selector for quota table |
| [033_cols.md](033_cols.md) | `cols::` — column visibility modifiers for quota table |
| [034_touch.md](034_touch.md) | `touch::` — activate idle accounts' 5h windows via isolated subprocess |
| [035_imodel.md](035_imodel.md) | `imodel::` — isolated subprocess model selection (`auto`, `sonnet`, `opus`, `haiku`, `keep`) |
| [036_effort.md](036_effort.md) | `effort::` — isolated subprocess effort level (`auto`, `low`, `normal`, `high`, `max`) |

**Total:** 36 parameters

### Overview Table

| # | Parameter | Type | Default | Valid Values | Purpose | Used In |
|---|-----------|------|---------|--------------|---------|---------|
| 1 | `name::` | `AccountName` | Varies | Email or prefix | Target account identifier | 5 cmds |
| 2 | `format::` / `fmt::` | `OutputFormat` | `text` | `text`, `json` | Output format | 6 cmds |
| 3 | `threshold::` | `WarningThreshold` | `3600` | Non-negative integer (seconds) | Token expiry warning boundary | 1 cmd |
| 4 | `dry::` | `bool` | `0` | `0`, `1`, `false`, `true` | Dry-run simulation | 3 cmds |
| 5 | `account::` | `bool` | `1` | `0`, `1` | Account name line toggle | 1 cmd |
| 6 | `sub::` | `bool` | `1` | `0`, `1` | Subscription type line toggle | 2 cmds |
| 7 | `tier::` | `bool` | `1` | `0`, `1` | Rate-limit tier line toggle | 2 cmds |
| 8 | `token::` | `bool` | `1` | `0`, `1` | Token status line toggle | 1 cmd |
| 9 | `expires::` | `bool` | `1` | `0`, `1` | Token expiry line toggle | 2 cmds |
| 10 | `email::` | `bool` | `1` | `0`, `1` | Email address line toggle | 2 cmds |
| 11 | `file::` | `bool` | `0` | `0`, `1` | Credentials file path toggle (opt-in) | 1 cmd |
| 12 | `saved::` | `bool` | `0` | `0`, `1` | Saved account count toggle (opt-in) | 1 cmd |
| 13 | `active::` | `bool` | `1` | `0`, `1` | Active/inactive status toggle | 1 cmd |
| 14 | `display_name::` | `bool` | `0` | `0`, `1` | Display name toggle (opt-in) | 2 cmds |
| 15 | `role::` | `bool` | `0` | `0`, `1` | Organisation role toggle (opt-in) | 2 cmds |
| 16 | `billing::` | `bool` | `0` | `0`, `1` | Billing type toggle (opt-in) | 2 cmds |
| 17 | `model::` | `bool` | `0` | `0`, `1` | Active model toggle (opt-in) | 2 cmds |
| 18 | `current::` | `bool` | `1` | `0`, `1` | Current (live) account line toggle | 1 cmd |
| 19 | `refresh::` | `bool` | `1` | `0`, `1` | Auto-refresh expired tokens on auth error | 1 cmd |
| 20 | `live::` | `bool` | `0` | `0`, `1` | Continuous refresh loop | 1 cmd |
| 21 | `interval::` | `u64` | `30` | ≥ 30 (seconds) | Live mode cycle duration | 1 cmd |
| 22 | `jitter::` | `u64` | `0` | 0 ≤ jitter ≤ interval | Live mode cycle timing variance | 1 cmd |
| 23 | `trace::` | `bool` | `0` | `0`, `1` | Diagnostic trace output to stderr | 11 cmds |
| 24 | `field::` | `String` | `""` (show all) | `base`, `credentials`, `credential_store`, `projects`, `stats`, `settings`, `session_env`, `sessions` | Single-path output selector | 1 cmd |
| 25 | `sort::` | `enum` | `renew` | `name`, `endurance`, `drain`, `renew`, `next` | Row ordering strategy for quota table | 1 cmd |
| 26 | `desc::` | `bool` | context-sensitive | `0`, `1`, `false`, `true` | Sort direction; default per `sort::` strategy | 1 cmd |
| 27 | `prefer::` | `enum` | `any` | `any`, `opus`, `sonnet` | Weekly quota column for sort heuristics | 1 cmd |
| 28 | `uuid::` | `bool` | `0` | `0`, `1` | Stable user ID toggle (opt-in) | 2 cmds |
| 29 | `capabilities::` | `bool` | `0` | `0`, `1` | Product capabilities list toggle (opt-in) | 2 cmds |
| 30 | `org_uuid::` | `bool` | `0` | `0`, `1` | Organisation UUID toggle (opt-in) | 2 cmds |
| 31 | `org_name::` | `bool` | `0` | `0`, `1` | Organisation display name toggle (opt-in) | 2 cmds |
| 32 | `next::` | `enum` | `drain` | `endurance`, `drain` | Recommendation strategy selector | 1 cmd |
| 33 | `cols::` | `string` | `""` | `+col_id`, `-col_id` modifiers | Column visibility modifiers | 1 cmd |
| 34 | `touch::` | `bool` | `1` | `0`, `1`, `false`, `true` | Activate idle accounts' 5h windows | 2 cmds |
| 35 | `imodel::` | `enum` | `auto` | `auto`, `sonnet`, `opus`, `haiku`, `keep` | Isolated subprocess model selection | 2 cmds |
| 36 | `effort::` | `enum` | `auto` | `auto`, `low`, `normal`, `high`, `max` | Isolated subprocess effort level | 2 cmds |

*Params 1 = Account Targeting; param 2 = Output Control group; params 5–18, 28–31 = Field Presence group; params 19–23, 34–36 = Fetch Behavior group; param 24 = Output Selection group; params 25–27, 32 = Sort Control group; param 33 = Display Control group*

### See Also

- [../type/](../type/readme.md) — types used by parameters
- [../command/](../command/readme.md) — commands that accept these parameters
- [../param_group/](../param_group/readme.md) — parameter group definitions
