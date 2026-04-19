# Commands

### All Commands (12 total)

| # | Command | Purpose | Params | Example |
|---|---------|---------|--------|---------|
| 1 | `.` | Show help information (hidden dot-shorthand) | 0 | `clp .` |
| 2 | `.help` | Display command reference and usage examples | 0 | `clp .help` |
| 3 | `.account.list` | List all saved accounts with subscription type and token state | 2 | `clp .account.list` |
| 4 | `.account.status` | Show account name and token state; optionally query a named account | 3 | `clp .account.status name::work` |
| 5 | `.account.save` | Save current credentials as a named account profile | 2 | `clp .account.save name::work` |
| 6 | `.account.switch` | Switch active account by name with atomic credential rotation | 2 | `clp .account.switch name::personal` |
| 7 | `.account.delete` | Delete a saved account from the account store | 2 | `clp .account.delete name::old` |
| 8 | `.token.status` | Show active OAuth token expiry classification | 3 | `clp .token.status` |
| 9 | `.paths` | Show all resolved ~/.claude/ canonical file paths | 2 | `clp .paths` |
| 10 | `.usage` | Show token usage statistics from stats-cache.json | 2 | `clp .usage v::0` |
| 11 | `.credentials.status` | Show live credential metadata without account store dependency | 2 | `clp .credentials.status` |
| 12 | `.account.limits` | Show rate-limit utilization for the active or named account | 3 | `clp .account.limits name::work` |

**Total:** 12 commands (10 visible + 2 hidden)

---

### Quick Reference

**Required Parameters:**
- `name::` — required on `.account.save`, `.account.switch`, `.account.delete`

**Most-Used Parameters:**
- `verbosity::` / `v::` — 7 commands
- `format::` — 7 commands

**Commands by Parameter Count:**

| Count | Commands |
|-------|----------|
| 0 | `.`, `.help` |
| 2 | `.account.list`, `.account.save`, `.account.switch`, `.account.delete`, `.paths`, `.usage`, `.credentials.status` |
| 3 | `.account.status`, `.token.status`, `.account.limits` |

---

### Command :: 3. `.account.list`

Enumerates all credential snapshots in `~/.claude/accounts/` and displays name, subscription type, rate-limit tier, token expiry, and active marker. Use this to see which accounts are available before switching.

-- **Parameters:** [`v::`](params.md#parameter--2-verbosity--v), [`format::`](params.md#parameter--3-format)
-- **Exit:** 0 (success) | 2 (runtime: accounts dir unreadable)

**Syntax:**

```bash
clp .account.list
clp .account.list v::0
clp .account.list format::json
```

| Parameter | Type | Default | Purpose |
|-----------|------|---------|---------|
| `v::` | [`VerbosityLevel`] | `1` | Output detail level |
| `format::` | [`OutputFormat`] | `text` | Output format |

**Examples:**

```bash
clp .account.list
# work <- active (max, standard, expires in 47m)
# personal (pro, standard, expires in 3h12m)

clp .account.list v::0
# work
# personal

clp .account.list format::json
# [{"name":"work","subscription_type":"max","rate_limit_tier":"standard","expires_at_ms":1711234567000,"is_active":true},...]
```

---

### Command :: 4. `.account.status`

Reads the `_active` marker and the active OAuth token to report account name, token state, subscription, tier, email, and org in one call. With the optional `name::` parameter, queries any named account's stored token state regardless of which account is active.

-- **Parameters:** [`name::`](params.md#parameter--1-name) *(optional)*, [`v::`](params.md#parameter--2-verbosity--v), [`format::`](params.md#parameter--3-format)
-- **Exit:** 0 (success) | 1 (usage: invalid `name::` chars) | 2 (runtime: account not found, no active account set, HOME unset)

**Syntax:**

```bash
clp .account.status
clp .account.status name::work
clp .account.status name::personal v::2
clp .account.status format::json
```

| Parameter | Type | Default | Purpose |
|-----------|------|---------|---------|
| `name::` | [`AccountName`] | *(omit for active)* | Query a named account instead of the active account |
| `v::` | [`VerbosityLevel`] | `1` | Output detail level |
| `format::` | [`OutputFormat`] | `text` | Output format |

**Examples:**

```bash
clp .account.status
# Account: work
# Token:   valid
# Sub:     pro
# Tier:    standard
# Email:   alice@example.com
# Org:     Acme Corp

clp .account.status name::personal
# Account: personal
# Token:   expired
# Sub:     pro
# Tier:    standard
# Email:   N/A
# Org:     N/A

clp .account.status name::work v::1
# Account: work
# Token:   valid
# Sub:     pro
# Tier:    standard
# Email:   alice@example.com
# Org:     Acme Corp

clp .account.status v::0
# work
# valid

clp .account.status v::2
# Account: work
# Token:   valid
# Sub:     pro
# Tier:    standard
# Expires: in 47h 12m
# Email:   alice@example.com
# Org:     Acme Corp

clp .account.status format::json
# {"account":"work","token":"valid"}
```

**Notes:**
- Without `name::`: reads the active account from `_active` and the live credentials file. Token state can be `unknown` if the credentials file is unreadable.
- With `name::`: reads the named account's own `expiresAt` from its stored credential file. Token state is always `valid`, `expiring in Xm`, or `expired` (never `unknown`).
- At `v::1`: shows `Sub:` (subscriptionType) and `Tier:` (rateLimitTier) for all accounts; shows `Email:` and `Org:` from `~/.claude/.claude.json` for the active account only — `N/A` for non-active accounts.
- Reports exit 1 for invalid `name::` characters; exit 2 if no `_active` marker is set or the named account is not found.

---

### Command :: 5. `.account.save`

Copies `~/.claude/.credentials.json` to `~/.claude/accounts/{name}.credentials.json`, creating the account store directory if needed. Use this to snapshot the current credentials before switching accounts.

-- **Parameters:** [`name::`](params.md#parameter--1-name) **(required)**, [`dry::`](params.md#parameter--5-dry)
-- **Exit:** 0 (success) | 1 (usage: invalid name) | 2 (runtime: credentials unreadable)

**Syntax:**

```bash
clp .account.save name::work
clp .account.save name::work dry::1
```

| Parameter | Type | Default | Purpose |
|-----------|------|---------|---------|
| `name::` | [`AccountName`] | **(required)** | Account name to save as |
| `dry::` | `bool` | `0` | Preview action without executing |

**Examples:**

```bash
clp .account.save name::work
# saved current credentials as 'work'

clp .account.save name::work dry::1
# [dry-run] would save current credentials as 'work'
```

---

### Command :: 6. `.account.switch`

Atomically overwrites `~/.claude/.credentials.json` with the named account's credentials (write-then-rename), then updates the `_active` marker. Use this to rotate to a different account when the current token expires.

-- **Parameters:** [`name::`](params.md#parameter--1-name) **(required)**, [`dry::`](params.md#parameter--5-dry)
-- **Exit:** 0 (success) | 1 (usage: invalid name) | 2 (runtime: account not found)

**Syntax:**

```bash
clp .account.switch name::personal
clp .account.switch name::personal dry::1
```

| Parameter | Type | Default | Purpose |
|-----------|------|---------|---------|
| `name::` | [`AccountName`] | **(required)** | Account name to switch to |
| `dry::` | `bool` | `0` | Preview action without executing |

**Examples:**

```bash
clp .account.switch name::personal
# switched to 'personal'

clp .account.switch name::personal dry::1
# [dry-run] would switch to 'personal'
```

---

### Command :: 7. `.account.delete`

Removes `~/.claude/accounts/{name}.credentials.json` from the account store. Refuses to delete the currently active account — switch to another account first.

-- **Parameters:** [`name::`](params.md#parameter--1-name) **(required)**, [`dry::`](params.md#parameter--5-dry)
-- **Exit:** 0 (success) | 1 (usage: invalid name) | 2 (runtime: account not found, account is active)

**Syntax:**

```bash
clp .account.delete name::old
clp .account.delete name::old dry::1
```

| Parameter | Type | Default | Purpose |
|-----------|------|---------|---------|
| `name::` | [`AccountName`] | **(required)** | Account name to delete |
| `dry::` | `bool` | `0` | Preview action without executing |

**Examples:**

```bash
clp .account.delete name::old
# deleted account 'old'

clp .account.delete name::old dry::1
# [dry-run] would delete account 'old'

clp .account.delete name::work
# error: cannot delete active account 'work' — switch to another account first
```

---

### Command :: 8. `.token.status`

Reads `expiresAt` from `~/.claude/.credentials.json` and classifies the active OAuth token as Valid, ExpiringSoon, or Expired. Use this to detect when account rotation is needed.

-- **Parameters:** [`v::`](params.md#parameter--2-verbosity--v), [`format::`](params.md#parameter--3-format), [`threshold::`](params.md#parameter--4-threshold)
-- **Exit:** 0 (success) | 2 (runtime: credentials unreadable, expiresAt unparseable)

**Syntax:**

```bash
clp .token.status
clp .token.status threshold::1800
clp .token.status format::json
```

| Parameter | Type | Default | Purpose |
|-----------|------|---------|---------|
| `v::` | [`VerbosityLevel`] | `1` | Output detail level |
| `format::` | [`OutputFormat`] | `text` | Output format |
| `threshold::` | [`WarningThreshold`] | `3600` | ExpiringSoon threshold in seconds |

**Examples:**

```bash
clp .token.status
# valid — 47m remaining

clp .token.status v::0
# valid

clp .token.status threshold::1800
# expiring soon — 25m remaining

clp .token.status format::json
# {"status":"valid","expires_in_secs":2820}
```

---

### Command :: 9. `.paths`

Displays all canonical `~/.claude/` file and directory paths resolved from `HOME`. Use this for diagnostics and tooling integration.

-- **Parameters:** [`v::`](params.md#parameter--2-verbosity--v), [`format::`](params.md#parameter--3-format)
-- **Exit:** 0 (success) | 2 (runtime: HOME not set)

**Syntax:**

```bash
clp .paths
clp .paths format::json
```

| Parameter | Type | Default | Purpose |
|-----------|------|---------|---------|
| `v::` | [`VerbosityLevel`] | `1` | Output detail level |
| `format::` | [`OutputFormat`] | `text` | Output format |

**Examples:**

```bash
clp .paths
# credentials: /home/user/.claude/.credentials.json
# accounts:    /home/user/.claude/accounts/
# projects:    /home/user/.claude/projects/
# stats:       /home/user/.claude/stats-cache.json
# settings:    /home/user/.claude/settings.json
# session-env: /home/user/.claude/session-env/
# sessions:    /home/user/.claude/sessions/

clp .paths v::0
# /home/user/.claude

clp .paths format::json
# {"base":"/home/user/.claude","credentials":"/home/user/.claude/.credentials.json",...}
```

---

### Command :: 10. `.usage`

Reads `~/.claude/stats-cache.json` and displays token usage statistics for the last 7 days. Shows per-model token counts with compact formatting and daily breakdowns at higher verbosity levels.

-- **Parameters:** [`v::`](params.md#parameter--2-verbosity--v), [`format::`](params.md#parameter--3-format)
-- **Exit:** 0 (success) | 2 (runtime: stats file missing or HOME not set)

**Syntax:**

```bash
clp .usage
clp .usage v::0
clp .usage format::json
```

| Parameter | Type | Default | Purpose |
|-----------|------|---------|---------|
| `v::` | [`VerbosityLevel`] | `1` | Output detail level |
| `format::` | [`OutputFormat`] | `text` | Output format |

**Examples:**

```bash
clp .usage
# Token Usage (Mar 22 – Mar 29)
# sonnet-4-20250514  142.3K  87%
# haiku-3.5          21.0K   13%
# Total: 163.3K tokens

clp .usage v::0
# 163.3K

clp .usage v::2
# Token Usage (Mar 22 – Mar 29)
# ...per-model summary...
# Daily:
#   Mar 29: sonnet-4 42.1K, haiku-3.5 3.2K
#   Mar 28: sonnet-4 38.7K, haiku-3.5 5.1K
#   ...

clp .usage format::json
# {"period_days":7,"period_start":"2026-03-22","period_end":"2026-03-29","total_tokens":163300,"by_model":[...]}
```

---

### Command :: 11. `.credentials.status`

Show live credential metadata by reading `~/.claude/.credentials.json` directly. Succeeds on any authenticated machine regardless of whether account store setup exists.

-- **Parameters:** [`v::`](params.md#parameter--2-verbosity--v), [`format::`](params.md#parameter--3-format)
-- **Exit:** 0 (success) | 2 (credential file absent or HOME unset)

**Syntax:**

```bash
clp .credentials.status
clp .credentials.status v::2
clp .credentials.status format::json
```

| Parameter | Type | Default | Purpose |
|-----------|------|---------|---------|
| `v::` | [`VerbosityLevel`] | `1` | Output detail level (0=sub+token, 1=+tier/email/org, 2=+expiry) |
| `format::` | [`OutputFormat`] | `text` | Output format |

**Examples:**

```bash
clp .credentials.status
# Sub:     pro
# Tier:    standard
# Token:   valid
# Email:   user@example.com
# Org:     Acme Corp

clp .credentials.status v::0
# Sub:     pro
# Token:   valid

clp .credentials.status v::2
# Sub:     pro
# Tier:    standard
# Token:   valid
# Expires: in 3h 42m
# Email:   user@example.com
# Org:     Acme Corp

clp .credentials.status format::json
# {"subscription":"pro","tier":"standard","token":"valid","expires_in_secs":13320}
```

---

### Command :: 12. `.account.limits`

Show plan and rate-limit utilization for the active or named account. Displays session usage, weekly all-model usage, and weekly Sonnet usage with percentage consumed and reset times.

-- **Parameters:** [`name::`](params.md#parameter--1-name) *(optional)*, [`v::`](params.md#parameter--2-verbosity--v), [`format::`](params.md#parameter--3-format)
-- **Exit:** 0 (success) | 1 (usage: invalid `name::` chars) | 2 (runtime: account not found, data unavailable, HOME unset)

**Syntax:**

```bash
clp .account.limits
clp .account.limits name::work
clp .account.limits format::json
```

| Parameter | Type | Default | Purpose |
|-----------|------|---------|---------|
| `name::` | [`AccountName`] | *(omit for active)* | Query a named account instead of the active account |
| `v::` | [`VerbosityLevel`] | `1` | Output detail level |
| `format::` | [`OutputFormat`] | `text` | Output format |

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
- Data source: `anthropic-ratelimit-unified-*` response headers from a lightweight API call; see [feature/013_account_limits.md](../feature/013_account_limits.md). Happy path blocked until HTTP client added to workspace.
- With `name::`: shows limits for the named account (requires account store entry).
- Without `name::`: shows limits for the active account.
