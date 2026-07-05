# Noun: account

A saved credential profile stored in the per-machine account store (`{credential_store}/{name}.credentials.json` + `{name}.json`). Represents a named Claude Code identity that can be activated, inspected, modified, or removed.

### Commands

| # | Command | Verb | Purpose | Idempotent | Requires Session |
|---|---------|------|---------|-----------|-----------------|
| 1 | `.accounts` | list | Enumerate saved accounts with per-field presence control | Yes | No |
| 2 | `.account.save` | save | Capture current credentials as a named profile | Conditional | Yes |
| 3 | `.account.use` | use | Atomically switch active session to a named account | Conditional | No |
| 4 | `.account.delete` | delete | Remove account profile from credential store | Conditional | No |
| 5 | `.account.limits` | limits | Show rate-limit utilization for named account | Yes | No |
| 6 | `.account.relogin` | relogin | Force browser re-authentication for expired refresh token | No | No |
| 7 | `.account.rotate` | rotate | **DEPRECATED** — hidden redirector; exits 1 with notice to use `.usage rotate::1` | No | No |
| 8 | `.account.renewal` | renewal | Set or clear billing renewal timestamp override | Yes | No |
| 9 | `.account.inspect` | inspect | Live multi-endpoint identity and subscription diagnostic | Yes | No |
| 10 | `.account.assign` *(removed — Feature 037)* | — | **Removed.** Use `.accounts assignee::USER@MACHINE name::X` (Feature 065) | — | — |
| 11 | `.account.unclaim` *(removed — Feature 037; `unclaim::1` also REMOVED — Feature 064)* | — | **Removed.** Use `.accounts owner::0 name::X` (Feature 064) | — | — |

### Parameter Matrix

| Parameter | `.accounts` | `.account.save` | `.account.use` | `.account.delete` | `.account.limits` | `.account.relogin` | `.account.rotate` | `.account.renewal` | `.account.inspect` | `.account.assign` | `.account.unclaim` |
|-----------|------------|----------------|----------------|-------------------|-------------------|--------------------|-------------------|--------------------|--------------------|--------------------|---------------------|
| `name::` | optional | optional | **required** | **required** | optional | optional | — | **required** | optional | optional | **required** |
| `dry::` | — | optional | optional | optional | — | optional | — | optional | — | optional | optional |
| `format::` | optional | — | — | — | optional | — | — | — | optional | — | — |
| `trace::` | optional | optional | optional | optional | optional | optional | — | optional | optional | — | optional |
| `touch::` | — | — | optional | — | — | — | — | — | — | — | — |
| `refresh::` | — | — | optional | — | — | — | — | — | optional | — | — |
| `imodel::` | — | — | optional | — | — | — | — | — | — | — | — |
| `effort::` | — | — | optional | — | — | — | — | — | — | — | — |
| `set_model::` | — | — | optional | — | — | — | — | — | — | — | — |
| `host::` | optional | optional | — | — | — | — | — | — | — | — | — |
| `role::` | optional | optional | — | — | — | — | — | — | — | — | — |
| `at::` | — | — | — | — | — | — | — | optional | — | — | — |
| `from_now::` | — | — | — | — | — | — | — | optional | — | — | — |
| `clear::` | — | — | — | — | — | — | — | optional | — | — | — |
| ~~`for::`~~ | — | — | — | — | — | — | — | — | — | *(REMOVED — Feature 064)* | — |
| Field display params (×13) | optional | — | — | — | — | — | — | — | — | — | — |

Legacy field-toggle params on `.accounts` (×12, all exit 1 with `cols::` migration hint — Feature 037): `current::`, `sub::`, `tier::`, `expires::`, `email::`, `display_name::`, `billing::`, `model::`, `uuid::`, `capabilities::`, `org_uuid::`, `org_name::`. Note: `active::` was repurposed as a mutation param in Feature 064 then REMOVED in Feature 065 — use `assignee::USER@MACHINE` (or `assignee::0` for current machine).

### Lifecycle

```
[absent] --account.save--> [saved]
[saved]  --account.use-->  [active]
[active] --account.use name::other--> [saved]
[saved]  --account.delete--> [absent]
[active] --account.delete--> [absent]
[saved]  --account.relogin--> [saved]
[active] --account.relogin--> [active]
[saved]  --account.renewal--> [saved]
[active] --account.renewal--> [active]
[absent/saved/active] --accounts assignee::USER@MACHINE name::X--> [same state, marker written]
[saved/active]       --accounts owner::0 name::X--> [same state, owner: ""]
```

An account is created by `save`, activated by `use`, and removed by `delete`. The `active` state is machine-scoped: one account is active per `{hostname}_{user}` pair at any time. `relogin` refreshes credentials in-place without changing lifecycle state. `renewal`, `inspect`, and `limits` are non-lifecycle operations (metadata update and read). `.accounts assignee::USER@MACHINE name::X` writes the per-machine marker without changing lifecycle state (ownership-neutral). `.accounts owner::0 name::X` releases ownership without changing lifecycle state.

### Provider Contract

| Operation | Implementation |
|-----------|---------------|
| `.accounts` | `account::list_accounts()` — enumerates `{credential_store}/*.credentials.json` |
| `.account.save` | `account::save_account()` — copies `.credentials.json`, merges `{name}.json` via read-merge |
| `.account.use` | `account::switch_account()` — atomic write-then-rename to `~/.claude/.credentials.json` |
| `.account.delete` | `account::delete_account()` — removes `.credentials.json` + `{name}.json` + legacy files |
| `.account.limits` | `claude_quota::fetch_rate_limits()` — reads `anthropic-ratelimit-unified-*` response headers |
| `.account.relogin` | TTY subprocess sequence: switch → spawn `claude` → detect credential change → save → restore |
| `.account.rotate` | **DEPRECATED** — redirector prints deprecation notice, exits 1; rotation moved to `.usage rotate::1` |
| `.account.renewal` | `account::set_renewal_at()` — read-merge write to `{name}.json` `_renewal_at` key |
| `.account.inspect` | Endpoints 002/005/001 — `fetch_oauth_account()`, `fetch_claude_cli_roles()`, `fetch_oauth_usage()` |
| `.account.assign` *(removed Feature 037)* | Use `.accounts assignee::USER@MACHINE name::X` (Feature 065) → `account::write_active_marker()` |
| `.account.unclaim` *(removed Feature 037; `unclaim::1` REMOVED Feature 064)* | Use `.accounts owner::0 name::X` (Feature 064) → `account::write_owner()` |

### Output Schema

**`.accounts format::json` (array of account objects):**

```json
[
  {
    "name": "alice@acme.com",
    "active": true,
    "current": false,
    "sub": "max",
    "tier": "default_claude_max_20x",
    "expires_in_secs": 7860,
    "email": "alice@acme.com"
  }
]
```

**`.account.inspect format::json` (single account with memberships):**

```json
{
  "account": "alice@acme.com",
  "status": "valid",
  "expires_in_secs": 13920,
  "tagged_id": "user_01ABCdef",
  "uuid": "aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee",
  "email_address": "alice@acme.com",
  "full_name": "Alice",
  "display_name": "Alice",
  "memberships": [
    { "index": 0, "billing_type": "none", "has_max": false, "capabilities": ["chat"], "selected": false },
    { "index": 1, "billing_type": "stripe_subscription", "has_max": true, "capabilities": ["claude_max", "chat"], "selected": true }
  ],
  "billing_type": "stripe_subscription",
  "has_max": true,
  "capabilities": ["claude_max", "chat"],
  "rate_limit_tier": "default_claude_max_20x",
  "session_5h_pct": 45,
  "session_5h_reset_ts": 1750089000,
  "weekly_7d_pct": 33,
  "weekly_7d_reset_ts": 1750180000,
  "sonnet_7d_pct": 53,
  "sonnet_7d_reset_ts": 1750180000,
  "organization_name": "alice@acme.com's Organization",
  "organization_uuid": "aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee",
  "organization_role": "admin",
  "workspace_uuid": "",
  "workspace_name": "",
  "data_source": "live"
}
```

### Error Codes

| Code | Trigger | Recovery |
|------|---------|---------|
| exit 1 | Invalid `name::` characters (`/`, `\`, `*`) or missing `@` in email | Use a valid email address |
| exit 1 | `name::` prefix matches multiple accounts (ambiguous) | Use full email address |
| exit 1 | `.account.renewal` called without `at::`, `from_now::`, or `clear::` | Provide one operation parameter |
| exit 1 | Ownership violation on `.account.use`, `.account.delete`, `.account.relogin`, `.accounts owner::0`, or `.accounts owner::USER@MACHINE` | Run from the owning machine; or `.accounts owner::0 name::X` from the owner first; `force::1` bypasses G8 |
| exit 2 | Account not found in credential store | Run `.account.save` to create; check `name::` spelling |
| exit 2 | Credential store unreadable or `$HOME` unset | Check `$HOME` env and file permissions |
| exit 2 | No active account for commands that default to active | Run `.account.use name::EMAIL` first |
| exit 3 | Token expired + refresh failed + `refresh::1` (`.account.use`, `.account.inspect`) | Run `.account.relogin name::EMAIL` |
| exit 3 | `.account.relogin` timed out or browser abandoned | Retry; verify Claude binary is on `$PATH` |

### Relationships

| Related Entity | Relationship | Direction |
|----------------|-------------|---------|
| `token` | Account contains an OAuth access token; token state derives from account credentials | account → token |
| `credentials` | Live session credentials reflect the active account; independent read path | account → credentials |

### See Also

| File | Relationship |
|------|-------------|
| [feature/001_account_store_init.md](../../feature/001_account_store_init.md) | Credential store initialization before first save |
| [feature/002_account_save.md](../../feature/002_account_save.md) | Save algorithm and `{name}.json` read-merge semantics |
| [feature/004_account_use.md](../../feature/004_account_use.md) | Atomic switch algorithm and active marker update |
| [feature/005_account_delete.md](../../feature/005_account_delete.md) | File removal sequence and legacy satellite cleanup |
| [feature/025_per_machine_active_marker.md](../../feature/025_per_machine_active_marker.md) | `_active_{machine}_{user}` marker semantics |
| [feature/030_account_renewal_override.md](../../feature/030_account_renewal_override.md) | `_renewal_at` storage, monthly auto-advance, and `~Renews` rendering |
| [feature/031_account_inspect.md](../../feature/031_account_inspect.md) | Multi-endpoint inspection with membership selection priority |
| [feature/032_account_assign.md](../../feature/032_account_assign.md) | Marker-only write semantics and `for::` resolution |
| [feature/036_account_ownership.md](../../feature/036_account_ownership.md) | Ownership model; G1–G8 enforcement gates; `.account.unclaim` design |

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | [`.accounts`](../command/001_account.md#command-3-accounts) | List all saved accounts |
| 2 | [`.account.save`](../command/001_account.md#command-4-accountsave) | Save current credentials as named profile |
| 3 | [`.account.use`](../command/001_account.md#command-5-accountuse) | Switch active account |
| 4 | [`.account.delete`](../command/001_account.md#command-6-accountdelete) | Delete account from store |
| 5 | [`.account.limits`](../command/001_account.md#command-11-accountlimits) | Show rate-limit utilization |
| 6 | [`.account.relogin`](../command/001_account.md#command-12-accountrelogin) | Browser re-authentication |
| 7 | [`.account.rotate`](../command/001_account.md#command-13-accountrotate-deprecated-feature-038) | **DEPRECATED** — redirector; use `.usage rotate::1` |
| 8 | [`.account.renewal`](../command/001_account.md#command-14-accountrenewal) | Set or clear billing renewal override |
| 9 | [`.account.inspect`](../command/001_account.md#command-15-accountinspect) | Live identity and subscription diagnostic |
| 10 | [`.account.assign`](../command/001_account.md#command-16-accountassign-removed-feature-037-migration-path-superseded-feature-064065) *(removed Feature 037)* | Use `.accounts assignee::USER@MACHINE name::X` (Feature 065) |
| 11 | [`.account.unclaim`](../command/001_account.md#command-17-accountunclaim-removed-feature-037-migration-path-superseded-feature-064) *(removed Feature 037; `unclaim::1` REMOVED Feature 064)* | Use `.accounts owner::0 name::X` (Feature 064) |

### Referenced Parameters

| # | Parameter | Role |
|---|-----------|------|
| 1 | [`name::`](../param/001_name.md) | Account identifier (email or prefix) |
| 2 | [`format::`](../param/002_format.md) | Output serialization format |
| 3 | [`dry::`](../param/004_dry.md) | Preview mutation without writing |
| 4 | [`trace::`](../param/023_trace.md) | Diagnostic trace output |
| 5 | [`refresh::`](../param/019_refresh.md) | Force token refresh before operation |
| 6 | [`touch::`](../param/034_touch.md) | Activate idle 5h window after use |
| 7 | [`imodel::`](../param/035_imodel.md) | Model for post-switch activation subprocess |
| 8 | [`effort::`](../param/036_effort.md) | Effort for post-switch activation subprocess |
| 9 | [`set_model::`](../param/054_set_model.md) | Set session model after switch |
| 10 | [`host::`](../param/048_host.md) | Machine label for save/display |
| 11 | [`role::`](../param/052_role.md) | User-defined role label for save/display |
| 12 | [`at::`](../param/049_at.md) | Absolute billing renewal timestamp override |
| 13 | [`from_now::`](../param/050_from_now.md) | Delta-from-now renewal timestamp |
| 14 | [`clear::`](../param/051_clear.md) | Remove `_renewal_at` override |
| 15 | [`force::`](../param/058_force.md) | Bypass G5–G8 ownership gates |
| 16 | [`owner::`](../param/062_owner.md) | Set or release account ownership |
| 17 | [`assignee::`](../param/063_assignee.md) | Write per-machine active-account marker |
| 18 | [`sort::`](../param/025_sort.md) | Row ordering strategy |
| 19 | [`desc::`](../param/026_desc.md) | Reverse sort direction |
| 20 | [`prefer::`](../param/027_prefer.md) | Tiebreaker sort strategy |
| 21 | [`cols::`](../param/033_cols.md) | Column set modifiers for quota table |
| 22 | [`count::`](../param/037_count.md) | Limit row count after filtering |
| 23 | [`offset::`](../param/038_offset.md) | Skip first N rows |
| 24 | [`only_active::`](../param/039_only_active.md) | Keep only currently active account row |
| 25 | [`only_next::`](../param/040_only_next.md) | Keep only recommended next account row |
| 26 | [`min_5h::`](../param/041_min_5h.md) | Keep only rows with 5h quota ≥ N% |
| 27 | [`min_7d::`](../param/042_min_7d.md) | Keep only rows with 7d quota ≥ N% |
| 28 | [`only_valid::`](../param/043_only_valid.md) | Keep only non-exhausted non-expired rows |
| 29 | [`exclude_exhausted::`](../param/044_exclude_exhausted.md) | Remove exhausted rows |
| 30 | [`abs::`](../param/046_abs.md) | Show absolute token counts |
| 31 | [`no_color::`](../param/047_no_color.md) | Strip emoji and ANSI sequences |
| 32 | [`get::`](../param/045_get.md) | Extract bare field value from first row |
| 33 | [`live::`](../param/020_live.md) | Continuous monitor mode |
| 34 | [`interval::`](../param/021_interval.md) | Seconds between live refresh cycles |
| 35 | [`jitter::`](../param/022_jitter.md) | Random jitter added to interval |
