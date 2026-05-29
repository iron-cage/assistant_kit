# Commands :: Account

Account management commands: list, save, use, delete, limits, and relogin.

---

### Command :: 3. `.accounts`

List all saved accounts or show a single named account with per-field presence control. Without `name::`: shows every account in the credential store as an indented key-val block; with `name::EMAIL`: shows that account's block only.

-- **Parameters:** [`name::`](../param/001_name.md) *(optional)*, [`active::`](../param/013_active.md), [`current::`](../param/018_current.md), [`sub::`](../param/006_sub.md), [`tier::`](../param/007_tier.md), [`expires::`](../param/009_expires.md), [`email::`](../param/010_email.md), [`display_name::`](../param/014_display_name.md), [`role::`](../param/015_role.md), [`billing::`](../param/016_billing.md), [`model::`](../param/017_model.md), [`uuid::`](../param/028_uuid.md), [`capabilities::`](../param/029_capabilities.md), [`org_uuid::`](../param/030_org_uuid.md), [`org_name::`](../param/031_org_name.md), [`format::`](../param/002_format.md), [`trace::`](../param/023_trace.md)
-- **Exit:** 0 (success) | 1 (usage: invalid `name::` chars) | 2 (runtime: account not found or credential store unreadable)

**Syntax:**

```bash
clp .accounts
clp .accounts name::alice@acme.com
clp .accounts alice@acme.com         # positional: same as name::alice@acme.com
clp .accounts car                     # prefix: first saved account starting with "car"
clp .accounts sub::0 tier::0
clp .accounts display_name::1 role::1 billing::1 model::1
clp .accounts format::json
clp .accounts format::table
```

| Parameter | Type | Default | Purpose |
|-----------|------|---------|---------|
| `name::` | [`AccountName`](../type/001_account_name.md) | *(omit to list all)* | Show a single named account instead of listing all |
| `active::` | `bool` | `1` | Show active/inactive status line |
| `current::` | `bool` | `1` | Show current (live) account line; suppressed when `~/.claude/.credentials.json` is unreadable |
| `sub::` | `bool` | `1` | Show subscription type line |
| `tier::` | `bool` | `1` | Show rate-limit tier line |
| `expires::` | `bool` | `1` | Show token expiry duration line |
| `email::` | `bool` | `1` | Show email address from saved `{name}.claude.json` snapshot |
| `display_name::` | `bool` | `0` | Show display name from saved `~/.claude.json` snapshot (opt-in) |
| `role::` | `bool` | `0` | Show organisation role from saved `~/.claude.json` snapshot (opt-in) |
| `billing::` | `bool` | `0` | Show billing type from saved `~/.claude.json` snapshot (opt-in) |
| `model::` | `bool` | `0` | Show active model (always `N/A` for saved accounts — settings.json not captured in snapshot) (opt-in) |
| `uuid::` | `bool` | `0` | Show stable user ID (`taggedId`) from saved `.claude.json` snapshot (opt-in) |
| `capabilities::` | `bool` | `0` | Show product capabilities list from saved `.claude.json` snapshot (opt-in) |
| `org_uuid::` | `bool` | `0` | Show organisation UUID from saved `{name}.roles.json` snapshot (opt-in) |
| `org_name::` | `bool` | `0` | Show organisation display name from saved `{name}.roles.json` snapshot (opt-in) |
| `format::` | [`OutputFormat`](../type/002_output_format.md) | `text` | Output format |
| `trace::` | `bool` | `0` | Print `[trace]` lines to stderr for each credential file read |

**Examples:**

```bash
clp .accounts
# alice@acme.com
#   Active:  yes
#   Current: no
#   Sub:     max
#   Tier:    default_claude_max_20x
#   Expires: in 2h 11m
#   Email:   alice@acme.com
#
# alice@home.com
#   Active:  no
#   Current: yes
#   Sub:     pro
#   Tier:    default_claude_pro
#   Expires: in 5h 30m
#   Email:   N/A

clp .accounts format::table
# Accounts
#
#    Account         Sub   Tier                     Expires     Email
# -  --------------  ----  -----------------------  ----------  ----------------
# ✓  alice@acme.com  max   default_claude_max_20x   in 2h 11m   alice@acme.com
#    alice@home.com  pro   default_claude_pro        in 5h 30m   N/A
```

**Notes:**
- Without `name::`: all accounts listed as indented blocks, separated by blank lines. Empty store → `(no accounts configured)`.
- Field params affect text output only; `format::json` always includes all fields regardless of presence params.
- `format::table` renders a compact one-row-per-account table with fixed columns (flag, Account, Sub, Tier, Expires, Email) — field-presence params are ignored in table mode.
- `current::` shows `Current: yes` for the account whose `accessToken` matches `~/.claude/.credentials.json`. See [feature/016_current_account_awareness.md](../../feature/016_current_account_awareness.md).
- `uuid::`, `capabilities::` source from `{name}.claude.json` snapshot — `N/A` when snapshot absent. See [feature/021_extended_snapshot_fields.md](../../feature/021_extended_snapshot_fields.md).
- `org_uuid::`, `org_name::` source from `{name}.roles.json` snapshot (written by `.account.save` via endpoint 005) — `N/A` when snapshot absent. See [feature/022_org_identity_snapshot.md](../../feature/022_org_identity_snapshot.md).

---

### Command :: 4. `.account.save`

Copies `~/.claude/.credentials.json` to `{credential_store}/{name}.credentials.json` and extracts the `oauthAccount` subtree from `~/.claude.json` into `{name}.claude.json`. Machine-global state (`commands.*`, `mcpServers`, `projects`, `settings.json`) is not captured. Use this to preserve account identity before switching.

-- **Parameters:** [`name::`](../param/001_name.md), [`dry::`](../param/004_dry.md), [`trace::`](../param/023_trace.md)
-- **Exit:** 0 (success) | 1 (usage: invalid name or no active account set) | 2 (runtime: credentials unreadable)

**Syntax:**

```bash
clp .account.save                          # infer name from oauthAccount.emailAddress in ~/.claude.json (falls back to _active_{hostname}_{user})
clp .account.save name::alice@acme.com    # explicit name
clp .account.save name::alice@acme.com dry::1
```

| Parameter | Type | Default | Purpose |
|-----------|------|---------|---------|
| `name::` | [`AccountName`](../type/001_account_name.md) | `auto` (inferred from `oauthAccount.emailAddress` in `~/.claude.json`; falls back to per-machine active marker — see [Feature 025](../../feature/025_per_machine_active_marker.md); exits 1 if neither source present) | Account email to save as |
| `dry::` | `bool` | `0` | Preview action without executing |
| `trace::` | `bool` | `0` | Print `[trace]` lines to stderr for credential read and file write steps |

**Examples:**

```bash
clp .account.save
# saved current credentials as 'alice@acme.com'   (inferred from oauthAccount.emailAddress)

clp .account.save name::alice@acme.com dry::1
# [dry-run] would save current credentials as 'alice@acme.com'
```

**Notes:**
- Also writes `{credential_store}/_active_{hostname}_{user}` = `{name}` on every successful save (per-machine active marker via `active_marker_filename()`).
- Also calls endpoint 005 (`GET /api/oauth/claude_cli/roles`) and writes `{name}.roles.json` (best-effort: failure is silently skipped).
- **Metadata refresh:** Re-running `.account.save` for an existing name refreshes all snapshot files and re-fetches endpoint 005 — this is the canonical way to refresh cached org identity without re-login. `{name}.claude.json` is updated via read-merge (not full overwrite): the `oauthAccount` key is replaced but all other keys (e.g., `_renewal_at` set by `.account.renewal`) are preserved.

---

### Command :: 5. `.account.use`

Atomically overwrites `~/.claude/.credentials.json` with the named account's credentials (write-then-rename), updates the active marker (`_active_{hostname}_{user}`), and best-effort patches `~/.claude.json["oauthAccount"]` from the saved snapshot — preserving all machine-global keys untouched. When `touch::1` (default), fetches quota for the target account and spawns an isolated subprocess to activate its idle 5h session window if `five_hour.resets_at` is absent.

-- **Parameters:** [`name::`](../param/001_name.md) **(required)**, [`dry::`](../param/004_dry.md), [`touch::`](../param/034_touch.md), [`imodel::`](../param/035_imodel.md), [`effort::`](../param/036_effort.md), [`trace::`](../param/023_trace.md)
-- **Exit:** 0 (success) | 1 (usage: invalid name or invalid `imodel::`/`effort::`/`trace::` value) | 2 (runtime: account not found or HOME unset) | 3 (account credentials expired — `touch::1` + fetch failed + `expiresAt` in the past)

**Syntax:**

```bash
clp .account.use name::alice@home.com
clp .account.use alice@home.com               # positional: same as name::alice@home.com
clp .account.use car                           # prefix: first saved account starting with "car"
clp .account.use name::alice@home.com dry::1
clp .account.use name::alice@home.com touch::0
clp .account.use name::alice@home.com imodel::opus effort::max
clp .account.use name::alice@home.com trace::1
```

| Parameter | Type | Default | Purpose |
|-----------|------|---------|---------|
| `name::` | [`AccountName`](../type/001_account_name.md) | **(required)** | Account email to switch to |
| `dry::` | `bool` | `0` | Preview action without executing |
| `touch::` | `bool` | `1` | Activate idle 5h session window via subprocess after switch |
| `imodel::` | `enum` | `auto` | Model for post-switch subprocess: `auto` (sonnet if `7d(Son)≥30%`, else opus), `sonnet`, `opus`, `haiku`, `keep` |
| `effort::` | `enum` | `auto` | Effort for post-switch subprocess: `auto` (high for sonnet, max for opus, none for haiku/keep), `low`, `normal`, `high`, `max` |
| `trace::` | `bool` | `0` | Print `[trace] account.use` lines to stderr: credential read, quota fetch, idle check, model resolution, subprocess dispatch |

**Examples:**

```bash
clp .account.use name::alice@home.com
# switched to 'alice@home.com'   (idle account: subprocess spawned to activate 5h session)

clp .account.use name::alice@home.com touch::0
# switched to 'alice@home.com'   (pure credential rotation — no subprocess)

clp .account.use name::alice@home.com dry::1
# [dry-run] would switch to 'alice@home.com'

clp .account.use name::alice@home.com trace::1
# [trace] account.use  alice@home.com  reading /...alice@home.com.credentials.json
# [trace] account.use  alice@home.com  reading: OK
# [trace] account.use  alice@home.com  quota fetch: OK
# [trace] account.use  alice@home.com  idle check: resets_at=absent → idle
# [trace] account.use  alice@home.com  model: claude-opus-4-6  effort: max
# [trace] account.use  alice@home.com  subprocess: spawned
# switched to 'alice@home.com'
```

**Notes:**
- `touch::1` (default): fetches quota for the target account; if `five_hour.resets_at` is absent (idle), spawns `run_isolated(["--print", "."])` with resolved model/effort to start a 5h session. Quota fetch failure checks `expiresAt` — if locally expired, exits 3 with an error; if not expired, skips touch silently and the switch completes.
- `touch::0`: pure credential rotation — no quota fetch, no subprocess, no expiry check. Pre-Feature-027 behavior.
- `imodel::` and `effort::` follow the same resolution logic as `.usage` (Feature 026): `resolve_model()` selects Sonnet when `7d(Son) ≥ 30%`, Opus otherwise; `resolve_effort()` maps Sonnet → `high`, Opus → `max`, Haiku → no flag. `imodel::haiku` is explicit only — `auto` never selects it.
- `trace::1` only produces output when `touch::1`; with `touch::0` there are no fetch operations to trace.
- See [feature/027_account_use_post_switch_touch.md](../../feature/027_account_use_post_switch_touch.md) for full execution sequence and acceptance criteria.

---

### Command :: 6. `.account.delete`

Removes `{credential_store}/{name}.credentials.json` from the credential store and best-effort removes the accompanying `{name}.claude.json` and `{name}.settings.json` snapshot files.

-- **Parameters:** [`name::`](../param/001_name.md) **(required)**, [`dry::`](../param/004_dry.md), [`trace::`](../param/023_trace.md)
-- **Exit:** 0 (success) | 1 (usage: invalid name) | 2 (runtime: account not found)

**Syntax:**

```bash
clp .account.delete name::alice@oldco.com
clp .account.delete alice@oldco.com         # positional
clp .account.delete car                      # prefix
clp .account.delete name::alice@oldco.com dry::1
```

| Parameter | Type | Default | Purpose |
|-----------|------|---------|---------|
| `name::` | [`AccountName`](../type/001_account_name.md) | **(required)** | Account email to delete |
| `dry::` | `bool` | `0` | Preview action without executing |
| `trace::` | `bool` | `0` | Print `[trace]` lines to stderr for each file removal step |

**Examples:**

```bash
clp .account.delete name::alice@oldco.com
# deleted account 'alice@oldco.com'

clp .account.delete name::alice@oldco.com dry::1
# [dry-run] would delete account 'alice@oldco.com'
```

**Notes:**
- Snapshot files (`{name}.claude.json`, `{name}.settings.json`) are removed best-effort: missing snapshots are silently skipped.
- Deleting the active account also removes the active marker (`_active_{hostname}_{user}`).

---

### Command :: 11. `.account.limits`

Show rate-limit utilization for the active or named account. Displays session (5h) usage, weekly all-model (7d) usage, and rate-limit status with percentage consumed and reset times.

-- **Parameters:** [`name::`](../param/001_name.md) *(optional)*, [`format::`](../param/002_format.md), [`trace::`](../param/023_trace.md)
-- **Exit:** 0 (success) | 1 (usage: invalid `name::` chars) | 2 (runtime: account not found, data unavailable, HOME unset)

**Syntax:**

```bash
clp .account.limits
clp .account.limits name::alice@acme.com
clp .account.limits alice@acme.com       # positional
clp .account.limits car                   # prefix
clp .account.limits format::json
```

| Parameter | Type | Default | Purpose |
|-----------|------|---------|---------|
| `name::` | [`AccountName`](../type/001_account_name.md) | *(omit for active)* | Query a named account instead of the active account |
| `format::` | [`OutputFormat`](../type/002_output_format.md) | `text` | Output format |
| `trace::` | `bool` | `0` | Print `[trace]` lines to stderr for credential store read and API call |

**Examples:**

```bash
clp .account.limits
# Session (5h):   62%  resets in 1h 48m
# Weekly (all):   41%  resets in 3d 12h
# Weekly (sonnet): 38%  resets in 3d 12h

clp .account.limits format::json
# {"session_pct":62,"session_reset_secs":6480,"weekly_all_pct":41,"weekly_all_reset_secs":302400,"weekly_sonnet_pct":38,"weekly_sonnet_reset_secs":302400}
```

**Notes:**
- Data source: `anthropic-ratelimit-unified-*` response headers; transport: `claude_quota::fetch_rate_limits()`. See [feature/013_account_limits.md](../../feature/013_account_limits.md).

---

### Command :: 12. `.account.relogin`

Force browser-based re-authentication for a named account whose `refreshToken` is expired or revoked. This is the recovery path when `refresh::1` silently fails (trace shows `run_isolated: OK credentials=None` — Claude starts but performs no OAuth refresh because the refresh token itself is dead).

-- **Parameters:** [`name::`](../param/001_name.md) *(optional, defaults to active)*, [`dry::`](../param/004_dry.md), [`trace::`](../param/023_trace.md)
-- **Exit:** 0 (success: credentials refreshed and saved) | 1 (usage: invalid name value) | 2 (runtime: name omitted and no active account; account not found; or Claude spawn failed) | 3 (timeout or login abandoned: claude exited without updating credentials)

**Syntax:**

```bash
clp .account.relogin                   # default: active account
clp .account.relogin name::carol@example.com
clp .account.relogin carol@example.com       # positional
clp .account.relogin car               # prefix
clp .account.relogin name::carol@example.com dry::1
clp .account.relogin dry::1            # dry-run for active account
```

| Parameter | Type | Default | Purpose |
|-----------|------|---------|---------|
| `name::` | [`AccountName`](../type/001_account_name.md) | *(active account)* | Account to re-authenticate; omit to use the currently active account |
| `dry::` | `bool` | `0` | Preview the steps without executing |
| `trace::` | `bool` | `0` | Print `[trace]` lines to stderr for each step: store read, switch, spawn, credential change, save, restore |

**Mechanism (6 steps):**
1. Resolve `name::` via [`AccountSelector`](../type/004_account_selector.md) → validate account exists in credential store
2. Snapshot the current active account name (for restoration after login)
3. `switch_account(name)` — makes the named account active in `~/.claude/`
4. Spawn `claude` with inherited TTY (stdin/stdout/stderr connected — NOT isolated subprocess) — Claude detects empty or invalid credentials and opens the browser login page
5. Wait for `claude` to exit; if `~/.claude/.credentials.json` changed → `account::save(name)` propagates fresh credentials to credential store
6. `switch_account(original_active)` — restore the prior active account

**Examples:**

```bash
clp .account.relogin dry::1
# [dry-run] would re-authenticate 'alice@example.com' via browser login

clp .account.relogin name::carol@example.com
# re-authenticated 'carol@example.com' — credentials saved

clp .account.relogin name::carol@example.com dry::1
# [dry-run] would re-authenticate 'carol@example.com' via browser login
```

**Notes:**
- Requires a TTY — `clp .account.relogin` in a piped non-TTY context will fail at step 4 (Claude cannot open a browser or display the login prompt).
- The `claude` subprocess runs with the full inherited environment; no credential isolation (contrast with `refresh::1` which uses an isolated subprocess).
- If `claude` exits without updating `~/.claude/.credentials.json`, the command exits 3. The active account is still restored (step 6 runs regardless of outcome).
- Use this when `clp .usage refresh::1 trace::1` shows `run_isolated: OK credentials=None` for an account — that trace indicates a dead refresh token requiring full browser re-auth.

---

### Command :: 13. `.account.rotate`

Auto-rotate to the best inactive account: selects the saved account with the highest remaining token expiry and atomically switches to it. No account name required — selection is fully automatic via `account::auto_rotate()`.

-- **Parameters:** [`dry::`](../param/004_dry.md), [`trace::`](../param/023_trace.md)
-- **Exit:** 0 (success: switched) | 1 (usage: invalid param) | 2 (runtime: no inactive accounts available, or credential store unreadable)

**Syntax:**

```bash
clp .account.rotate
clp .account.rotate dry::1
```

| Parameter | Type | Default | Purpose |
|-----------|------|---------|---------|
| `dry::` | `bool` | `0` | Preview which account would be selected without switching |
| `trace::` | `bool` | `0` | Print `[trace]` lines to stderr for account selection and switch step |

**Examples:**

```bash
clp .account.rotate
# rotated to 'alice@home.com'

clp .account.rotate dry::1
# [dry-run] would rotate to 'alice@home.com' (expires in 4h 12m, best of 2 inactive)

# When no inactive accounts exist (single account or all accounts = active):
clp .account.rotate
# exit 2: no inactive accounts available
```

**Notes:**
- Selects the inactive account with the highest `expiresAt` value across all saved credential files.
- "Inactive" = any saved account whose name differs from the active marker (`_active_{hostname}_{user}`).
- Equivalent to `clp .account.use $(best_account)` but without requiring the caller to determine the best candidate.
- If only one account is configured, or all saved accounts match the active one, exits 2.
- For explicit selection by name, use [`.account.use`](#command--5-accountuse).

---

### Command :: 14. `.account.renewal`

Set, preview, or clear the billing renewal timestamp override (`_renewal_at`) stored in `{name}.claude.json`. When set, the `.usage` `~Renews` column shows an exact duration (`in Xh Ym`) instead of the estimated `~`-prefixed value derived from `org_created_at`. Supports single account, comma-separated list, or `name::all` to update every saved account in one operation.

-- **Parameters:** [`name::`](../param/001_name.md) **(required)**, [`at::`](../param/049_at.md), [`from_now::`](../param/050_from_now.md), [`clear::`](../param/051_clear.md), [`dry::`](../param/004_dry.md), [`trace::`](../param/023_trace.md)
-- **Exit:** 0 (success) | 1 (usage: no operation provided, conflicting params, or invalid format) | 2 (runtime: account not found or credential store unreadable)

**Syntax:**

```bash
clp .account.renewal name::alice@acme.com at::2026-06-29T21:00:00Z
clp .account.renewal name::alice@acme.com from_now::+1h30m
clp .account.renewal name::alice@acme.com from_now::-30m
clp .account.renewal name::alice@acme.com clear::1
clp .account.renewal name::all from_now::+0m
clp .account.renewal name::alice@acme.com,bob@acme.com at::2026-06-29T21:00:00Z
clp .account.renewal name::alice@acme.com at::2026-06-29T21:00:00Z dry::1
```

| Parameter | Type | Default | Purpose |
|-----------|------|---------|---------|
| `name::` | [`AccountName`](../type/001_account_name.md) or `all` or comma-list | **(required)** | Target account(s): single email/prefix, comma-separated list, or `all` for every saved account |
| `at::` | `string` | *(omit)* | Absolute ISO-8601 UTC renewal timestamp (e.g., `2026-06-29T21:00:00Z`); mutually exclusive with `from_now::` and `clear::` |
| `from_now::` | `string` | *(omit)* | Signed duration delta from now (e.g., `+3h30m`, `-30m`, `+0m`); mutually exclusive with `at::` and `clear::` |
| `clear::` | `bool` | `0` | Remove `_renewal_at` from `{name}.claude.json`; mutually exclusive with `at::` and `from_now::` |
| `dry::` | `bool` | `0` | Preview operation without writing files |
| `trace::` | `bool` | `0` | Print `[trace]` lines to stderr for each file read and write step |

**Examples:**

```bash
clp .account.renewal name::alice@acme.com at::2026-06-29T21:00:00Z
# renewal set for 'alice@acme.com': 2026-06-29T21:00:00Z  (in 30d 14h)

clp .account.renewal name::all from_now::+0m
# renewal set for 'alice@acme.com': 2026-05-29T18:34:22Z  (now)
# renewal set for 'bob@acme.com':   2026-05-29T18:34:22Z  (now)
# renewal set for 'carol@acme.com': 2026-05-29T18:34:22Z  (now)

clp .account.renewal name::alice@acme.com clear::1
# renewal cleared for 'alice@acme.com'  (~Renews will show estimate from org_created_at)

clp .account.renewal name::alice@acme.com at::2026-06-29T21:00:00Z dry::1
# [dry-run] would set renewal for 'alice@acme.com': 2026-06-29T21:00:00Z  (in 30d 14h)
```

**Notes:**
- `_renewal_at` is stored as a top-level key in `{name}.claude.json` alongside `oauthAccount`. It is preserved when `clp .account.save` re-saves that account (read-merge).
- Past `_renewal_at` values are auto-advanced monthly by `.usage` at render time — no need to re-set after each billing cycle.
- `from_now::+0m` sets the override to the current time, which immediately enters the monthly auto-advance cycle.
- `name::all` targets every account in the credential store at the time of execution.
- See [feature/030_account_renewal_override.md](../../feature/030_account_renewal_override.md) for full semantics, `~Renews` rendering rules, and acceptance criteria.
