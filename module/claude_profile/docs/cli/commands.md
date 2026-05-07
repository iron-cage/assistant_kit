# Commands

### All Commands (11 total)

| # | Command | Purpose | Params | Example |
|---|---------|---------|--------|---------|
| 1 | `.` | Show help information (hidden dot-shorthand) | 0 | `clp .` |
| 2 | `.help` | Display command reference and usage examples | 0 | `clp .help` |
| 3 | `.accounts` | List all saved accounts or show a single named account | 7 | `clp .accounts` |
| 4 | `.account.save` | Save current credentials as a named account profile | 2 | `clp .account.save name::alice@acme.com` |
| 5 | `.account.switch` | Switch active account by name with atomic credential rotation | 2 | `clp .account.switch name::alice@home.com` |
| 6 | `.account.delete` | Delete a saved account from the account store | 2 | `clp .account.delete name::alice@oldco.com` |
| 7 | `.token.status` | Show active OAuth token expiry classification | 3 | `clp .token.status` |
| 8 | `.paths` | Show all resolved ~/.claude/ canonical file paths | 2 | `clp .paths` |
| 9 | `.usage` | Show token usage statistics from stats-cache.json | 2 | `clp .usage v::0` |
| 10 | `.credentials.status` | Show live credential metadata without account store dependency | 14 | `clp .credentials.status` |
| 11 | `.account.limits` | Show rate-limit utilization for the active or named account | 3 | `clp .account.limits name::alice@acme.com` |

**Total:** 11 commands (9 visible + 2 hidden)

---

### Quick Reference

**Required Parameters:**
- `name::` — required on `.account.save`, `.account.switch`, `.account.delete` (must be an email address)

**Most-Used Parameters:**
- `format::` — 6 commands
- `verbosity::` / `v::` — 4 commands

**Commands by Parameter Count:**

| Count | Commands |
|-------|----------|
| 0 | `.`, `.help` |
| 2 | `.account.save`, `.account.switch`, `.account.delete`, `.paths`, `.usage` |
| 3 | `.token.status`, `.account.limits` |
| 7 | `.accounts` |
| 14 | `.credentials.status` |

---

### Meta-flag :: `--version` / `-V`

Print the binary name and version string, then exit. This flag takes priority over all commands and parameters — it wins regardless of argv order. Not a command (does not appear in `.help` listing).

- **Aliases:** `-V`
- **Exit:** 0 (always)
- **Output:** `clp X.Y.Z` (one line on stdout; stderr is empty)
- **Implementation:** `src/lib.rs::cli::run()`

**Examples:**

```bash
clp --version   # → "clp 0.12.3"
clp -V          # → identical output
```

---

### Command :: 3. `.accounts`

List all saved accounts or show a single named account with per-field presence control. Without `name::`: shows every account in the credential store as an indented key-val block; with `name::EMAIL`: shows that account's block only.

-- **Parameters:** [`name::`](params.md#parameter--1-name) *(optional)*, [`active::`](params.md#parameter--15-active), [`sub::`](params.md#parameter--7-sub), [`tier::`](params.md#parameter--8-tier), [`expires::`](params.md#parameter--10-expires), [`org::`](params.md#parameter--12-org), [`format::`](params.md#parameter--3-format)
-- **Exit:** 0 (success) | 1 (usage: invalid `name::` chars) | 2 (runtime: account not found or credential store unreadable)

**Syntax:**

```bash
clp .accounts
clp .accounts name::alice@acme.com
clp .accounts sub::0 tier::0
clp .accounts format::json
```

| Parameter | Type | Default | Purpose |
|-----------|------|---------|---------|
| `name::` | [`AccountName`](types.md#type--1-accountname) | *(omit to list all)* | Show a single named account instead of listing all |
| `active::` | `bool` | `1` | Show active/inactive status line |
| `sub::` | `bool` | `1` | Show subscription type line |
| `tier::` | `bool` | `1` | Show rate-limit tier line |
| `expires::` | `bool` | `1` | Show token expiry duration line |
| `org::` | `bool` | `1` | Show organisation name line |
| `format::` | [`OutputFormat`](types.md#type--3-outputformat) | `text` | Output format |

**Examples:**

```bash
clp .accounts
# alice@acme.com
#   Active:  yes
#   Sub:     max
#   Tier:    default_claude_max_20x
#   Expires: in 2h 11m
#   Org:     N/A
#
# alice@home.com
#   Active:  no
#   Sub:     pro
#   Tier:    default_claude_pro
#   Expires: in 5h 30m
#   Org:     N/A

clp .accounts name::alice@acme.com
# alice@acme.com
#   Active:  yes
#   Sub:     max
#   Tier:    default_claude_max_20x
#   Expires: in 2h 11m
#   Org:     N/A

clp .accounts sub::0 tier::0 org::0
# alice@acme.com
#   Active:  yes
#   Expires: in 2h 11m
#
# alice@home.com
#   Active:  no
#   Expires: in 5h 30m

clp .accounts active::0 sub::0 tier::0 expires::0 org::0
# alice@acme.com
# alice@home.com

clp .accounts format::json
# [{"name":"alice@acme.com","is_active":true,"subscription_type":"max","rate_limit_tier":"default_claude_max_20x","expires_at_ms":1711234567000,"org":"N/A"},{"name":"alice@home.com","is_active":false,"subscription_type":"pro","rate_limit_tier":"default_claude_pro","expires_at_ms":1711243567000,"org":"N/A"}]
```

**Notes:**
- Without `name::`: all accounts listed as indented blocks, separated by blank lines. Empty store → `(no accounts configured)`.
- With `name::EMAIL`: shows exactly one account's block — same format as listing.
- Field params affect text output only; `format::json` always includes all fields regardless of presence params.
- Reports exit 1 for invalid `name::` value; exit 2 if the named account is not found.

---

### Command :: 4. `.account.save`

Copies `~/.claude/.credentials.json` to `{credential_store}/{name}.credentials.json`, creating the credential store directory if needed. Use this to snapshot the current credentials before switching accounts.

-- **Parameters:** [`name::`](params.md#parameter--1-name) **(required)**, [`dry::`](params.md#parameter--5-dry)
-- **Exit:** 0 (success) | 1 (usage: invalid name) | 2 (runtime: credentials unreadable)

**Syntax:**

```bash
clp .account.save name::alice@acme.com
clp .account.save name::alice@acme.com dry::1
```

| Parameter | Type | Default | Purpose |
|-----------|------|---------|---------|
| `name::` | [`AccountName`] | **(required)** | Account email to save as |
| `dry::` | `bool` | `0` | Preview action without executing |

**Examples:**

```bash
clp .account.save name::alice@acme.com
# saved current credentials as 'alice@acme.com'

clp .account.save name::alice@acme.com dry::1
# [dry-run] would save current credentials as 'alice@acme.com'
```

---

### Command :: 5. `.account.switch`

Atomically overwrites `~/.claude/.credentials.json` with the named account's credentials (write-then-rename), then updates the `_active` marker. Use this to rotate to a different account when the current token expires.

-- **Parameters:** [`name::`](params.md#parameter--1-name) **(required)**, [`dry::`](params.md#parameter--5-dry)
-- **Exit:** 0 (success) | 1 (usage: invalid name) | 2 (runtime: account not found)

**Syntax:**

```bash
clp .account.switch name::alice@home.com
clp .account.switch name::alice@home.com dry::1
```

| Parameter | Type | Default | Purpose |
|-----------|------|---------|---------|
| `name::` | [`AccountName`] | **(required)** | Account email to switch to |
| `dry::` | `bool` | `0` | Preview action without executing |

**Examples:**

```bash
clp .account.switch name::alice@home.com
# switched to 'alice@home.com'

clp .account.switch name::alice@home.com dry::1
# [dry-run] would switch to 'alice@home.com'
```

---

### Command :: 6. `.account.delete`

Removes `{credential_store}/{name}.credentials.json` from the credential store. Refuses to delete the currently active account — switch to another account first.

-- **Parameters:** [`name::`](params.md#parameter--1-name) **(required)**, [`dry::`](params.md#parameter--5-dry)
-- **Exit:** 0 (success) | 1 (usage: invalid name) | 2 (runtime: account not found, account is active)

**Syntax:**

```bash
clp .account.delete name::alice@oldco.com
clp .account.delete name::alice@oldco.com dry::1
```

| Parameter | Type | Default | Purpose |
|-----------|------|---------|---------|
| `name::` | [`AccountName`] | **(required)** | Account email to delete |
| `dry::` | `bool` | `0` | Preview action without executing |

**Examples:**

```bash
clp .account.delete name::alice@oldco.com
# deleted account 'alice@oldco.com'

clp .account.delete name::alice@oldco.com dry::1
# [dry-run] would delete account 'alice@oldco.com'

clp .account.delete name::alice@acme.com
# error: cannot delete active account 'alice@acme.com' — switch to another account first
```

---

### Command :: 7. `.token.status`

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

### Command :: 8. `.paths`

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
# credentials:      /home/user/.claude/.credentials.json
# credential_store: /home/user/.persistent/claude/credential/
# projects:         /home/user/.claude/projects/
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

### Command :: 9. `.usage`

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

### Command :: 10. `.credentials.status`

Show live credential metadata by reading `~/.claude/.credentials.json` directly. Succeeds on any authenticated machine regardless of whether account store setup exists.

-- **Parameters:** [`format::`](params.md#parameter--3-format), [`account::`](params.md#parameter--6-account), [`sub::`](params.md#parameter--7-sub), [`tier::`](params.md#parameter--8-tier), [`token::`](params.md#parameter--9-token), [`expires::`](params.md#parameter--10-expires), [`email::`](params.md#parameter--11-email), [`org::`](params.md#parameter--12-org), [`file::`](params.md#parameter--13-file), [`saved::`](params.md#parameter--14-saved), [`display_name::`](params.md#parameter--16-display_name), [`role::`](params.md#parameter--17-role), [`billing::`](params.md#parameter--18-billing), [`model::`](params.md#parameter--19-model)
-- **Exit:** 0 (success) | 2 (credential file absent or HOME unset)

**Syntax:**

```bash
clp .credentials.status
clp .credentials.status email::0 org::0
clp .credentials.status file::1 saved::1
clp .credentials.status display_name::1 role::1 billing::1 model::1
clp .credentials.status format::json
```

| Parameter | Type | Default | Purpose |
|-----------|------|---------|---------|
| `format::` | [`OutputFormat`](types.md#type--3-outputformat) | `text` | Output format |
| `account::` | `bool` | `1` | Show active account name line |
| `sub::` | `bool` | `1` | Show subscription type line |
| `tier::` | `bool` | `1` | Show rate-limit tier line |
| `token::` | `bool` | `1` | Show token status line |
| `expires::` | `bool` | `1` | Show token expiry duration line |
| `email::` | `bool` | `1` | Show email address line |
| `org::` | `bool` | `1` | Show organisation name line |
| `file::` | `bool` | `0` | Show credentials file path (opt-in) |
| `saved::` | `bool` | `0` | Show saved account count (opt-in) |
| `display_name::` | `bool` | `0` | Show display name from `~/.claude.json` (opt-in) |
| `role::` | `bool` | `0` | Show organisation role from `~/.claude.json` (opt-in) |
| `billing::` | `bool` | `0` | Show billing type from `~/.claude.json` (opt-in) |
| `model::` | `bool` | `0` | Show active model from `~/.claude/settings.json` (opt-in) |

**Examples:**

```bash
clp .credentials.status
# Account: alice@acme.com
# Sub:     max
# Tier:    default_claude_max_20x
# Token:   valid
# Expires: in 7h 24m
# Email:   N/A
# Org:     N/A

clp .credentials.status email::0 org::0
# Account: alice@acme.com
# Sub:     max
# Tier:    default_claude_max_20x
# Token:   valid
# Expires: in 7h 24m

clp .credentials.status file::1 saved::1
# Account: alice@acme.com
# Sub:     max
# Tier:    default_claude_max_20x
# Token:   valid
# Expires: in 7h 24m
# Email:   N/A
# Org:     N/A
# File:    /home/user/.claude/.credentials.json
# Saved:   2 account(s)

clp .credentials.status display_name::1 role::1 billing::1 model::1
# Account: alice@acme.com
# Sub:     max
# Tier:    default_claude_max_20x
# Token:   valid
# Expires: in 7h 24m
# Email:   alice@acme.com
# Org:     Acme Corp
# Display: alice
# Role:    admin
# Billing: stripe_subscription
# Model:   sonnet

clp .credentials.status format::json
# {"subscription":"max","tier":"default_claude_max_20x","token":"valid","expires_in_secs":26640,"email":"alice@acme.com","org":"Acme Corp","account":"alice@acme.com","file":"/home/user/.claude/.credentials.json","saved":2,"display_name":"alice","role":"admin","billing":"stripe_subscription","model":"sonnet"}
```

**Notes:**
- Field-presence params only affect text output. `format::json` always includes all fields (including `display_name`, `role`, `billing`, `model`) regardless of field-presence params.
- `account::` reads the `_active` marker; shows `N/A` on fresh installs without an account store.
- `saved::` counts `*.credentials.json` files in the credential store; shows `0` when the credential store is absent.
- `display_name::`, `role::`, `billing::` read from `~/.claude.json` `oauthAccount`; all show `N/A` when the file is absent.
- `model::` reads from `~/.claude/settings.json`; shows `N/A` when the file is absent or the `model` field is missing.

---

### Command :: 11. `.account.limits`

Show rate-limit utilization for the active or named account. Displays session (5h) usage, weekly all-model (7d) usage, and rate-limit status with percentage consumed and reset times.

-- **Parameters:** [`name::`](params.md#parameter--1-name) *(optional)*, [`v::`](params.md#parameter--2-verbosity--v), [`format::`](params.md#parameter--3-format)
-- **Exit:** 0 (success) | 1 (usage: invalid `name::` chars) | 2 (runtime: account not found, data unavailable, HOME unset)

**Syntax:**

```bash
clp .account.limits
clp .account.limits name::alice@acme.com
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
