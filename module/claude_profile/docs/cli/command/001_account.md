# Commands :: Account

Account management commands: list, save, use, delete, limits, and relogin.

---

### Command :: 3. `.accounts`

List all saved accounts or show a single named account with per-field presence control. Without `name::`: shows every account in the credential store as an indented key-val block; with `name::EMAIL`: shows that account's block only.

-- **Parameters:** [`name::`](../param/001_name.md) *(optional)*, [`active::`](../param/013_active.md), [`current::`](../param/018_current.md), [`sub::`](../param/006_sub.md), [`tier::`](../param/007_tier.md), [`expires::`](../param/009_expires.md), [`email::`](../param/010_email.md), [`display_name::`](../param/014_display_name.md), [`host::`](../param/048_host.md), [`role::`](../param/015_role.md), [`billing::`](../param/016_billing.md), [`model::`](../param/017_model.md), [`uuid::`](../param/028_uuid.md), [`capabilities::`](../param/029_capabilities.md), [`org_uuid::`](../param/030_org_uuid.md), [`org_name::`](../param/031_org_name.md), [`format::`](../param/002_format.md), [`trace::`](../param/023_trace.md)
-- **Exit:** 0 (success) | 1 (usage: invalid `name::` chars) | 2 (runtime: account not found or credential store unreadable)

**Syntax:**

```bash
clp .accounts
clp .accounts name::alice@acme.com
clp .accounts alice@acme.com         # positional: bare name at any position
clp .accounts sub::0 alice@acme.com  # reversed: arg order does not matter
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
| `email::` | `bool` | `1` | Show email address from saved `{name}.json` snapshot |
| `display_name::` | `bool` | `0` | Show display name from saved `{name}.json` snapshot (opt-in) |
| `host::` | `bool` | `0` | Show host/machine label from saved `{name}.json` (opt-in) |
| `role::` | `bool` | `0` | Show user-defined role label from saved `{name}.json` (opt-in) |
| `billing::` | `bool` | `0` | Show billing type from saved `{name}.json` snapshot (opt-in) |
| `model::` | `bool` | `0` | Show active model from saved `{name}.json` snapshot (opt-in) |
| `uuid::` | `bool` | `0` | Show stable user ID (`taggedId`) from saved `{name}.json` snapshot (opt-in) |
| `capabilities::` | `bool` | `0` | Show product capabilities list from saved `{name}.json` snapshot (opt-in) |
| `org_uuid::` | `bool` | `0` | Show organisation UUID from saved `{name}.json` snapshot (opt-in) |
| `org_name::` | `bool` | `0` | Show organisation display name from saved `{name}.json` snapshot (opt-in) |
| `format::` | [`OutputFormat`](../type/002_output_format.md) | `text` | Output format |
| `trace::` | `bool` | `0` | Print `[trace]` lines to stderr for each credential file read |

**Algorithm (4 steps):**
1. Enumerate `{credential_store}/*.credentials.json` alphabetically; build account list
2. `(when name:: provided)` Resolve via `AccountSelector`; filter list to single match
3. For each matched account: read credential JSON + `_active_{hostname}_{user}` marker + snapshot files per enabled field params
4. Render in requested `format::`

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
- `host::`, `role::` source from `{name}.json` (written by `.account.save host::` / `role::`) — `N/A` when no metadata exists. See [feature/029_account_host_metadata.md](../../feature/029_account_host_metadata.md).
- `uuid::`, `capabilities::` source from `{name}.json` snapshot — `N/A` when snapshot absent. See [feature/021_extended_snapshot_fields.md](../../feature/021_extended_snapshot_fields.md).
- `org_uuid::`, `org_name::` source from `{name}.json` snapshot (written by `.account.save` via endpoint 005) — `N/A` when snapshot absent. See [feature/022_org_identity_snapshot.md](../../feature/022_org_identity_snapshot.md).

### Referenced Features

| # | Feature | Role |
|---|---------|------|
| 1 | [Account List](../../feature/003_account_list.md) | Account enumeration and per-account block rendering |
| 2 | [Rich Account Metadata](../../feature/014_rich_account_metadata.md) | Extended metadata fields shown per account |
| 3 | [Name Shortcut Syntax](../../feature/015_name_shortcut_syntax.md) | Prefix and positional `name::` resolution |
| 4 | [Current Account Awareness](../../feature/016_current_account_awareness.md) | `current::` field and `✓` flag column |
| 5 | [Extended Snapshot Fields](../../feature/021_extended_snapshot_fields.md) | Opt-in snapshot fields (`uuid::`, `capabilities::`) |
| 6 | [Org Identity Snapshot](../../feature/022_org_identity_snapshot.md) | Org identity fields (`org_uuid::`, `org_name::`) |
| 7 | [Host Metadata](../../feature/029_account_host_metadata.md) | `host::` and `role::` fields from saved snapshot |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 1 | [Account Rotation](../user_story/001_account_rotation.md) | Inspect accounts before and after rotation |
| 2 | [Account Onboarding](../user_story/002_onboarding.md) | Verify saved account metadata during onboarding |

---

### Command :: 4. `.account.save`

Copies `~/.claude/.credentials.json` to `{credential_store}/{name}.credentials.json` and merges identity, model, roles, and profile metadata into the unified `{name}.json`. Machine-global state (`commands.*`, `mcpServers`, `projects`) is not captured. Use this to preserve account identity before switching.

-- **Parameters:** [`name::`](../param/001_name.md), [`dry::`](../param/004_dry.md), [`host::`](../param/048_host.md), [`role::`](../param/052_role.md), [`trace::`](../param/023_trace.md)
-- **Exit:** 0 (success) | 1 (usage: invalid name or no active account set) | 2 (runtime: credentials unreadable)

**Syntax:**

```bash
clp .account.save                                      # infer name from oauthAccount.emailAddress in ~/.claude.json (falls back to _active_{hostname}_{user})
clp .account.save name::alice@acme.com                # explicit name
clp .account.save name::alice@acme.com dry::1
clp .account.save host::workstation                   # store host label in {name}.json
clp .account.save role::work                          # store role label in {name}.json
clp .account.save host::workstation role::personal    # both metadata fields
```

| Parameter | Type | Default | Purpose |
|-----------|------|---------|---------|
| `name::` | [`AccountName`](../type/001_account_name.md) | `auto` (inferred from `oauthAccount.emailAddress` in `~/.claude.json`; falls back to per-machine active marker — see [Feature 025](../../feature/025_per_machine_active_marker.md); exits 1 if neither source present) | Account email to save as |
| `dry::` | `bool` | `0` | Preview action without executing |
| `host::` | `string` | `""` (auto-detected hostname) | Machine/host label stored in `{name}.json` (see [feature/029](../../feature/029_account_host_metadata.md)) |
| `role::` | `string` | `""` | User-defined role label stored in `{name}.json` (see [param 052](../param/052_role.md)) |
| `trace::` | `bool` | `0` | Print `[trace]` lines to stderr for credential read and file write steps |

**Algorithm (5 steps):**
1. Resolve `name::`: read `oauthAccount.emailAddress` from `~/.claude.json`; fall back to `_active_{hostname}_{user}` marker; exit 1 if neither present
2. `(when dry::0)` Copy `~/.claude/.credentials.json` → `{name}.credentials.json` (atomic write)
3. `(when dry::0)` Read `~/.claude.json` + `~/.claude/settings.json` + call `GET /api/oauth/claude_cli/roles` (best-effort); merge all into unified `{name}.json` (preserves `_renewal_at` and other keys)
4. `(when dry::0)` Write host, role, and owner into `{name}.json`: `host::` (auto-captured `$USER@$HOSTNAME` when omitted); `role::` via read-merge; `owner` = `current_identity()` always
5. `(when dry::0)` Write `_active_{hostname}_{user}` = `{name}` (per-machine active marker)

**Examples:**

```bash
clp .account.save
# saved current credentials as 'alice@acme.com'   (inferred from oauthAccount.emailAddress)

clp .account.save name::alice@acme.com dry::1
# [dry-run] would save current credentials as 'alice@acme.com'

clp .account.save host::workstation role::work
# saved current credentials as 'alice@acme.com'   (host='workstation', role='work')
```

**Notes:**
- Also writes `{credential_store}/_active_{hostname}_{user}` = `{name}` on every successful save (per-machine active marker via `active_marker_filename()`).
- Also calls endpoint 005 (`GET /api/oauth/claude_cli/roles`) and merges result into `{name}.json` (best-effort: failure is silently skipped).
- **Metadata refresh:** Re-running `.account.save` for an existing name refreshes the unified `{name}.json` and re-fetches endpoint 005 — this is the canonical way to refresh cached org identity without re-login. `{name}.json` is updated via read-merge (not full overwrite): the `oauthAccount` key is replaced but all other keys (e.g., `_renewal_at` set by `.account.renewal`) are preserved.
- **Ownership stamp:** `.account.save` always writes `current_identity()` as `owner` in `{name}.json` on every interactive save. Background refresh callers pass `owner: None`. To release ownership, use `clp .account.unclaim name::EMAIL` — calls `write_owner(name, store, "")` directly without touching credentials. See [Feature 036](../../feature/036_account_ownership.md).

### Referenced Features

| # | Feature | Role |
|---|---------|------|
| 1 | [Store Init](../../feature/001_store_init.md) | Credential store initialization before save |
| 2 | [Save Account](../../feature/002_account_save.md) | Core save algorithm and file layout |
| 3 | [Persistent Storage](../../feature/010_persistent_storage.md) | Unified `{name}.json` merge semantics |
| 4 | [Per-Machine Active Marker](../../feature/025_per_machine_active_marker.md) | `_active_{hostname}_{user}` marker written on save |
| 5 | [Host Metadata](../../feature/029_account_host_metadata.md) | `host::` and `role::` metadata stored in `{name}.json` |
| 6 | [Account Ownership](../../feature/036_account_ownership.md) | Ownership model — `.account.save` stamps `current_identity()`; `.account.unclaim` clears ownership; `.account.assign` is marker-only |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 1 | [Account Onboarding](../user_story/002_onboarding.md) | Saving credentials during initial account setup |

---

### Command :: 5. `.account.use`

Atomically overwrites `~/.claude/.credentials.json` with the named account's credentials (write-then-rename), updates the active marker (`_active_{hostname}_{user}`), and best-effort patches `~/.claude.json["oauthAccount"]` from the saved snapshot — preserving all machine-global keys untouched. When `touch::1` (default), fetches quota for the target account and spawns an isolated subprocess to activate its idle 5h session window if `five_hour.resets_at` is absent.

-- **Parameters:** [`name::`](../param/001_name.md) **(required)**, [`dry::`](../param/004_dry.md), [`touch::`](../param/034_touch.md), [`refresh::`](../param/019_refresh.md), [`imodel::`](../param/035_imodel.md), [`effort::`](../param/036_effort.md), [`trace::`](../param/023_trace.md), [`set_model::`](../param/054_set_model.md)
-- **Exit:** 0 (success) | 1 (usage: invalid name or invalid `imodel::`/`effort::`/`trace::`/`set_model::` value) | 2 (runtime: account not found or HOME unset) | 3 (account credentials expired — `touch::1` + fetch failed + `expiresAt` in the past, AND refresh failed or `refresh::0`)

**Syntax:**

```bash
clp .account.use name::alice@home.com
clp .account.use alice@home.com               # positional: bare name at any position
clp .account.use dry::1 alice@home.com        # reversed: arg order does not matter
clp .account.use car                           # prefix: first saved account starting with "car"
clp .account.use name::alice@home.com dry::1
clp .account.use name::alice@home.com touch::0
clp .account.use name::alice@home.com refresh::0
clp .account.use name::alice@home.com imodel::opus effort::max
clp .account.use name::alice@home.com trace::1
clp .account.use name::alice@home.com set_model::opus
clp .account.use name::alice@home.com set_model::default
```

| Parameter | Type | Default | Purpose |
|-----------|------|---------|---------|
| `name::` | [`AccountName`](../type/001_account_name.md) | **(required)** | Account email to switch to |
| `dry::` | `bool` | `0` | Preview action without executing |
| `touch::` | `bool` | `1` | Activate idle 5h session window via subprocess after switch |
| `refresh::` | `bool` | `1` | Attempt OAuth token refresh when locally expired before refusing with exit 3 |
| `imodel::` | `enum` | `auto` | Model for post-switch subprocess: `auto` (haiku — sufficient for keep-alive), `sonnet`, `opus`, `haiku`, `keep` |
| `effort::` | `enum` | `auto` | Effort for post-switch subprocess: `auto` (`low` for any model; no flag for haiku/keep), `low`, `normal`, `high`, `max` |
| `trace::` | `bool` | `0` | Print `[trace] account.use` lines to stderr: credential read, quota fetch, idle check, model resolution, subprocess dispatch |
| `set_model::` | `enum` | *(omit)* | Explicitly write session model to `settings.json`: `opus` (`claude-opus-4-6`), `sonnet` (`claude-sonnet-4-6`), `haiku` (`claude-haiku-4-5-20251001`), `default` (removes override); takes precedence over automatic `apply_model_override()` |

**Algorithm (7 steps):**
1. Resolve `name::` via `AccountSelector`; load `{name}.credentials.json`
2. `(when dry::0)` Atomically overwrite `~/.claude/.credentials.json` via write-then-rename
3. `(when dry::0)` Write `_active_{hostname}_{user}` = `{name}` (active marker)
4. `(when dry::0)` Best-effort patch `~/.claude.json["oauthAccount"]` from saved snapshot (preserves machine-global keys)
5. `(when touch::1)` Fetch quota via `GET /api/oauth/usage`; `(when refresh::1 + locally expired)` call `refresh_account_token()` first; evaluate idle: `five_hour.resets_at` absent → idle
6. `(when touch::1 + idle)` Resolve model+effort via `resolve_model()`/`resolve_effort()`; spawn isolated subprocess via `run_isolated()`
7. Session-model override: `(when set_model:: provided)` write requested model via `set_session_model()`; `(otherwise, when target was already active + valid quota)` write resolved model via `apply_model_override()`

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
# [trace] account.use  alice@home.com  subprocess: scheduled (idle check removed)
# [trace] account.use  alice@home.com  model: claude-opus-4-6  effort: low
# [trace] account.use  alice@home.com  subprocess: spawned
# switched to 'alice@home.com'
```

**Notes:**
- `touch::1` (default): fetches quota for the target account; when fetch succeeds, always spawns `run_isolated(["--print", "."])` with resolved model/effort (subprocess is idempotent; Fix(BUG-285): idle check removed). Quota fetch failure checks `expiresAt` — if locally expired and `refresh::1` (default), attempts token refresh and re-probes touch context on success; exits 3 if refresh fails. If locally expired and `refresh::0`, exits 3 immediately. If `expiresAt` is absent or not yet expired, skips touch silently and the switch completes.
- `touch::0`: pure credential rotation — no quota fetch, no subprocess, no expiry check. Pre-Feature-027 behavior.
- `imodel::` and `effort::` follow the same resolution logic as `.usage` (Feature 026): `imodel::auto` always selects Haiku (sufficient for keep-alive pings); `resolve_effort()` maps Haiku and keep → no `--effort` flag, other models → `low`. See [feature/026](../../feature/026_subprocess_model_effort.md).
- `set_model::`: when provided, `set_session_model()` writes the requested model to `settings.json` last (after any `apply_post_switch_touch()` or `apply_model_override()`), ensuring it takes precedence. `default` removes the `model` key entirely.
- `trace::1` only produces output when `touch::1`; with `touch::0` there are no fetch operations to trace.
- See [feature/027_account_use_post_switch_touch.md](../../feature/027_account_use_post_switch_touch.md) for full execution sequence and acceptance criteria.

### Referenced Features

| # | Feature | Role |
|---|---------|------|
| 1 | [Switch Account](../../feature/004_account_use.md) | Atomic credential rotation and active marker update |
| 2 | [Token Refresh](../../feature/017_token_refresh.md) | Pre-switch refresh on locally-expired token |
| 3 | [Session Touch](../../feature/024_session_touch.md) | Idle 5h window activation after switch |
| 4 | [Subprocess Model/Effort](../../feature/026_subprocess_model_effort.md) | Model and effort selection for post-switch subprocess |
| 5 | [Post-Switch Touch](../../feature/027_account_use_post_switch_touch.md) | Full execution sequence with touch and model override |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 1 | [Account Rotation](../user_story/001_account_rotation.md) | Primary command for switching to a named account |

---

### Command :: 6. `.account.delete`

Removes `{credential_store}/{name}.credentials.json` and `{name}.json` from the credential store, plus any legacy satellite files from pre-consolidation layout.

-- **Parameters:** [`name::`](../param/001_name.md) **(required)**, [`dry::`](../param/004_dry.md), [`trace::`](../param/023_trace.md)
-- **Exit:** 0 (success) | 1 (usage: invalid name) | 2 (runtime: account not found)

**Syntax:**

```bash
clp .account.delete name::alice@oldco.com
clp .account.delete alice@oldco.com          # positional: bare name at any position
clp .account.delete dry::1 alice@oldco.com   # reversed: arg order does not matter
clp .account.delete car                      # prefix
clp .account.delete name::alice@oldco.com dry::1
```

| Parameter | Type | Default | Purpose |
|-----------|------|---------|---------|
| `name::` | [`AccountName`](../type/001_account_name.md) | **(required)** | Account email to delete |
| `dry::` | `bool` | `0` | Preview action without executing |
| `trace::` | `bool` | `0` | Print `[trace]` lines to stderr for each file removal step |

**Algorithm (4 steps):**
1. Resolve `name::` via `AccountSelector`; validate account exists in credential store
2. `(when dry::0)` Delete `{name}.credentials.json`
3. `(when dry::0)` Best-effort delete `{name}.json` + legacy files (`.claude.json`, `.settings.json`, `.roles.json`, `.profile.json`; skip missing)
4. `(when dry::0 + deleted account = active)` Delete `_active_{hostname}_{user}` marker

**Examples:**

```bash
clp .account.delete name::alice@oldco.com
# deleted account 'alice@oldco.com'

clp .account.delete name::alice@oldco.com dry::1
# [dry-run] would delete account 'alice@oldco.com'
```

**Notes:**
- Metadata file (`{name}.json`) and legacy satellite files are removed best-effort: missing files are silently skipped.
- Deleting the active account also removes the active marker (`_active_{hostname}_{user}`).

### Referenced Features

| # | Feature | Role |
|---|---------|------|
| 1 | [Delete Account](../../feature/005_account_delete.md) | File removal sequence and legacy satellite cleanup |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 1 | [Account Onboarding](../user_story/002_onboarding.md) | Removing stale accounts during account management |

---

### Command :: 11. `.account.limits`

Show rate-limit utilization for the active or named account. Displays session (5h) usage, weekly all-model (7d) usage, and rate-limit status with percentage consumed and reset times.

-- **Parameters:** [`name::`](../param/001_name.md) *(optional)*, [`format::`](../param/002_format.md), [`trace::`](../param/023_trace.md)
-- **Exit:** 0 (success) | 1 (usage: invalid `name::` chars) | 2 (runtime: account not found, data unavailable, HOME unset)

**Syntax:**

```bash
clp .account.limits
clp .account.limits name::alice@acme.com
clp .account.limits alice@acme.com            # positional: bare name at any position
clp .account.limits format::json alice@acme.com  # reversed: arg order does not matter
clp .account.limits car                   # prefix
clp .account.limits format::json
```

| Parameter | Type | Default | Purpose |
|-----------|------|---------|---------|
| `name::` | [`AccountName`](../type/001_account_name.md) | *(omit for active)* | Query a named account instead of the active account |
| `format::` | [`OutputFormat`](../type/002_output_format.md) | `text` | Output format |
| `trace::` | `bool` | `0` | Print `[trace]` lines to stderr for credential store read and API call |

**Algorithm (3 steps):**
1. Resolve `name::` (omit → active account from `_active_{hostname}_{user}` marker); load credentials
2. Fetch rate-limit headers via `fetch_rate_limits()` (`anthropic-ratelimit-unified-*` response headers)
3. Render session (5h), weekly all-model (7d), and weekly sonnet utilization in requested `format::`

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

### Referenced Features

| # | Feature | Role |
|---|---------|------|
| 1 | [Account Limits](../../feature/013_account_limits.md) | Rate-limit header parsing and utilization rendering |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 1 | [Multi-Account Quota Monitoring](../user_story/003_quota_monitoring.md) | Per-account rate-limit utilization check |

---

### Command :: 12. `.account.relogin`

Force browser-based re-authentication for a named account whose `refreshToken` is expired or revoked. This is the recovery path when `refresh::1` silently fails (trace shows `run_isolated: OK credentials=None` — Claude starts but performs no OAuth refresh because the refresh token itself is dead).

-- **Parameters:** [`name::`](../param/001_name.md) *(optional, defaults to active)*, [`dry::`](../param/004_dry.md), [`trace::`](../param/023_trace.md)
-- **Exit:** 0 (success: credentials refreshed and saved) | 1 (usage: invalid name value) | 2 (runtime: name omitted and no active account; account not found; or Claude spawn failed) | 3 (timeout or login abandoned: claude exited without updating credentials)

**Syntax:**

```bash
clp .account.relogin                   # default: active account
clp .account.relogin name::carol@example.com
clp .account.relogin carol@example.com          # positional: bare name at any position
clp .account.relogin dry::1 carol@example.com   # reversed: arg order does not matter
clp .account.relogin car               # prefix
clp .account.relogin name::carol@example.com dry::1
clp .account.relogin dry::1            # dry-run for active account
```

| Parameter | Type | Default | Purpose |
|-----------|------|---------|---------|
| `name::` | [`AccountName`](../type/001_account_name.md) | *(active account)* | Account to re-authenticate; omit to use the currently active account |
| `dry::` | `bool` | `0` | Preview the steps without executing |
| `trace::` | `bool` | `0` | Print `[trace]` lines to stderr for each step: store read, switch, spawn, credential change, save, restore |

**Algorithm (6 steps):**
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

### Referenced Features

| # | Feature | Role |
|---|---------|------|
| 1 | [Auto Rotate](../../feature/008_auto_rotate.md) | Relogin as recovery path for dead refresh tokens |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 1 | [Account Onboarding](../user_story/002_onboarding.md) | Re-authenticating an account with expired refresh token |

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

**Algorithm (4 steps):**
1. Enumerate all saved accounts; read `_active_{hostname}_{user}` marker
2. Filter to inactive accounts (name ≠ active marker value); exit 2 if none
3. Select account with highest `expiresAt` value (`account::auto_rotate()`)
4. `(when dry::0)` Execute `.account.use` steps for selected account

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

### Referenced Features

| # | Feature | Role |
|---|---------|------|
| 1 | [Auto Rotate](../../feature/008_auto_rotate.md) | Automatic best-account selection algorithm |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 1 | [Account Rotation](../user_story/001_account_rotation.md) | Automatic rotation to the account with most remaining token life |

---

### Command :: 14. `.account.renewal`

Set, preview, or clear the billing renewal timestamp override (`_renewal_at`) stored in `{name}.json`. When set, the `.usage` `~Renews` column shows an exact duration (`in Xh Ym`) instead of the estimated `~`-prefixed value derived from `org_created_at`. Supports single account, comma-separated list, or `name::all` to update every saved account in one operation.

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
| `clear::` | `bool` | `0` | Remove `_renewal_at` from `{name}.json`; mutually exclusive with `at::` and `from_now::` |
| `dry::` | `bool` | `0` | Preview operation without writing files |
| `trace::` | `bool` | `0` | Print `[trace]` lines to stderr for each file read and write step |

**Algorithm (4 steps):**
1. Resolve target account list from `name::`: single email/prefix, comma-separated list, or `all` (every saved account)
2. For each target: read `{name}.json`
3. Compute new `_renewal_at` value from `at::` (absolute ISO-8601), `from_now::` (signed delta), or `clear::1` (remove key)
4. `(when dry::0)` Write `{name}.json` with updated `_renewal_at` key per account

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
- `_renewal_at` is stored as a top-level key in `{name}.json` alongside `oauthAccount`. It is preserved when `clp .account.save` re-saves that account (read-merge).
- Past `_renewal_at` values are auto-advanced monthly by `.usage` at render time — no need to re-set after each billing cycle.
- `from_now::+0m` sets the override to the current time, which immediately enters the monthly auto-advance cycle.
- `name::all` targets every account in the credential store at the time of execution.
- See [feature/030_account_renewal_override.md](../../feature/030_account_renewal_override.md) for full semantics, `~Renews` rendering rules, and acceptance criteria.

### Referenced Features

| # | Feature | Role |
|---|---------|------|
| 1 | [Account Renewal Override](../../feature/030_account_renewal_override.md) | `_renewal_at` storage, monthly auto-advance, and `~Renews` rendering |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 1 | [Account Onboarding](../user_story/002_onboarding.md) | Set accurate billing renewal dates during account setup |

---

### Command :: 15. `.account.inspect`

Unified live account diagnostic — identity, subscription, org, and quota utilization for one account. Calls endpoints 002 (`GET /api/oauth/account`), 005 (`GET /api/oauth/claude_cli/roles`), and 001 (`GET /api/oauth/usage`) and renders identity fields (tagged_id, uuid, email, name), ALL membership entries with a selection-priority indicator, capabilities, rate-limit tier, and 5h/7d/Sonnet quota utilization with reset countdowns. Primary use case: diagnosing account state and remaining quota (see BUG-237 / feature 031).

-- **Parameters:** [`name::`](../param/001_name.md), [`refresh::`](../param/019_refresh.md), [`trace::`](../param/023_trace.md), [`format::`](../param/002_format.md)
-- **Exit:** 0 (success) | 1 (usage: invalid param) | 2 (runtime: account not found or credential store unreadable)

**Syntax:**

```bash
clp .account.inspect                    # default: active account
clp .account.inspect name::alice@acme.com
clp .account.inspect alice             # prefix
clp .account.inspect refresh::0        # skip token refresh on expired credentials
clp .account.inspect format::json
clp .account.inspect trace::1          # show [trace] endpoint calls to stderr
```

| Parameter | Type | Default | Purpose |
|-----------|------|---------|---------|
| `name::` | [`AccountName`](../type/001_account_name.md) | *(active account)* | Account to inspect; omit to use the currently active account |
| `refresh::` | `bool` | `1` | Attempt OAuth token refresh via isolated subprocess when `expiresAt` is locally expired, before endpoint calls |
| `trace::` | `bool` | `0` | Print `[trace]` lines to stderr for each endpoint call: URL, HTTP status, field extraction summary |
| `format::` | [`OutputFormat`](../type/002_output_format.md) | `text` | Output format: `text` (default) or `json` |

**Algorithm (5 steps):**
1. Resolve `name::` (omit → active account from `_active_{hostname}_{user}` marker); load credentials
2. `(when refresh::1 + locally expired)` Call `refresh_account_token()` to obtain a fresh token
3. Call endpoint 002 (`GET /api/oauth/account`), endpoint 005 (`GET /api/oauth/claude_cli/roles`), and endpoint 001 (`GET /api/oauth/usage`) — each independently; failure falls back to local snapshots with `(snapshot)` suffix per field (quota fields show `N/A` with no snapshot fallback)
4. Apply membership selection priority: `billing_type=stripe_subscription + claude_max` > `billing_type=stripe_subscription` > `memberships[0]`
5. Render all fields in requested `format::`

**Output (text):**

```
Account:         alice@acme.com
Name:            Alice (Alice)
Email:           alice@acme.com
Status:          🟢 valid (expires in 3h 52m)
Tagged ID:       user_01abc...def
UUID:            aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee

Memberships:     2
  [0]  billing_type=none              has_max=false  capabilities=[chat]
  [1]  billing_type=stripe_subscription  has_max=true   capabilities=[claude_max, chat]  ← selected

Billing:         stripe_subscription
Has Max:         yes
Capabilities:    [claude_max, chat]
Tier:            default_claude_max_20x

Session (5h):    45% consumed, resets in 12m
Weekly (7d):     33% consumed, resets in 1d 5h
Sonnet (7d):     53% consumed, resets in 1d 5h

Org:             alice@acme.com's Organization
Org UUID:        aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee
Org Role:        admin
Workspace UUID:  (none)
Workspace:       (none)
```

**Membership selection priority:**

| Priority | Criteria |
|----------|----------|
| 1 (highest) | `billing_type == "stripe_subscription"` AND capabilities contain `"claude_max"` |
| 2 | `billing_type == "stripe_subscription"` (any capabilities) |
| 3 (fallback) | `memberships[0]` |

The selected membership is marked `← selected` when there are multiple memberships; the `Billing:` and `Has Max:` fields reflect the selected membership.

**Examples:**

```bash
clp .account.inspect
# Account:     alice@acme.com
# Status:      🟢 valid (expires in 3h 52m)
# ...

clp .account.inspect name::i5@wbox.pro
# Account:     i5@wbox.pro
# Memberships: 2
#   [0]  billing_type=none              has_max=false  ...
#   [1]  billing_type=stripe_subscription  has_max=true  ...  ← selected
# Billing:     stripe_subscription

clp .account.inspect format::json | jq '.memberships | length'
# 2
```

**Notes:**
- Endpoints 002, 005, and 001 are called independently. A failure on one endpoint falls back to the local snapshot from `{name}.json` with a `(snapshot)` suffix per field; quota fields (endpoint 001) have no snapshot fallback and are omitted when unavailable.
- `refresh::1` (default) behaves identically to `.usage`'s `refresh::1`: calls `refresh_account_token()` once when `expiresAt` is locally expired; retries endpoint calls with the fresh token.
- See [feature/031_account_inspect.md](../../feature/031_account_inspect.md) for full design, graceful fallback semantics, and all acceptance criteria.

### Referenced Features

| # | Feature | Role |
|---|---------|------|
| 1 | [Account Inspect](../../feature/031_account_inspect.md) | Unified account diagnostic — identity, subscription, org, and quota utilization |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 1 | [Credential Diagnostics](../user_story/005_credential_diagnostics.md) | Live multi-endpoint inspection for subscription diagnosis |

---

### Command :: 16. `.account.assign`

Write (or overwrite) the per-machine active-account marker for any host+user pair without performing a full credential rotation. No `~/.claude.*` files are touched — marker-only write.

-- **Parameters:** [`name::`](../param/001_name.md) *(optional¹)*, [`for::`](../param/053_for.md), [`dry::`](../param/004_dry.md), [`trace::`](../param/023_trace.md)
-- **Exit:** 0 (success; or live usage block when `name::` absent) | 1 (usage: invalid `name::` chars, `for::` missing `@`, or empty `for::` component) | 2 (runtime: account not found)

¹ When `name::` is absent the command emits a live usage block instead of an error.

**Syntax:**

```bash
clp .account.assign                                         # live usage block (current machine + active account)
clp .account.assign name::alice@corp.com                   # assign to current machine
clp .account.assign name::alice@corp.com for::bob@laptop   # assign to remote machine
clp .account.assign name::alice@corp.com dry::1            # preview without writing
clp .account.assign name::alice for::bob@laptop            # prefix resolution
```

| Parameter | Type | Default | Purpose |
|-----------|------|---------|---------|
| `name::` | [`AccountName`](../type/001_account_name.md) | *(omit for usage block)* | Account to assign; prefix resolution supported |
| `for::` | `string` (`USER@MACHINE`) | current `$USER@resolve_hostname()` | Target identity; split on first `@`; both parts required when provided; sanitized per `active_marker_filename()` rules |
| `dry::` | `bool` | `0` | Preview the would-be write without creating or modifying any file |
| `trace::` | `bool` | `0` | Print `[trace]` lines to stderr for marker write steps |

**Algorithm (5 steps):**
1. `(when name:: absent)` Emit live usage block with current machine identity and active account; exit 0
2. Resolve `name::` via `AccountSelector`; validate account exists in credential store
3. Resolve `for::`: split on first `@` (left = user, right = machine); validate both parts non-empty; default to `$USER@resolve_hostname()` when omitted
4. Sanitize each component (alphanumeric, `-`, `.` kept; other chars → `_`); compute target filename `_active_{machine}_{user}`
5. `(when dry::1)` Print `[dry-run] would assign {name} for {user}@{machine}  →  _active_{machine}_{user}`; exit 0. `(else)` Write `{credential_store}/_active_{machine}_{user}` = `{name}`; print `Assigned {name} for {user}@{machine}  →  _active_{machine}_{user}`

**Live usage block (no `name::`):**

```
.account.assign — write the active-account marker for any machine without credential rotation.

  name::   account to assign (required)
  for::    USER@MACHINE to target  (default: current machine)
  dry::1   preview without writing

Current machine:  user1@w003  (→ _active_w003_user1)
Active account:   alice@corp.com

Ready to copy:
  clp .account.assign name::alice@corp.com
  clp .account.assign name::alice@corp.com for::user1@w003
  clp .account.assign name::alice@corp.com for::otheruser@othermachine dry::1
```

The `Current machine:` and `Active account:` lines are resolved at runtime. When no active account is set, `Active account:` shows `(none)` and the `Ready to copy:` section is omitted.

**Examples:**

```bash
clp .account.assign name::alice@corp.com for::bob@laptop
# Assigned alice@corp.com for bob@laptop  →  _active_laptop_bob

clp .account.assign name::alice@corp.com
# Assigned alice@corp.com for user1@w003  →  _active_w003_user1

clp .account.assign name::alice@corp.com for::bob@laptop dry::1
# [dry-run] would assign alice@corp.com for bob@laptop  →  _active_laptop_bob
```

**Notes:**
- Writes ONLY `{credential_store}/_active_{machine}_{user}` (marker). Never touches `~/.claude/.credentials.json`, `~/.claude.json`, `~/.claude/settings.json`, or `{name}.json`. Ownership is managed by `.account.save`, not `.account.assign` (see [Feature 036](../../feature/036_account_ownership.md)).
- `for::` is split on the first `@`: left = user component, right = machine component. Each is sanitized: alphanumeric, `-`, `.` kept; all other characters become `_`.
- Use `.account.use` for full credential rotation (credential copy + `~/.claude.*` patches + post-switch touch). Use `.account.assign` when only the preference marker needs to be set.
- After `.account.assign` for a remote machine, that machine can run `.account.use name::alice@corp.com` to activate the credentials. The marker set by `.account.assign` is visible via `.accounts` on any machine sharing the same credential store.
- See [feature/032_account_assign.md](../../feature/032_account_assign.md) for full design and all acceptance criteria.

### Referenced Features

| # | Feature | Role |
|---|---------|------|
| 1 | [Account Assign](../../feature/032_account_assign.md) | Marker-only write algorithm and `for::` resolution |
| 2 | [Per-Machine Active Marker](../../feature/025_per_machine_active_marker.md) | `_active_{machine}_{user}` marker semantics and filename sanitization |
| 3 | [Account Ownership](../../feature/036_account_ownership.md) | Ownership model; enforcement gates G1–G8; `.account.assign` is marker-only (does NOT stamp ownership); ownership stamped by `.account.save` |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 1 | [Account Onboarding](../user_story/002_onboarding.md) | Pre-configure active account for a remote machine |

---

### Command :: 17. `.account.unclaim`

Release ownership of a saved account by writing `owner: ""` to `{name}.json` via `write_owner()` directly. Pure metadata operation — credentials and active marker are NOT touched.

-- **Parameters:** [`name::`](../param/001_name.md) *(required)*, [`dry::`](../param/004_dry.md), [`trace::`](../param/023_trace.md)
-- **Exit:** 0 (success or dry-run) | 1 (usage: missing `name::`, ownership violation) | 2 (runtime: account not found)

**Syntax:**

```bash
clp .account.unclaim name::alice@acme.com           # unclaim account
clp .account.unclaim name::alice@acme.com dry::1    # preview without writing
clp .account.unclaim name::alice@acme.com trace::1  # with trace output
```

| Parameter | Type | Default | Purpose |
|-----------|------|---------|---------|
| `name::` | [`AccountName`](../type/001_account_name.md) | *(required)* | Account to unclaim; no name inference — must be provided explicitly |
| `dry::` | `bool` | `0` | Preview the would-be unclaim without modifying any file |
| `trace::` | `bool` | `0` | Print `[trace]` lines to stderr for write_owner steps |

**Algorithm (4 steps):**
1. Require `name::` (non-empty string); no name inference
2. Verify `{credential_store}/{name}.json` exists; exit 2 if absent
3. G8 ownership gate: `read_owner()` → `is_owned()`. Non-owner → exit 1 with `"ownership violation: this account is owned by {owner}"`. Gate runs BEFORE `dry::1` check
4. `(when dry::1)` Print `[dry-run] would unclaim {name}`; exit 0. `(else)` `write_owner(name, credential_store, "")` → print `unclaimed {name}`; exit 0

**Exit Codes:**

| Code | Condition |
|------|-----------|
| 0 | Ownership cleared (or dry-run preview printed) |
| 1 | Missing `name::` argument; or G8 ownership violation (account owned by another identity) |
| 2 | Account `{name}.json` does not exist in credential store |

**Examples:**

```bash
clp .account.unclaim name::alice@acme.com
# unclaimed alice@acme.com

clp .account.unclaim name::alice@acme.com dry::1
# [dry-run] would unclaim alice@acme.com

clp .account.unclaim name::alice@acme.com
# (when owned by bob@laptop) ownership violation: this account is owned by bob@laptop
```

**Notes:**
- Pure metadata operation: writes only the `owner` field in `{name}.json` via `write_owner()`. Does NOT read or write `{name}.credentials.json`, does NOT touch `~/.claude/.credentials.json`, does NOT modify the active marker.
- `name::` is required — no inference from active marker or `oauthAccount.emailAddress`. This prevents accidental unclaim of the wrong account.
- Idempotent on unowned accounts: if `owner` is already empty, `write_owner()` writes `""` again — no-op semantically.
- G8 gate evaluates BEFORE `dry::1` — a non-owner gets exit 1 even in dry-run mode, preventing information leakage about whether an unclaim would succeed.
- See [feature/036_account_ownership.md](../../feature/036_account_ownership.md) for the full ownership model, G1–G8 enforcement gates, and all acceptance criteria.

### Referenced Features

| # | Feature | Role |
|---|---------|------|
| 1 | [Account Ownership](../../feature/036_account_ownership.md) | G8 ownership gate; `write_owner()` / `read_owner()` / `is_owned()` API; AC-02/AC-16/AC-17 |

### Referenced User Stories

*None — `.account.unclaim` is an operational maintenance command, not a user-story-driven feature.*
