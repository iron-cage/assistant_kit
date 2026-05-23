# Parameters

All `clp` CLI parameters with type, default, and command coverage.

| File | Responsibility |
|------|----------------|
| [01_name.md](01_name.md) | `name::` — target account identifier |
| [02_format.md](02_format.md) | `format::` / `fmt::` — output serialization format |
| [03_threshold.md](03_threshold.md) | `threshold::` — token expiry warning boundary |
| [04_dry.md](04_dry.md) | `dry::` — dry-run simulation mode |
| [05_account.md](05_account.md) | `account::` — account name field toggle |
| [06_sub.md](06_sub.md) | `sub::` — subscription type field toggle |
| [07_tier.md](07_tier.md) | `tier::` — rate-limit tier field toggle |
| [08_token.md](08_token.md) | `token::` — token status field toggle |
| [09_expires.md](09_expires.md) | `expires::` — token expiry field toggle |
| [10_email.md](10_email.md) | `email::` — email address field toggle |
| [11_file.md](11_file.md) | `file::` — credentials file path field toggle |
| [12_saved.md](12_saved.md) | `saved::` — saved account count field toggle |
| [13_active.md](13_active.md) | `active::` — active/inactive status field toggle |
| [14_display_name.md](14_display_name.md) | `display_name::` — display name field toggle |
| [15_role.md](15_role.md) | `role::` — organisation role field toggle |
| [16_billing.md](16_billing.md) | `billing::` — billing type field toggle |
| [17_model.md](17_model.md) | `model::` — active model field toggle |
| [18_current.md](18_current.md) | `current::` — current live account field toggle |
| [19_refresh.md](19_refresh.md) | `refresh::` — expired token auto-refresh on auth error |
| [20_live.md](20_live.md) | `live::` — continuous quota refresh loop |
| [21_interval.md](21_interval.md) | `interval::` — live mode cycle duration |
| [22_jitter.md](22_jitter.md) | `jitter::` — live mode cycle timing variance |
| [23_trace.md](23_trace.md) | `trace::` — diagnostic trace output to stderr |
| [24_field.md](24_field.md) | `field::` — single-path output selector |

**Total:** 24 parameters

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
| 23 | `trace::` | `bool` | `0` | `0`, `1` | Diagnostic trace output to stderr | 1 cmd |
| 24 | `field::` | `String` | `""` (show all) | `base`, `credentials`, `credential_store`, `projects`, `stats`, `settings`, `session_env`, `sessions` | Single-path output selector | 1 cmd |

*Params 1 = Account Targeting; param 2 = Output Control group; params 5–18 = Field Presence group; params 19–23 = Fetch Behavior group; param 24 = Output Selection group*

### See Also

- [../type/](../type/readme.md) — types used by parameters
- [../command/](../command/readme.md) — commands that accept these parameters
- [../param_group/](../param_group/readme.md) — parameter group definitions
