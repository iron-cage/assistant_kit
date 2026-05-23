# Commands :: Account

Account management commands: list, save, use, delete, limits, and relogin.

---

### Command :: 3. `.accounts`

List all saved accounts or show a single named account with per-field presence control. Without `name::`: shows every account in the credential store as an indented key-val block; with `name::EMAIL`: shows that account's block only.

-- **Parameters:** [`name::`](../param/01_name.md) *(optional)*, [`active::`](../param/13_active.md), [`current::`](../param/18_current.md), [`sub::`](../param/06_sub.md), [`tier::`](../param/07_tier.md), [`expires::`](../param/09_expires.md), [`email::`](../param/10_email.md), [`display_name::`](../param/14_display_name.md), [`role::`](../param/15_role.md), [`billing::`](../param/16_billing.md), [`model::`](../param/17_model.md), [`format::`](../param/02_format.md)
-- **Exit:** 0 (success) | 1 (usage: invalid `name::` chars) | 2 (runtime: account not found or credential store unreadable)

**Syntax:**

```bash
clp .accounts
clp .accounts name::alice@acme.com
clp .accounts alice@acme.com         # positional: same as name::alice@acme.com
clp .accounts i3                     # prefix: first saved account starting with "i3"
clp .accounts sub::0 tier::0
clp .accounts display_name::1 role::1 billing::1 model::1
clp .accounts format::json
clp .accounts format::table
```

| Parameter | Type | Default | Purpose |
|-----------|------|---------|---------|
| `name::` | [`AccountName`](../type/01_account_name.md) | *(omit to list all)* | Show a single named account instead of listing all |
| `active::` | `bool` | `1` | Show active/inactive status line |
| `current::` | `bool` | `1` | Show current (live) account line; suppressed when `~/.claude/.credentials.json` is unreadable |
| `sub::` | `bool` | `1` | Show subscription type line |
| `tier::` | `bool` | `1` | Show rate-limit tier line |
| `expires::` | `bool` | `1` | Show token expiry duration line |
| `email::` | `bool` | `1` | Show email address from saved `{name}.claude.json` snapshot |
| `display_name::` | `bool` | `0` | Show display name from saved `~/.claude.json` snapshot (opt-in) |
| `role::` | `bool` | `0` | Show organisation role from saved `~/.claude.json` snapshot (opt-in) |
| `billing::` | `bool` | `0` | Show billing type from saved `~/.claude.json` snapshot (opt-in) |
| `model::` | `bool` | `0` | Show active model from saved `settings.json` snapshot (opt-in) |
| `format::` | [`OutputFormat`](../type/02_output_format.md) | `text` | Output format |

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

---

### Command :: 4. `.account.save`

Copies `~/.claude/.credentials.json` to `{credential_store}/{name}.credentials.json` and snapshots `~/.claude.json` and `~/.claude/settings.json` as named per-account files. Use this to preserve the full current account state before switching.

-- **Parameters:** [`name::`](../param/01_name.md), [`dry::`](../param/04_dry.md)
-- **Exit:** 0 (success) | 1 (usage: invalid name or cannot infer email) | 2 (runtime: credentials unreadable)

**Syntax:**

```bash
clp .account.save                          # infer name from ~/.claude.json emailAddress
clp .account.save name::alice@acme.com    # explicit name
clp .account.save name::alice@acme.com dry::1
```

| Parameter | Type | Default | Purpose |
|-----------|------|---------|---------|
| `name::` | [`AccountName`](../type/01_account_name.md) | `auto` (inferred from `~/.claude.json` `emailAddress`) | Account email to save as |
| `dry::` | `bool` | `0` | Preview action without executing |

**Examples:**

```bash
clp .account.save
# saved current credentials as 'alice@acme.com'   (inferred from ~/.claude.json)

clp .account.save name::alice@acme.com dry::1
# [dry-run] would save current credentials as 'alice@acme.com'
```

**Notes:**
- Also writes `{credential_store}/_active` = `{name}` on every successful save.

---

### Command :: 5. `.account.use`

Atomically overwrites `~/.claude/.credentials.json` with the named account's credentials (write-then-rename), updates the `_active` marker, and best-effort restores the account's `~/.claude.json` and `~/.claude/settings.json` snapshots.

-- **Parameters:** [`name::`](../param/01_name.md) **(required)**, [`dry::`](../param/04_dry.md)
-- **Exit:** 0 (success) | 1 (usage: invalid name) | 2 (runtime: account not found)

**Syntax:**

```bash
clp .account.use name::alice@home.com
clp .account.use alice@home.com          # positional: same as name::alice@home.com
clp .account.use i3                      # prefix: first saved account starting with "i3"
clp .account.use name::alice@home.com dry::1
```

| Parameter | Type | Default | Purpose |
|-----------|------|---------|---------|
| `name::` | [`AccountName`](../type/01_account_name.md) | **(required)** | Account email to switch to |
| `dry::` | `bool` | `0` | Preview action without executing |

**Examples:**

```bash
clp .account.use name::alice@home.com
# switched to 'alice@home.com'

clp .account.use name::alice@home.com dry::1
# [dry-run] would switch to 'alice@home.com'
```

---

### Command :: 6. `.account.delete`

Removes `{credential_store}/{name}.credentials.json` from the credential store and best-effort removes the accompanying `{name}.claude.json` and `{name}.settings.json` snapshot files.

-- **Parameters:** [`name::`](../param/01_name.md) **(required)**, [`dry::`](../param/04_dry.md)
-- **Exit:** 0 (success) | 1 (usage: invalid name) | 2 (runtime: account not found)

**Syntax:**

```bash
clp .account.delete name::alice@oldco.com
clp .account.delete alice@oldco.com         # positional
clp .account.delete i3                      # prefix
clp .account.delete name::alice@oldco.com dry::1
```

| Parameter | Type | Default | Purpose |
|-----------|------|---------|---------|
| `name::` | [`AccountName`](../type/01_account_name.md) | **(required)** | Account email to delete |
| `dry::` | `bool` | `0` | Preview action without executing |

**Examples:**

```bash
clp .account.delete name::alice@oldco.com
# deleted account 'alice@oldco.com'

clp .account.delete name::alice@oldco.com dry::1
# [dry-run] would delete account 'alice@oldco.com'
```

**Notes:**
- Snapshot files (`{name}.claude.json`, `{name}.settings.json`) are removed best-effort: missing snapshots are silently skipped.
- Deleting the active account also removes the `_active` marker.

---

### Command :: 11. `.account.limits`

Show rate-limit utilization for the active or named account. Displays session (5h) usage, weekly all-model (7d) usage, and rate-limit status with percentage consumed and reset times.

-- **Parameters:** [`name::`](../param/01_name.md) *(optional)*, [`format::`](../param/02_format.md)
-- **Exit:** 0 (success) | 1 (usage: invalid `name::` chars) | 2 (runtime: account not found, data unavailable, HOME unset)

**Syntax:**

```bash
clp .account.limits
clp .account.limits name::alice@acme.com
clp .account.limits alice@acme.com       # positional
clp .account.limits i3                   # prefix
clp .account.limits format::json
```

| Parameter | Type | Default | Purpose |
|-----------|------|---------|---------|
| `name::` | [`AccountName`](../type/01_account_name.md) | *(omit for active)* | Query a named account instead of the active account |
| `format::` | [`OutputFormat`](../type/02_output_format.md) | `text` | Output format |

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

-- **Parameters:** [`name::`](../param/01_name.md) **(required)**, [`dry::`](../param/04_dry.md)
-- **Exit:** 0 (success: credentials refreshed and saved) | 1 (usage: missing or invalid name) | 2 (runtime: account not found or Claude spawn failed) | 3 (timeout or login abandoned: claude exited without updating credentials)

**Syntax:**

```bash
clp .account.relogin name::i3@wbox.pro
clp .account.relogin i3@wbox.pro       # positional
clp .account.relogin i3               # prefix
clp .account.relogin name::i3@wbox.pro dry::1
```

| Parameter | Type | Default | Purpose |
|-----------|------|---------|---------|
| `name::` | [`AccountName`](../type/01_account_name.md) | **(required)** | Account to re-authenticate |
| `dry::` | `bool` | `0` | Preview the steps without executing |

**Mechanism (6 steps):**
1. Resolve `name::` via [`AccountSelector`](../type/04_account_selector.md) → validate account exists in credential store
2. Snapshot the current active account name (for restoration after login)
3. `switch_account(name)` — makes the named account active in `~/.claude/`
4. Spawn `claude` with inherited TTY (stdin/stdout/stderr connected — NOT isolated subprocess) — Claude detects empty or invalid credentials and opens the browser login page
5. Wait for `claude` to exit; if `~/.claude/.credentials.json` changed → `account::save(name)` propagates fresh credentials to credential store
6. `switch_account(original_active)` — restore the prior active account

**Examples:**

```bash
clp .account.relogin name::i3@wbox.pro
# [relogin] switching to i3@wbox.pro...
# [relogin] spawning claude for browser re-authentication (Ctrl-C to abort)
# ... (user completes browser login) ...
# [relogin] credentials updated — saving as i3@wbox.pro
# [relogin] restored active account: i12@wbox.pro
# relogin successful

clp .account.relogin name::i3@wbox.pro dry::1
# [dry-run] would relogin i3@wbox.pro:
#   1. switch_account(i3@wbox.pro)
#   2. spawn claude (inherited TTY)
#   3. wait for credential file change
#   4. account::save(i3@wbox.pro)
#   5. switch_account(i12@wbox.pro)  [restore]
```

**Notes:**
- Requires a TTY — `clp .account.relogin` in a piped non-TTY context will fail at step 4 (Claude cannot open a browser or display the login prompt).
- The `claude` subprocess runs with the full inherited environment; no credential isolation (contrast with `refresh::1` which uses an isolated subprocess).
- If `claude` exits without updating `~/.claude/.credentials.json`, the command exits 3. The active account is still restored (step 6 runs regardless of outcome).
- Use this when `clp .usage refresh::1 trace::1` shows `run_isolated: OK credentials=None` for an account — that trace indicates a dead refresh token requiring full browser re-auth.

---

### Command :: 13. `.account.rotate`

Auto-rotate to the best inactive account: selects the saved account with the highest remaining token expiry and atomically switches to it. No account name required — selection is fully automatic via `account::auto_rotate()`.

-- **Parameters:** [`dry::`](../param/04_dry.md)
-- **Exit:** 0 (success: switched) | 1 (usage: invalid param) | 2 (runtime: no inactive accounts available, or credential store unreadable)

**Syntax:**

```bash
clp .account.rotate
clp .account.rotate dry::1
```

| Parameter | Type | Default | Purpose |
|-----------|------|---------|---------|
| `dry::` | `bool` | `0` | Preview which account would be selected without switching |

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
- "Inactive" = any saved account whose name differs from the `_active` marker.
- Equivalent to `clp .account.use $(best_account)` but without requiring the caller to determine the best candidate.
- If only one account is configured, or all saved accounts match the active one, exits 2.
- For explicit selection by name, use [`.account.use`](#command--5-accountuse).
