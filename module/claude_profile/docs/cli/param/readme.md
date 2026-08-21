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
| [013_active.md](013_active.md) | `active::` — **REMOVED** (Feature 065); replaced by `assignee::USER@MACHINE name::X` |
| [014_display_name.md](014_display_name.md) | `display_name::` — display name field toggle |
| [015_role.md](015_role.md) | `role::` — organisation role field toggle |
| [016_billing.md](016_billing.md) | `billing::` — billing type field toggle |
| [017_model.md](017_model.md) | `model::` — active model field toggle |
| [018_current.md](018_current.md) | `current::` — current live account field toggle |
| [019_refresh.md](019_refresh.md) | `refresh::` — expired token refresh on auth error or locally-expired `expiresAt` |
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
| [032_next.md](032_next.md) | `next::` — REMOVED; recommendation driven by `sort::` |
| [033_cols.md](033_cols.md) | `cols::` — column visibility modifiers for quota table |
| [034_touch.md](034_touch.md) | `touch::` — activate idle accounts' 5h windows via isolated subprocess |
| [035_imodel.md](035_imodel.md) | `imodel::` — isolated subprocess model selection (`auto`, `sonnet`, `opus`, `haiku`, `keep`) |
| [036_effort.md](036_effort.md) | `effort::` — isolated subprocess effort level (`auto`, `low`, `normal`, `high`, `max`) |
| [037_count.md](037_count.md) | `count::` — maximum rows to display in quota table (0 = all) |
| [038_offset.md](038_offset.md) | `offset::` — skip first N rows from filtered result |
| [039_only_active.md](039_only_active.md) | `only_active::` — show only the active account row |
| [040_only_next.md](040_only_next.md) | `only_next::` — show only the recommended next account row |
| [041_min_5h.md](041_min_5h.md) | `min_5h::` — minimum 5h Left percentage threshold filter |
| [042_min_7d.md](042_min_7d.md) | `min_7d::` — minimum 7d Left percentage threshold filter |
| [043_only_valid.md](043_only_valid.md) | `only_valid::` — hide invalid-token (🔴) account rows |
| [044_exclude_exhausted.md](044_exclude_exhausted.md) | `exclude_exhausted::` — hide exhausted (🟡) and invalid (🔴) account rows |
| [045_get.md](045_get.md) | `get::` — single column value extraction for first filtered row |
| [046_abs.md](046_abs.md) | `abs::` — REMOVED; registered no-op deleted (absolute-count display never implemented) |
| [047_no_color.md](047_no_color.md) | `no_color::` — strip emoji and ANSI colors from output |
| [048_host.md](048_host.md) | `host::` — host/machine label captured at account save time |
| [049_at.md](049_at.md) | `at::` — absolute ISO-8601 UTC renewal timestamp for `.account.renewal` |
| [050_from_now.md](050_from_now.md) | `from_now::` — signed duration delta from now for `.account.renewal` |
| [051_clear.md](051_clear.md) | `clear::` — remove billing renewal override from `{name}.json` |
| [052_role.md](052_role.md) | `role::` (metadata label) — **REMOVED** (Feature 075); superseded by `tags::` (082) with lazy field migration |
| [053_for.md](053_for.md) | `for::` — **REMOVED** (Feature 064); absorbed into `active::` value |
| [054_set_model.md](054_set_model.md) | `set_model::` — explicit Claude Code session model write to `settings.json` |
| [055_set.md](055_set.md) | `set::` — **RETIRED** (Feature 035); replaced by `model::` (076) on the unified `.model` |
| [056_unclaim.md](056_unclaim.md) | `unclaim::` — **REMOVED** (Feature 064); replaced by `owner::0` sentinel |
| [057_assign.md](057_assign.md) | `assign::` — **REMOVED** (Feature 064); replaced by `assignee::USER@MACHINE name::X` |
| [058_force.md](058_force.md) | `force::` — bypass G5–G8 ownership enforcement on mutation commands |
| [059_rotate.md](059_rotate.md) | `rotate::` — after quota table render, switch to the footer-recommended account; mutually exclusive with `live::1`; G5 ownership gate; `dry::1` previews |
| [060_solo.md](060_solo.md) | `solo::` — token conservation mode restricting all credential-consuming operations to the current+owned account; others use `approximate_quota()` |
| [061_who.md](061_who.md) | `who::` — sessions table visibility in `.usage` (auto: shown when >1 active marker) |
| [062_owner.md](062_owner.md) | `owner::` — ownership set (`USER@MACHINE`) or release (`owner::0`); batch via comma-list `name::` |
| [063_assignee.md](063_assignee.md) | `assignee::` — `USER@MACHINE` (or sentinel `0` = current machine) mutation param: assign/unassign active-account marker (Feature 065) |
| [064_id.md](064_id.md) | `id::` — provider name to select via `~/.clr/config.toml`; activates set mode on `.provider.select` (narrowed, Feature 035 — formerly also pinned `.model.select`'s subprocess model) |
| [065_offline.md](065_offline.md) | `offline::` — use static embedded model catalog instead of live `GET /v1/models`; no credentials required |
| [066_reset.md](066_reset.md) | `reset::` — remove `provider` from `~/.clr/config.toml`'s user tier; idempotent; mutually exclusive with `id::` on `.provider.select` (narrowed, Feature 035 — formerly also reset `.model.select`'s subprocess model) |
| [067_lock.md](067_lock.md) | `lock::` — set/clear `claim_lock` on an account; ungated write; batch via comma-list `name::` |
| [068_reserve.md](068_reserve.md) | `reserve::` — set/clear `reserve` on an account; ungated write; batch via comma-list `name::` |
| [069_backend.md](069_backend.md) | `backend::` — selects `anthropic` (OAuth) or `redirect` (foreign endpoint) backend at account save time |
| [070_base_url.md](070_base_url.md) | `base_url::` — redirect target's API base URL; redirect-only |
| [071_api_key.md](071_api_key.md) | `api_key::` — redirect target's static API key; redirect-only |
| [072_redirect_model.md](072_redirect_model.md) | `redirect_model::` — redirect target's own model identifier; redirect-only |
| [073_inference_provider.md](073_inference_provider.md) | `inference_provider::` — inference provider label written to `{name}.json` at account save; governs Gate 10 rotation grouping |
| [074_preset.md](074_preset.md) | `preset::` — named provider preset pre-filling `backend::`/`base_url::`/`inference_provider::`; only `kimi` recognized today |
| [075_scope.md](075_scope.md) | `scope::` — backing-store router on `.model`: `session` (`~/.claude/settings.json`) or `subprocess` (`~/.clr/config.toml`) |
| [076_model_value.md](076_model_value.md) | `model::` — model to write for the selected `scope::` on `.model`; replaces retired `set::` and `.model.select`'s `id::` |
| [077_effort_level.md](077_effort_level.md) | `effort_level::` — effort to write for the selected `scope::` on `.model`; new direct control, deliberately distinct from the unrelated ephemeral `effort::` (036) |
| [078_reset_model.md](078_reset_model.md) | `reset_model::` — remove the model key for the selected `scope::` on `.model`; replaces `.model.select`'s `reset::` for the model concept |
| [079_reset_effort_level.md](079_reset_effort_level.md) | `reset_effort_level::` — remove the effort key for the selected `scope::` on `.model`; new, no prior equivalent on either store |
| [080_stalest.md](080_stalest.md) | `stalest::` — restrict HTTP fetch to the K accounts with the oldest quota cache; others render from cache |
| [081_max_age.md](081_max_age.md) | `max_age::` — staleness eligibility threshold for `stalest::`; fully-fresh fleet fetches nothing |
| [082_tags.md](082_tags.md) | `tags::` — tag set write (`.account.save`/`.account.tag` replace) and subset row filter (`.accounts`) |
| [083_add.md](083_add.md) | `add::` — union tags into an account's set on `.account.tag` |
| [084_remove.md](084_remove.md) | `remove::` — remove tags from an account's set on `.account.tag`; idempotent |
| [085_include.md](085_include.md) | `include::` — replace a Tag Filter's include side on `.identity.filter` |
| [086_exclude.md](086_exclude.md) | `exclude::` — replace a Tag Filter's exclude side on `.identity.filter` |
| [087_identity.md](087_identity.md) | `identity::` — target another seat's Tag Filter on `.identity.filter` |
| [088_alert.md](088_alert.md) | `alert::` — burn-rate alert horizon in minutes for `.usage` footer warnings |

**Total:** 79 active parameters (Feature 023 deprecated: param 032 `next::` REMOVED, absorbed into feature 020's `sort::`; Feature 065: param 013 `active::` REMOVED; param 063 `assignee::` added as replacement; Feature 064: params 053 `for::`, 056 `unclaim::`, 057 `assign::` REMOVED; param 062 `owner::` extended with `owner::0` sentinel + batch; Feature 070: params 067 `lock::`, 068 `reserve::` added; Feature 071: params 069 `backend::`, 070 `base_url::`, 071 `api_key::`, 072 `redirect_model::` added; Feature 072: param 073 `inference_provider::` added; Feature 073: param 074 `preset::` added; Feature 035: param 055 `set::` RETIRED — `.model`/`.model.select` merged; params 064 `id::`/066 `reset::` narrowed to `.provider.select` only; params 075 `scope::`, 076 `model::`, 077 `effort_level::`, 078 `reset_model::`, 079 `reset_effort_level::` added; task 499: params 080 `stalest::`, 081 `max_age::` added; audit remediation: param 046 `abs::` REMOVED — registered no-op, absolute-count display never implemented; Features 075/076: params 082 `tags::`, 083 `add::`, 084 `remove::`, 085 `include::`, 086 `exclude::`, 087 `identity::` added; param 052 `role::` REMOVED — superseded by `tags::` with lazy field migration)

### Overview Table

| # | Parameter | Type | Default | Valid Values | Purpose | Used In |
|---|-----------|------|---------|--------------|---------|---------|
| 1 | `name::` | `AccountName` | Varies | Email or prefix | Target account identifier | 8 cmds |
| 2 | `format::` / `fmt::` | `OutputFormat` | `text` | `text`, `json`, `table`; `value`/`tsv`/`plain` (.usage only) | Output format | 7 cmds |
| 3 | `threshold::` | `WarningThreshold` | `3600` | Non-negative integer (seconds) | Token expiry warning boundary | 1 cmd |
| 4 | `dry::` | `bool` | `0` | `0`, `1`, `false`, `true` | Dry-run simulation | 6 cmds |
| 5 | `account::` | `bool` | `1` | `0`, `1` | Account name line toggle | 1 cmd |
| 6 | `sub::` | `bool` | `1` | `0`, `1` | Subscription type line toggle | 2 cmds |
| 7 | `tier::` | `bool` | `1` | `0`, `1` | Rate-limit tier line toggle | 2 cmds |
| 8 | `token::` | `bool` | `1` | `0`, `1` | Token status line toggle | 1 cmd |
| 9 | `expires::` | `bool` | `1` | `0`, `1` | Token expiry line toggle | 2 cmds |
| 10 | `email::` | `bool` | `1` | `0`, `1` | Email address line toggle | 2 cmds |
| 11 | `file::` | `bool` | `0` | `0`, `1` | Credentials file path toggle (opt-in) | 1 cmd |
| 12 | `saved::` | `bool` | `0` | `0`, `1` | Saved account count toggle (opt-in) | 1 cmd |
| 13 | `active::` | — | — | — | REMOVED (Feature 065) — use `assignee::USER@MACHINE name::X` | — |
| 14 | `display_name::` | `bool` | `0` | `0`, `1` | Display name toggle (opt-in) | 2 cmds |
| 15 | `role::` | `bool` | `0` | `0`, `1` | Organisation role toggle (opt-in) | 2 cmds |
| 16 | `billing::` | `bool` | `0` | `0`, `1` | Billing type toggle (opt-in) | 2 cmds |
| 17 | `model::` | `bool` | `0` | `0`, `1` | Active model toggle (opt-in) | 2 cmds |
| 18 | `current::` | `bool` | `1` | `0`, `1` | Current (live) account line toggle | 1 cmd |
| 19 | `refresh::` | `bool` | `1` | `0`, `1` | Refresh expired OAuth token; trigger is auth error or locally-expired `expiresAt` | 3 cmds |
| 20 | `live::` | `bool` | `0` | `0`, `1` | Continuous refresh loop | 1 cmd |
| 21 | `interval::` | `u64` | `30` | ≥ 30 (seconds) | Live mode cycle duration | 1 cmd |
| 22 | `jitter::` | `u64` | `0` | 0 ≤ jitter ≤ interval | Live mode cycle timing variance | 1 cmd |
| 23 | `trace::` | `bool` | `0` | `0`, `1` | Diagnostic trace output to stderr | 13 cmds |
| 24 | `field::` | `String` | `""` (show all) | `base`, `credentials`, `credential_store`, `projects`, `stats`, `settings`, `session_env`, `sessions` | Single-path output selector | 1 cmd |
| 25 | `sort::` | `enum` | `renew` | `name`, `renew`, `renews` | Row ordering strategy for quota table | 1 cmd |
| 26 | `desc::` | `bool` | context-sensitive | `0`, `1`, `false`, `true` | Sort direction; default per `sort::` strategy | 1 cmd |
| 27 | `prefer::` | `enum` | `any` | `any`, `opus`, `sonnet` | Weekly quota column for sort heuristics | 1 cmd |
| 28 | `uuid::` | `bool` | `0` | `0`, `1` | Stable user ID toggle (opt-in) | 2 cmds |
| 29 | `capabilities::` | `bool` | `0` | `0`, `1` | Product capabilities list toggle (opt-in) | 2 cmds |
| 30 | `org_uuid::` | `bool` | `0` | `0`, `1` | Organisation UUID toggle (opt-in) | 2 cmds |
| 31 | `org_name::` | `bool` | `0` | `0`, `1` | Organisation display name toggle (opt-in) | 2 cmds |
| 32 | `next::` | — | — | — | REMOVED — recommendation driven by `sort::` | — |
| 33 | `cols::` | `string` | `""` | `+col_id`, `-col_id` modifiers | Column visibility modifiers | 1 cmd |
| 34 | `touch::` | `bool` | `1` | `0`, `1`, `false`, `true` | Activate idle accounts' 5h windows | 2 cmds |
| 35 | `imodel::` | `enum` | `auto` | `auto`, `sonnet`, `opus`, `haiku`, `keep` | Isolated subprocess model selection | 2 cmds |
| 36 | `effort::` | `enum` | `auto` | `auto`, `low`, `normal`, `high`, `max` | Isolated subprocess effort level | 2 cmds |
| 37 | `count::` | `u64` | `0` | Non-negative integer | Max rows to display (0 = all) | 1 cmd |
| 38 | `offset::` | `u64` | `0` | Non-negative integer | Skip first N rows from result | 1 cmd |
| 39 | `only_active::` | `bool` | `0` | `0`, `1` | Show only active account row | 1 cmd |
| 40 | `only_next::` | `bool` | `0` | `0`, `1` | Show only the recommended account row | 1 cmd |
| 41 | `min_5h::` | `u8` | `0` | `0`–`100` | Minimum 5h Left % filter | 1 cmd |
| 42 | `min_7d::` | `u8` | `0` | `0`–`100` | Minimum 7d Left % filter | 1 cmd |
| 43 | `only_valid::` | `bool` | `0` | `0`, `1` | Hide 🔴 invalid-token rows | 1 cmd |
| 44 | `exclude_exhausted::` | `bool` | `0` | `0`, `1` | Hide 🟡 and 🔴 rows | 1 cmd |
| 45 | `get::` | `string` | `""` | Field IDs (see 045_get.md) | Single column value extraction | 1 cmd |
| 46 | `abs::` | — | — | — | REMOVED — registered no-op deleted; absolute-count display never implemented | — |
| 47 | `no_color::` | `bool` | `0` | `0`, `1` | Strip emoji and ANSI from output | 1 cmd |
| 48 | `host::` | `string` | `""` (auto) | Any string | Machine/host label at save; display toggle at list | 2 cmds |
| 49 | `at::` | `string` | *(omit)* | ISO-8601 UTC datetime | Absolute renewal timestamp for `.account.renewal` | 1 cmd |
| 50 | `from_now::` | `string` | *(omit)* | `+`/`-` duration (e.g., `+3h30m`) | Signed delta from now for `.account.renewal` | 1 cmd |
| 51 | `clear::` | `bool` | `0` | `0`, `1` | Remove billing renewal override | 1 cmd |
| 52 | `role::` (metadata label) | — | — | — | REMOVED (Feature 075) — use `tags::` (82); lazy field migration on first tag write | — |
| 53 | `for::` | — | — | — | REMOVED (Feature 064) — absorbed into `active::` value (Feature 065: `active::` also REMOVED — use `assignee::`) | — |
| 54 | `set_model::` | `enum` | *(omit)* | `opus`, `sonnet`, `haiku`, `default` | Explicit session model write to `settings.json` | 2 cmds |
| 55 | `set::` | — | — | — | RETIRED (Feature 035) — use `model::` (76) on the unified `.model` | — |
| 56 | `unclaim::` | — | — | — | REMOVED (Feature 064) — use `owner::0` | — |
| 57 | `assign::` | — | — | — | REMOVED (Feature 064) — use `assignee::USER@MACHINE name::X` | — |
| 58 | `force::` | `bool` | `0` | `0`, `1`, `false`, `true` | Bypass G5–G8 ownership enforcement on mutation commands | `.account.use`, `.account.delete`, `.account.relogin`, `.accounts`, `.usage` |
| 59 | `rotate::` | `bool` | `0` | `0`, `1` | After quota table render, switch to footer-recommended account; mutually exclusive with `live::1`; G5 ownership gate | `.usage` |
| 60 | `solo::` | `bool` | `0` | `0`, `1` | Token conservation: restrict all credential-consuming operations to current+owned account; others use `approximate_quota()` | `.usage` |
| 61 | `who::` | `bool` | `auto` | `0` (hide), `1` (show); omit = auto | Sessions table visibility in `.usage` output | `.usage` |
| 62 | `owner::` | `string` | *(omit)* | `USER@MACHINE`, `0` (release) | Set ownership (`USER@MACHINE`) or release (`0`); batch via comma-list `name::` | `.accounts`, `.usage` |
| 63 | `assignee::` | `string` | *(omit)* | `USER@MACHINE`, `0` (current machine) | Assign/unassign active-account marker; `0` sentinel expands to `$USER@$HOSTNAME` (Feature 065) | `.accounts`, `.usage` |
| 64 | `id::` | `string` | *(omit)* | Any non-empty provider name string | Select global inference provider in `~/.clr/config.toml`; activates set mode when present (narrowed, Feature 035) | `.provider.select` |
| 65 | `offline::` | `bool` | `0` | `0`, `1` | Use static embedded model catalog instead of live API; no network call made | `.models` |
| 66 | `reset::` | `bool` | `0` | `0`, `1` | Remove `provider` from `~/.clr/config.toml`'s user tier; idempotent; mutually exclusive with `id::` (narrowed, Feature 035) | `.provider.select` |
| 67 | `lock::` | `bool` | *(omit)* | `0`, `1`, `false`, `true` | Set/clear `claim_lock`; ungated write; batch via comma-list `name::` | `.accounts`, `.usage` |
| 68 | `reserve::` | `bool` | *(omit)* | `0`, `1`, `false`, `true` | Set/clear `reserve`; ungated write; batch via comma-list `name::` | `.accounts`, `.usage` |
| 69 | `backend::` | [`AccountBackend`](../type/005_account_backend.md) (`enum`) | `anthropic` | `anthropic`, `redirect` | Selects OAuth flow or foreign-endpoint redirect at account save time | `.account.save` |
| 70 | `base_url::` | `string` | *(omit; required when `backend::redirect`)* | Non-empty string | Redirect target's API base URL | `.account.save` |
| 71 | `api_key::` | `string` | *(omit; required when `backend::redirect`)* | Non-empty string | Redirect target's static API key | `.account.save` |
| 72 | `redirect_model::` | `string` | *(omit; required when `backend::redirect`)* | Non-empty string | Redirect target's own model identifier | `.account.save` |
| 73 | `inference_provider::` | `string` | *(omit; field absent — reads as `"anthropic"`)* | Any non-empty string | Inference provider label at account save; governs Gate 10 rotation grouping | `.account.save` |
| 74 | `preset::` | `string` | *(omit)* | `kimi` (only recognized value) | Named provider preset pre-filling `backend::`/`base_url::`/`inference_provider::` | `.account.save` |
| 75 | `scope::` | `enum` | `session` | `session`, `subprocess` | Backing-store router — every other parameter on the same `.model` call applies to this store | `.model` |
| 76 | `model::` | `string` | *(omit)* | Scope-dependent — see [076_model_value.md](076_model_value.md) | Write the model key for the selected `scope::` | `.model` |
| 77 | `effort_level::` | `string` | *(omit)* | Scope-dependent — see [077_effort_level.md](077_effort_level.md) | Write the effort key for the selected `scope::` | `.model` |
| 78 | `reset_model::` | `bool` | `0` | `0`, `1` | Remove the model key for the selected `scope::`; mutually exclusive with `model::` | `.model` |
| 79 | `reset_effort_level::` | `bool` | `0` | `0`, `1` | Remove the effort key for the selected `scope::`; mutually exclusive with `effort_level::` | `.model` |
| 80 | `stalest::` | `u32` | *(omit)* | Integer ≥ 1 | Fetch only the K accounts with the oldest quota cache; others render from cache; mutually exclusive with `only_active::1`; bypassed by `rotate::1` | `.usage` |
| 81 | `max_age::` | `u64` | `0` | Seconds ≥ 0 | With `stalest::`, only accounts with cache age > SECS are fetch-eligible; standalone use exits 1 | `.usage` |
| 82 | `tags::` | `string` | *(omit)* | Comma-separated tag list (`[a-z0-9_-]`, 1–64 each) | Tag set write at save / full replace on `.account.tag` / subset row filter on `.accounts` | `.account.save`, `.account.tag`, `.accounts` |
| 83 | `add::` | `string` | *(omit)* | Comma-separated tag list | Union tags into the account's set; one operation per invocation | `.account.tag` |
| 84 | `remove::` | `string` | *(omit)* | Comma-separated tag list | Remove tags from the account's set; absent tags are a no-op | `.account.tag` |
| 85 | `include::` | `string` | *(omit)* | Comma-separated tag list | Replace the Tag Filter's include side; overlap with exclude exits 1 | `.identity.filter` |
| 86 | `exclude::` | `string` | *(omit)* | Comma-separated tag list | Replace the Tag Filter's exclude side; overlap with include exits 1 | `.identity.filter` |
| 87 | `identity::` | `string` | *(omit — current `$USER@$HOSTNAME`)* | `USER@MACHINE` | Target another seat's Tag Filter for get/set/clear | `.identity.filter` |
| 88 | `alert::` | `u64` | `15` | `0` (off), minutes ≥ 1 | Burn-rate alert horizon: warn under the table when a 5h window is forecast to exhaust within N minutes | `.usage` |

*Param 1 = cross-command account selector (no formal group); params 48, 52, 73 = Group 006 Account Targeting; params 49–51 = ungrouped (`.account.renewal`-specific); param 53 = ungrouped (`.account.assign`-specific); param 55 = RETIRED (Feature 035, see row above); param 2 = Output Control group; params 5–18, 28–31 = Field Presence group; params 19–23, 34–36, 54, 60, 80–81 = Fetch Behavior group; param 24 = ungrouped; params 25–27, 32 = Sort Control group; params 33, 37–44, 47, 61, 88 = Display Control group (contains both display-toggle params and pipeline-coupled request-constraint row filters — see Pipeline Stage attribute in each param file); params 64, 66 = ungrouped (`.provider.select`-specific, narrowed Feature 035); param 65 = ungrouped (`.models`-specific); params 69–72, 74 = Group 007 Redirect Backend Config; params 75–79 = ungrouped (`.model`-specific, Feature 035); param 82 = Group 006 Account Targeting (supersedes 52); params 83–84 = ungrouped (`.account.tag`-specific); params 85–87 = ungrouped (`.identity.filter`-specific)*

### See Also

- [../type/](../type/readme.md) — types used by parameters
- [../command/](../command/readme.md) — commands that accept these parameters
- [../param_group/](../param_group/readme.md) — parameter group definitions
- [../user_story/](../user_story/readme.md) — user stories that reference these parameters
- [../command_noun/](../command_noun/readme.md) — domain nouns whose commands accept these parameters
- [../command_verb/](../command_verb/readme.md) — domain verbs that list common parameters
