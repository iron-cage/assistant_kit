# Commands

### All Commands (11 total)

| # | Command | Purpose | Params | Example |
|---|---------|---------|--------|---------|
| 1 | `.` | Show help information (hidden dot-shorthand) | 0 | `clp .` |
| 2 | `.help` | Display command reference and usage examples | 0 | `clp .help` |
| 3 | `.accounts` | List all saved accounts or show a single named account | 12 | `clp .accounts` |
| 4 | `.account.save` | Save current credentials as a named account profile | 2 | `clp .account.save name::alice@acme.com` |
| 5 | `.account.use` | Switch active account by name with atomic credential rotation | 2 | `clp .account.use name::alice@home.com` |
| 6 | `.account.delete` | Delete a saved account from the account store | 2 | `clp .account.delete name::alice@oldco.com` |
| 7 | `.token.status` | Show active OAuth token expiry classification | 2 | `clp .token.status` |
| 8 | `.paths` | Show all resolved ~/.claude/ canonical file paths | 1 | `clp .paths` |
| 9 | `.usage` | Show live rate-limit quota for all saved accounts | 6 | `clp .usage` |
| 10 | `.credentials.status` | Show live credential metadata without account store dependency | 13 | `clp .credentials.status` |
| 11 | `.account.limits` | Show rate-limit utilization for the active or named account | 2 | `clp .account.limits name::alice@acme.com` |

**Total:** 11 commands (9 visible + 2 hidden)

---

### Quick Reference

**Required Parameters:**
- `name::` — required on `.account.use`, `.account.delete` (must be an email address); optional on `.account.save` (inferred from `~/.claude.json` `emailAddress` when omitted). Also accepted as a bare positional argument (`clp .account.use alice@home.com`) or a prefix shortcut (`clp .account.use i3` resolves to the first saved account starting with `i3`).

**Most-Used Parameters:**
- `format::` — 6 commands

**Commands by Parameter Count:**

| Count | Commands |
|-------|----------|
| 0 | `.`, `.help` |
| 1 | `.paths` |
| 2 | `.account.save`, `.account.use`, `.account.delete`, `.token.status`, `.account.limits` |
| 5 | `.usage` |
| 12 | `.accounts` |
| 13 | `.credentials.status` |

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

### Command :: 1. `.`

Hidden alias that triggers `print_usage()` at the adapter layer — identical to typing `.help`.
The `.` command is registered in the registry with `hidden_from_list(true)` so it never
appears in its own output; its registered handler (`dot_routine`) is never invoked because
the adapter intercepts `.` before the unilang pipeline.

-- **Parameters:** none accepted (trailing `key::value` tokens are silently ignored)
-- **Exit:** 0 (always)

**Syntax:**

```bash
clp .
```

**Output (non-TTY, ANSI stripped):**

```
Usage: clp <command> [key::value ...]

Manage Claude Code account credentials and token state.

Commands:

  Account management
    .accounts             List all saved accounts
    .account.save         Save current credentials as a named profile
    .account.use          Switch the active account
    .account.delete       Delete a saved account
    .account.limits       Show rate-limit utilization (one account)

  Status & info
    .credentials.status   Show live credential metadata
    .token.status         Show OAuth token expiry classification
    .paths                Show all resolved ~/.claude/ paths
    .usage                Show live quota for all saved accounts

Options:
  format::text|json     Output format (default: text)
  dry::bool             Dry-run preview (no changes)
  name::EMAIL           Account name

Examples:
  clp .accounts
  clp .account.use alice@acme.com
  clp .usage
  clp .credentials.status
```

**Notes:**
- On a TTY, group headers and command names are rendered in ANSI colour (yellow/bold-cyan); piped output strips ANSI codes automatically (`std::io::IsTerminal`).
- Commands are grouped into "Account management" and "Status & info"; no per-command parameter listings are shown at this level of abstraction.
- Per-command parameter details are available via `clp <command>.help` (e.g., `clp .accounts.help`).
- Implemented in `src/lib.rs::cli::print_usage()`.

---

### Command :: 2. `.help`

Pre-registered by the unilang `CommandRegistry`. At the adapter layer, `.help` (and bare `help`) set `needs_help=true` which intercepts execution before the unilang pipeline, causing `print_usage()` to run — producing output byte-identical to `clp .`.

-- **Parameters:** none accepted (trailing `key::value` tokens are silently ignored)
-- **Exit:** 0 (always)

**Syntax:**

```bash
clp .help
clp help
```

**Notes:**
- Output is identical to `clp .` (both call `print_usage()`).
- `clp .help` is the canonical help invocation; `clp .` is a convenience alias.

---

### Command :: 3. `.accounts`

List all saved accounts or show a single named account with per-field presence control. Without `name::`: shows every account in the credential store as an indented key-val block; with `name::EMAIL`: shows that account's block only.

-- **Parameters:** [`name::`](params.md#parameter--1-name) *(optional)*, [`active::`](params.md#parameter--13-active), [`current::`](params.md#parameter--18-current), [`sub::`](params.md#parameter--6-sub), [`tier::`](params.md#parameter--7-tier), [`expires::`](params.md#parameter--9-expires), [`email::`](params.md#parameter--10-email), [`display_name::`](params.md#parameter--14-display_name), [`role::`](params.md#parameter--15-role), [`billing::`](params.md#parameter--16-billing), [`model::`](params.md#parameter--17-model), [`format::`](params.md#parameter--2-format)
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
| `name::` | [`AccountName`](types.md#type--1-accountname) | *(omit to list all)* | Show a single named account instead of listing all |
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
| `format::` | [`OutputFormat`](types.md#type--3-outputformat) | `text` | Output format |

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

clp .accounts name::alice@acme.com
# alice@acme.com
#   Active:  yes
#   Current: no
#   Sub:     max
#   Tier:    default_claude_max_20x
#   Expires: in 2h 11m
#   Email:   alice@acme.com

clp .accounts sub::0 tier::0 email::0
# alice@acme.com
#   Active:  yes
#   Current: no
#   Expires: in 2h 11m
#
# alice@home.com
#   Active:  no
#   Current: yes
#   Expires: in 5h 30m

clp .accounts active::0 current::0 sub::0 tier::0 expires::0 email::0
# alice@acme.com
# alice@home.com

clp .accounts display_name::1 role::1 billing::1 model::1
# alice@acme.com
#   Active:  yes
#   Current: no
#   Sub:     max
#   Tier:    default_claude_max_20x
#   Expires: in 2h 11m
#   Email:   alice@acme.com
#   Display: alice
#   Role:    admin
#   Billing: stripe_subscription
#   Model:   sonnet
#
# alice@home.com
#   Active:  no
#   Current: yes
#   Sub:     pro
#   Tier:    default_claude_pro
#   Expires: in 5h 30m
#   Email:   N/A
#   Display: N/A
#   Role:    N/A
#   Billing: N/A
#   Model:   N/A

clp .accounts format::json
# [{"name":"alice@acme.com","is_active":true,"is_current":false,"subscription_type":"max","rate_limit_tier":"default_claude_max_20x","expires_at_ms":1711234567000,"email":"alice@acme.com","display_name":"alice","role":"admin","billing":"stripe_subscription","model":"sonnet"},{"name":"alice@home.com","is_active":false,"is_current":true,"subscription_type":"pro","rate_limit_tier":"default_claude_pro","expires_at_ms":1711243567000,"email":"N/A","display_name":"N/A","role":"N/A","billing":"N/A","model":"N/A"}]

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
- With `name::EMAIL`: shows exactly one account's block — same format as listing.
- Field params affect text output only; `format::json` always includes all fields regardless of presence params.
- `format::table` renders a compact one-row-per-account table with fixed columns (flag, Account, Sub, Tier, Expires, Email) — field-presence params are ignored in table mode.
- Reports exit 1 for invalid `name::` value; exit 2 if the named account is not found.
- `email::`, `display_name::`, `role::`, `billing::` read from per-account saved `{name}.claude.json` snapshot; `model::` reads from `{name}.settings.json`. All show `N/A` when the snapshot file is absent (backward compatible with accounts saved before this feature).
- `current::` (default on): shows `Current:  yes` for the account whose `accessToken` matches `~/.claude/.credentials.json`; `Current:  no` for others. The line is suppressed entirely (regardless of the `current::` toggle) when `~/.claude/.credentials.json` is unreadable. See [feature/016_current_account_awareness.md](../feature/016_current_account_awareness.md).
- `format::json` includes `is_current` boolean per account object.

---

### Command :: 4. `.account.save`

Copies `~/.claude/.credentials.json` to `{credential_store}/{name}.credentials.json` and snapshots `~/.claude.json` and `~/.claude/settings.json` as named per-account files, creating the credential store directory if needed. Use this to preserve the full current account state before switching.

-- **Parameters:** [`name::`](params.md#parameter--1-name), [`dry::`](params.md#parameter--4-dry)
-- **Exit:** 0 (success) | 1 (usage: invalid name or cannot infer email) | 2 (runtime: credentials unreadable)

**Syntax:**

```bash
clp .account.save                          # infer name from ~/.claude.json emailAddress
clp .account.save name::alice@acme.com    # explicit name
clp .account.save name::alice@acme.com dry::1
```

| Parameter | Type | Default | Purpose |
|-----------|------|---------|---------|
| `name::` | [`AccountName`] | `auto` (inferred from `~/.claude.json` `emailAddress`) | Account email to save as |
| `dry::` | `bool` | `0` | Preview action without executing |

**Examples:**

```bash
clp .account.save
# saved current credentials as 'alice@acme.com'   (inferred from ~/.claude.json)

clp .account.save name::alice@acme.com
# saved current credentials as 'alice@acme.com'

clp .account.save name::alice@acme.com dry::1
# [dry-run] would save current credentials as 'alice@acme.com'
```

**Notes:**

- Also writes `{credential_store}/_active` = `{name}` on every successful save; `.credentials.status` shows `Account: {name}` immediately after save without requiring a separate `.account.use` call.

---

### Command :: 5. `.account.use`

Atomically overwrites `~/.claude/.credentials.json` with the named account's credentials (write-then-rename), updates the `_active` marker, and best-effort restores the account's `~/.claude.json` and `~/.claude/settings.json` snapshots. Use this to rotate to a different account when the current token expires.

-- **Parameters:** [`name::`](params.md#parameter--1-name) **(required)**, [`dry::`](params.md#parameter--4-dry)
-- **Exit:** 0 (success) | 1 (usage: invalid name) | 2 (runtime: account not found)

**Syntax:**

```bash
clp .account.use name::alice@home.com
clp .account.use alice@home.com          # positional: same as name::alice@home.com
clp .account.use i3                      # prefix: first saved account starting with "i3"
clp .account.use name::alice@home.com dry::1
clp .account.use alice@home.com dry::1
```

| Parameter | Type | Default | Purpose |
|-----------|------|---------|---------|
| `name::` | [`AccountName`] | **(required)** | Account email to switch to |
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

Removes `{credential_store}/{name}.credentials.json` from the credential store and best-effort removes the accompanying `{name}.claude.json` and `{name}.settings.json` snapshot files. When the deleted account is active, the `_active` marker is also removed.

-- **Parameters:** [`name::`](params.md#parameter--1-name) **(required)**, [`dry::`](params.md#parameter--4-dry)
-- **Exit:** 0 (success) | 1 (usage: invalid name) | 2 (runtime: account not found)

**Syntax:**

```bash
clp .account.delete name::alice@oldco.com
clp .account.delete alice@oldco.com         # positional: same as name::alice@oldco.com
clp .account.delete i3                      # prefix: first saved account starting with "i3"
clp .account.delete name::alice@oldco.com dry::1
clp .account.delete alice@oldco.com dry::1
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
# deleted account 'alice@acme.com'
```

**Notes:**
- Snapshot files (`{name}.claude.json`, `{name}.settings.json`) are removed best-effort: missing snapshots are silently skipped — the operation still succeeds.
- Deleting the active account also removes the `_active` marker, leaving no active account. Use `.account.use` or `.account.save` to restore an active account.

---

### Command :: 7. `.token.status`

Reads `expiresAt` from `~/.claude/.credentials.json` and classifies the active OAuth token as Valid, ExpiringSoon, or Expired. Use this to detect when account rotation is needed.

-- **Parameters:** [`format::`](params.md#parameter--2-format), [`threshold::`](params.md#parameter--3-threshold)
-- **Exit:** 0 (success) | 2 (runtime: credentials unreadable, expiresAt unparseable)

**Syntax:**

```bash
clp .token.status
clp .token.status threshold::1800
clp .token.status format::json
```

| Parameter | Type | Default | Purpose |
|-----------|------|---------|---------|
| `format::` | [`OutputFormat`] | `text` | Output format |
| `threshold::` | [`WarningThreshold`] | `3600` | ExpiringSoon threshold in seconds |

**Examples:**

```bash
clp .token.status
# valid — 47m remaining

clp .token.status threshold::1800
# expiring soon — 25m remaining

clp .token.status format::json
# {"status":"valid","expires_in_secs":2820}
```

---

### Command :: 8. `.paths`

Displays all canonical `~/.claude/` file and directory paths resolved from `HOME`. Use this for diagnostics and tooling integration.

-- **Parameters:** [`format::`](params.md#parameter--2-format)
-- **Exit:** 0 (success) | 2 (runtime: HOME not set)

**Syntax:**

```bash
clp .paths
clp .paths format::json
```

| Parameter | Type | Default | Purpose |
|-----------|------|---------|---------|
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

clp .paths format::json
# {"base":"/home/user/.claude","credentials":"/home/user/.claude/.credentials.json",...}
```

---

### Command :: 9. `.usage`

Fetches live quota utilization for every saved account via `claude_quota::fetch_oauth_usage()` (`GET /api/oauth/usage`). Renders results as a `data_fmt` table with per-account Expires, 5h Left, 5h Reset, 7d Left, 7d(Son), and 7d Reset columns, plus a footer recommendation line. Supports optional token refresh on auth errors (`refresh::1`) and continuous live-monitor mode (`live::1`).

-- **Parameters:** [`format::`](params.md#parameter--2-format), [`refresh::`](params.md#parameter--19-refresh), [`live::`](params.md#parameter--20-live), [`interval::`](params.md#parameter--21-interval), [`jitter::`](params.md#parameter--22-jitter), [`trace::`](params.md#parameter--23-trace)
-- **Exit:** 0 (success) | 1 (usage: invalid param combination) | 2 (runtime: credential store unreadable, HOME unset)

**Syntax:**

```bash
clp .usage
clp .usage format::json
clp .usage refresh::1
clp .usage live::1
clp .usage live::1 interval::60 jitter::10
clp .usage live::1 refresh::1 interval::60
clp .usage refresh::1 trace::1
```

| Parameter | Type | Default | Purpose |
|-----------|------|---------|---------|
| `format::` | [`OutputFormat`](types.md#type--3-outputformat) | `text` | Output format (`text` or `json`; `json` incompatible with `live::1`) |
| `refresh::` | `bool` | `0` | On 401/403 auth error, refresh token via isolated subprocess and retry |
| `live::` | `bool` | `0` | Enable continuous refresh loop (Ctrl-C to exit) |
| `interval::` | `u64` | `30` | Seconds between refresh cycles (≥ 30; only validated when `live::1`) |
| `jitter::` | `u64` | `0` | Max random seconds added to each cycle delay (≤ interval; only validated when `live::1`) |
| `trace::` | `bool` | `0` | Print `[trace]` lines to stderr: credential reads, API calls, and refresh steps |

**Examples:**

```bash
clp .usage
# Quota
#
#   Account          Expires     5h Left  5h Reset    7d Left  7d(Son)  7d Reset
# ✓ i12@wbox.pro    in 7h 24m  86%      in 3h 19m  65%      35%      in 4d 23h
# → i6@wbox.pro     in 5h 02m  100%     in 4h 58m  88%      28%      in 6d 14h
#   i7@wbox.pro     EXPIRED    —        —           —        —        (missing accessToken)
#
# Valid: 2 / 3   →  Next: i6@wbox.pro  (100% session left, token expires in 5h 02m)

clp .usage format::json
# [
#   {"account":"i12@wbox.pro","is_current":true,"is_active":false,"expires_in_secs":26640,"session_5h_left_pct":86,"session_5h_resets_in_secs":11940,"weekly_7d_left_pct":65,"weekly_7d_sonnet_left_pct":35,"weekly_7d_resets_in_secs":432540},
#   {"account":"i6@wbox.pro","is_current":false,"is_active":true,"expires_in_secs":18120,"session_5h_left_pct":100,"session_5h_resets_in_secs":17880,"weekly_7d_left_pct":88,"weekly_7d_sonnet_left_pct":28,"weekly_7d_resets_in_secs":500040},
#   {"account":"i7@wbox.pro","is_current":false,"is_active":false,"expires_in_secs":0,"error":"missing accessToken"}
# ]

clp .usage refresh::1
# (same table as above; expired tokens silently refreshed before fetch, invisible to user)

clp .usage live::1 interval::60 jitter::10
# Quota
# ...table...
#
#   Next update in 0:59 (at 14:32:07 UTC)  [Ctrl-C to exit]
# (refreshes every 60–70 seconds; Ctrl-C exits cleanly)
```

**Notes:**
- Accounts are enumerated from `{credential_store}/*.credentials.json` in alphabetical order.
- Flag column priority: `✓` = current account (live `accessToken` match), `*` = `_active`-but-not-current (divergence indicator), `→` = recommended next account (highest remaining session quota among non-current accounts with valid quota data and a non-expired token), ` ` = none. When current = active, only `✓` appears; `*` is suppressed. See [feature/016_current_account_awareness.md](../feature/016_current_account_awareness.md).
- `Expires` is sourced from `expiresAt` in the credential file — available even when the API call fails. Shows "in Xh Ym" or "EXPIRED".
- `5h Left` / `7d Left` show remaining quota percentage (100 − consumed); `7d(Son)` shows remaining Sonnet-only weekly quota or `—` when unavailable. `5h Reset` / `7d Reset` are independent countdown columns. All quota data sourced from `claude_quota::fetch_oauth_usage()` → `/api/oauth/usage`.
- Accounts with expired or missing `accessToken` show `—` for quota columns and a shortened error reason in the last column; other accounts continue processing (per-account errors are non-fatal).
- Footer "Valid: X / Y   →  Next: ..." appears when ≥2 accounts have valid quota data; omitted when 0 or 1. In live mode this footer is followed by the countdown line.
- Empty credential store exits 0 with `(no accounts configured)`.
- `refresh::1` triggers at most one retry per account per cycle; only HTTP 401/403 errors trigger the subprocess. See [feature/017_token_refresh.md](../feature/017_token_refresh.md).
- `live::1 format::json` exits 1 before any fetch. `interval::` and `jitter::` are validated only when `live::1`. See [feature/018_live_monitor.md](../feature/018_live_monitor.md).
- `refresh::` and `live::1` are composable — token refresh runs on every cycle in live mode.
- See [feature/009_token_usage.md](../feature/009_token_usage.md) for the baseline algorithm and AC criteria.

---

### Command :: 10. `.credentials.status`

Show live credential metadata by reading `~/.claude/.credentials.json` directly. Succeeds on any authenticated machine regardless of whether account store setup exists.

-- **Parameters:** [`format::`](params.md#parameter--2-format), [`account::`](params.md#parameter--5-account), [`sub::`](params.md#parameter--6-sub), [`tier::`](params.md#parameter--7-tier), [`token::`](params.md#parameter--8-token), [`expires::`](params.md#parameter--9-expires), [`email::`](params.md#parameter--10-email), [`file::`](params.md#parameter--11-file), [`saved::`](params.md#parameter--12-saved), [`display_name::`](params.md#parameter--14-display_name), [`role::`](params.md#parameter--15-role), [`billing::`](params.md#parameter--16-billing), [`model::`](params.md#parameter--17-model)
-- **Exit:** 0 (success) | 2 (credential file absent or HOME unset)

**Syntax:**

```bash
clp .credentials.status
clp .credentials.status email::0
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

clp .credentials.status email::0
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
# File:    /home/user/.claude/.credentials.json
# Saved:   2 account(s)

clp .credentials.status display_name::1 role::1 billing::1 model::1
# Account: alice@acme.com
# Sub:     max
# Tier:    default_claude_max_20x
# Token:   valid
# Expires: in 7h 24m
# Email:   alice@acme.com
# Display: alice
# Role:    admin
# Billing: stripe_subscription
# Model:   sonnet

clp .credentials.status format::json
# {"subscription":"max","tier":"default_claude_max_20x","token":"valid","expires_in_secs":26640,"email":"alice@acme.com","account":"alice@acme.com","file":"/home/user/.claude/.credentials.json","saved":2,"display_name":"alice","role":"admin","billing":"stripe_subscription","model":"sonnet"}
```

**Notes:**
- Field-presence params only affect text output. `format::json` always includes all fields (including `display_name`, `role`, `billing`, `model`) regardless of field-presence params.
- `account::` reads the `_active` marker; shows `N/A` on machines where no account has ever been saved.
- `saved::` counts `*.credentials.json` files in the credential store; shows `0` when the credential store is absent.
- `display_name::`, `role::`, `billing::` read from `~/.claude.json` `oauthAccount`; all show `N/A` when the file is absent.
- `model::` reads from `~/.claude/settings.json`; shows `N/A` when the file is absent or the `model` field is missing.

---

### Command :: 11. `.account.limits`

Show rate-limit utilization for the active or named account. Displays session (5h) usage, weekly all-model (7d) usage, and rate-limit status with percentage consumed and reset times.

-- **Parameters:** [`name::`](params.md#parameter--1-name) *(optional)*, [`format::`](params.md#parameter--2-format)
-- **Exit:** 0 (success) | 1 (usage: invalid `name::` chars) | 2 (runtime: account not found, data unavailable, HOME unset)

**Syntax:**

```bash
clp .account.limits
clp .account.limits name::alice@acme.com
clp .account.limits alice@acme.com       # positional: same as name::alice@acme.com
clp .account.limits i3                   # prefix: first saved account starting with "i3"
clp .account.limits format::json
```

| Parameter | Type | Default | Purpose |
|-----------|------|---------|---------|
| `name::` | [`AccountName`] | *(omit for active)* | Query a named account instead of the active account |
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
- Data source: `anthropic-ratelimit-unified-*` response headers from a lightweight API call; see [feature/013_account_limits.md](../feature/013_account_limits.md). Transport: `claude_quota::fetch_rate_limits()`.
- With `name::`: shows limits for the named account (requires account store entry).
- Without `name::`: shows limits for the active account.
