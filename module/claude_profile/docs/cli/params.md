# Parameters

### All Parameters (24 total)

| # | Parameter | Type | Default | Valid Values | Purpose | Used In |
|---|-----------|------|---------|--------------|---------|---------|
| 1 | `name::` | `AccountName` | Varies | Email or prefix | Account email or prefix shortcut for use/delete (required); save (optional, inferred from `~/.claude.json`); accounts/limits query (optional); bare positional arg accepted after command name | 5 cmds |
| 2 | `format::` / `fmt::` | `OutputFormat` | `text` | `text`, `json` | Output format: `text` or `json` | 6 cmds |
| 3 | `threshold::` | `WarningThreshold` | `3600` | Non-negative integer (seconds) | Seconds before token expiry to classify as ExpiringSoon | 1 cmd |
| 4 | `dry::` | `bool` | `0` | `0`, `1`, `false`, `true` | Print intended action without executing | 3 cmds |
| 5 | `account::` | `bool` | `1` | `0`, `1` | Show active account name line (`.credentials.status`) | 1 cmd |
| 6 | `sub::` | `bool` | `1` | `0`, `1` | Show subscription type line | 2 cmds |
| 7 | `tier::` | `bool` | `1` | `0`, `1` | Show rate-limit tier line | 2 cmds |
| 8 | `token::` | `bool` | `1` | `0`, `1` | Show token status line (`.credentials.status`) | 1 cmd |
| 9 | `expires::` | `bool` | `1` | `0`, `1` | Show token expiry duration line | 2 cmds |
| 10 | `email::` | `bool` | `1` | `0`, `1` | Show email address line | 2 cmds |
| 11 | `file::` | `bool` | `0` | `0`, `1` | Show credentials file path, opt-in (`.credentials.status`) | 1 cmd |
| 12 | `saved::` | `bool` | `0` | `0`, `1` | Show saved account count, opt-in (`.credentials.status`) | 1 cmd |
| 13 | `active::` | `bool` | `1` | `0`, `1` | Show active/inactive status line (`.accounts`) | 1 cmd |
| 14 | `display_name::` | `bool` | `0` | `0`, `1` | Show display name from `oauthAccount`, opt-in | 2 cmds |
| 15 | `role::` | `bool` | `0` | `0`, `1` | Show organisation role from `oauthAccount`, opt-in | 2 cmds |
| 16 | `billing::` | `bool` | `0` | `0`, `1` | Show billing type from `oauthAccount`, opt-in | 2 cmds |
| 17 | `model::` | `bool` | `0` | `0`, `1` | Show active model from settings, opt-in | 2 cmds |
| 18 | `current::` | `bool` | `1` | `0`, `1` | Show current (live) account line in `.accounts`; suppressed when `~/.claude/.credentials.json` is unreadable | 1 cmd |
| 19 | `refresh::` | `bool` | `1` | `0`, `1` | On auth error (401/403) or 429 with locally-expired token, refresh via isolated subprocess and retry quota fetch (`.usage`); default on | 1 cmd |
| 20 | `live::` | `bool` | `0` | `0`, `1` | Enable continuous refresh loop in `.usage`; incompatible with `format::json` | 1 cmd |
| 21 | `interval::` | `u64` | `30` | ≥ 30 (seconds) | Seconds between full refresh cycles in live mode; validated only when `live::1` | 1 cmd |
| 22 | `jitter::` | `u64` | `0` | 0 ≤ jitter ≤ interval | Max random seconds added to `interval` in live mode; validated only when `live::1` | 1 cmd |
| 23 | `trace::` | `bool` | `0` | `0`, `1` | Print `[trace]` lines to stderr: credential reads, API calls, and refresh steps (`.usage`) | 1 cmd |
| 24 | `field::` | `String` | `""` (show all) | `base`, `credentials`, `credential_store`, `projects`, `stats`, `settings`, `session_env`, `sessions` | Output a single named path value from `.paths`; overrides `format::` | 1 cmd |

**Total:** 24 parameters

*Parameter 2 forms the Output Control group; parameters 5-18 form the Field Presence group; parameters 19-23 form the Fetch Behavior group; parameter 24 forms the Output Selection group*

---

### Parameter :: 1. `name::`

Identifies the target account. Accepted as an explicit `name::EMAIL` pair, as a bare positional argument after the command name (no `name::` prefix required), or as a prefix shortcut (no `@`) that resolves to the first saved account whose name starts with that value.

- **Type:** `AccountName`
- **Default:** **(required)** on `.account.use`, `.account.delete`; **inferred** on `.account.save` (reads `emailAddress` from `~/.claude.json`; exits 1 if absent); **optional** on `.accounts` (omit to list all) and `.account.limits` (omit for active account)
- **Constraints:** Resolved value must be a valid email address (non-empty, must contain `@`, non-empty local part and domain); local part must not contain `/`, `\`, or `*` (path-unsafe characters rejected before any filesystem operation). Prefix input (no `@`) must be unambiguous — exits 1 when multiple saved accounts share the prefix.
- **Positional syntax:** On `.accounts`, `.account.use`, `.account.delete`, and `.account.limits` a bare argument after the command name is treated as the `name::` value. `clp .account.use alice@home.com` is equivalent to `clp .account.use name::alice@home.com`.
- **Prefix resolution:** When the supplied value contains no `@`, it is matched as a prefix against saved account names. The first alphabetically sorted match is used. If zero or multiple accounts match, the command exits 1 with a disambiguation error.
- **Commands:** [`.accounts`](commands.md#command--3-accounts) *(optional)*, [`.account.save`](commands.md#command--4-accountsave) *(optional/inferred)*, [`.account.use`](commands.md#command--5-accountuse), [`.account.delete`](commands.md#command--6-accountdelete), [`.account.limits`](commands.md#command--11-accountlimits) *(optional)*
- **Purpose:** Selects the target credential file at `{credential_store}/{email}.credentials.json`. Name validation matches the library's `account::validate_name()` rules. An invalid name exits 1; a valid but unknown name exits 2.

**Examples:**

```text
name::alice@acme.com   → explicit form → {credential_store}/alice@acme.com.credentials.json
alice@acme.com         → positional form (bare arg after command) → same as above
alice                  → prefix form → resolves to first saved account starting with "alice"
i3                     → prefix form → resolves to e.g. i3@wbox.pro
```

---

### Parameter :: 2. `format::` / `fmt::`

Selects between human-readable text output and machine-parseable JSON. Text is the default for interactive use; JSON enables pipeline integration.

- **Type:** `OutputFormat`
- **Default:** `text`
- **Alias:** `fmt::` (short form; both accepted at runtime)
- **Constraints:** One of `text`, `json`, `table` (case-insensitive); `table` accepted only on `.accounts`
- **Commands:** [`.accounts`](commands.md#command--3-accounts), [`.token.status`](commands.md#command--7-tokenstatus), [`.paths`](commands.md#command--8-paths), [`.usage`](commands.md#command--9-usage), [`.credentials.status`](commands.md#command--10-credentialsstatus), [`.account.limits`](commands.md#command--11-accountlimits)
- **Purpose:** Enables CLI composability — `format::json` output can be piped to `jq` for structured extraction without parsing fragile text layouts.
- **Group:** Output Control


**Examples:**

```text
format::text   → human-readable labeled output (default)
format::json   → JSON object or array
fmt::json      → same as format::json (short alias)
format::table  → compact one-row-per-account table (.accounts only)
```

---

### Parameter :: 3. `threshold::`

Overrides the default 60-minute warning window for token expiry classification. Tokens expiring within `threshold::` seconds are classified as `ExpiringSoon` instead of `Valid`.

- **Type:** `WarningThreshold`
- **Default:** `3600` (60 minutes, matching `token::WARNING_THRESHOLD_SECS`)
- **Constraints:** Non-negative integer (seconds)
- **Commands:** [`.token.status`](commands.md#command--7-tokenstatus)
- **Purpose:** Allows callers to tune the early-warning sensitivity — automation scripts may want `threshold::7200` (2 hours) for proactive rotation, while interactive users may prefer the default 60 minutes.

**Examples:**

```text
threshold::3600   → classify as ExpiringSoon when <=60 minutes remain (default)
threshold::1800   → classify as ExpiringSoon when <=30 minutes remain
threshold::7200   → classify as ExpiringSoon when <=2 hours remain
threshold::0      → never classify as ExpiringSoon (only Valid or Expired)
```

---

### Parameter :: 4. `dry::`

Activates simulation mode for mutation commands. When `dry::1`, the command prints what it *would* do without modifying any files. Part of the standard CLI dry-run pattern.

- **Type:** `bool` (`0` / `1` or `false` / `true`)
- **Default:** `0` (execute normally)
- **Constraints:** Accepted values: `0`, `1`, `false`, `true`
- **Commands:** [`.account.save`](commands.md#command--4-accountsave), [`.account.use`](commands.md#command--5-accountuse), [`.account.delete`](commands.md#command--6-accountdelete)
- **Purpose:** Lets users preview credential file changes before committing. Critical for account management where an accidental switch or delete could disrupt active sessions.

**Examples:**

```text
dry::1     → print intended action, skip execution
dry::0     → execute normally (default)
dry::true  → same as dry::1
dry::false → same as dry::0
```

**Notes:**
- Dry-run output uses `[dry-run]` prefix for clear visual distinction.
- Dry and execute modes share identical validation logic — if `dry::1` succeeds, `dry::0` will perform exactly those actions.

---

### Parameter :: 5. `account::`

Controls whether the active account name line appears in `.credentials.status` output. Reads the `_active` marker file; shows `N/A` when no account store has been initialised.

- **Type:** `bool`
- **Default:** `1` (shown)
- **Constraints:** Accepted values: `0`, `1`, `false`, `true`
- **Commands:** [`.credentials.status`](commands.md#command--10-credentialsstatus)
- **Purpose:** Lets callers suppress the account name line when it is irrelevant (e.g., scripting that only needs the token state).
- **Group:** Field Presence

**Examples:**

```text
account::1   → Account: alice@acme.com  (default)
account::0   → line omitted
```

---

### Parameter :: 6. `sub::`

Controls whether the subscription type line appears in output. Used by both `.accounts` (per stored credential) and `.credentials.status` (from live credentials).

- **Type:** `bool`
- **Default:** `1` (shown)
- **Constraints:** Accepted values: `0`, `1`, `false`, `true`
- **Commands:** [`.accounts`](commands.md#command--3-accounts), [`.credentials.status`](commands.md#command--10-credentialsstatus)
- **Purpose:** Allows suppression of the subscription type when only token validity or account name matters.
- **Group:** Field Presence

**Examples:**

```text
sub::1   → Sub:     max  (default)
sub::0   → line omitted
```

---

### Parameter :: 7. `tier::`

Controls whether the rate-limit tier line appears in output. Used by both `.accounts` (per stored credential) and `.credentials.status` (from live credentials).

- **Type:** `bool`
- **Default:** `1` (shown)
- **Constraints:** Accepted values: `0`, `1`, `false`, `true`
- **Commands:** [`.accounts`](commands.md#command--3-accounts), [`.credentials.status`](commands.md#command--10-credentialsstatus)
- **Purpose:** Allows suppression of the tier when only core token state is needed.
- **Group:** Field Presence

**Examples:**

```text
tier::1   → Tier:    default_claude_max_20x  (default)
tier::0   → line omitted
```

---

### Parameter :: 8. `token::`

Controls whether the token validity status line appears in `.credentials.status` output.

- **Type:** `bool`
- **Default:** `1` (shown)
- **Constraints:** Accepted values: `0`, `1`, `false`, `true`
- **Commands:** [`.credentials.status`](commands.md#command--10-credentialsstatus)
- **Purpose:** Allows suppression of the token status line (rare; usually the most important field).
- **Group:** Field Presence

**Examples:**

```text
token::1   → Token:   valid  (default)
token::0   → line omitted
```

---

### Parameter :: 9. `expires::`

Controls whether the token expiry duration line appears in output. Used by both `.accounts` (per stored credential) and `.credentials.status` (from live credentials).

- **Type:** `bool`
- **Default:** `1` (shown)
- **Constraints:** Accepted values: `0`, `1`, `false`, `true`
- **Commands:** [`.accounts`](commands.md#command--3-accounts), [`.credentials.status`](commands.md#command--10-credentialsstatus)
- **Purpose:** Allows suppression of the expiry countdown when exact timing is not needed.
- **Group:** Field Presence

**Examples:**

```text
expires::1   → Expires: in 7h 24m  (default)
expires::0   → line omitted
```

---

### Parameter :: 10. `email::`

Controls whether the email address line appears in output. Source for `.credentials.status`: `emailAddress` field in live `~/.claude.json`. Source for `.accounts`: `emailAddress` field in saved `{name}.claude.json` snapshot.

- **Type:** `bool`
- **Default:** `1` (shown)
- **Constraints:** Accepted values: `0`, `1`, `false`, `true`
- **Commands:** [`.accounts`](commands.md#command--3-accounts), [`.credentials.status`](commands.md#command--10-credentialsstatus)
- **Purpose:** Allows suppression of the email line; shows `N/A` when the source file is absent or `emailAddress` is empty.
- **Group:** Field Presence

**Examples:**

```text
email::1   → Email:   alice@acme.com  (default; N/A when absent)
email::0   → line omitted
```

---

### Parameter :: 11. `file::`

Controls whether the credentials file path line appears in `.credentials.status` output. Opt-in (default `0`).

- **Type:** `bool`
- **Default:** `0` (hidden)
- **Constraints:** Accepted values: `0`, `1`, `false`, `true`
- **Commands:** [`.credentials.status`](commands.md#command--10-credentialsstatus)
- **Purpose:** Exposes the resolved path to `~/.claude/.credentials.json` for diagnostics and tooling integration.
- **Group:** Field Presence

**Examples:**

```text
file::0   → line omitted  (default)
file::1   → File:    /home/user/.claude/.credentials.json
```

---

### Parameter :: 12. `saved::`

Controls whether the saved account count line appears in `.credentials.status` output. Opt-in (default `0`). Counts `*.credentials.json` files in the credential store.

- **Type:** `bool`
- **Default:** `0` (hidden)
- **Constraints:** Accepted values: `0`, `1`, `false`, `true`
- **Commands:** [`.credentials.status`](commands.md#command--10-credentialsstatus)
- **Purpose:** Shows how many credential profiles are saved in the credential store; shows `0` when the credential store is absent.
- **Group:** Field Presence

**Examples:**

```text
saved::0   → line omitted  (default)
saved::1   → Saved:   3 account(s)
```

---

### Parameter :: 13. `active::`

Controls whether the active/inactive status line appears in `.accounts` output for each account entry.

- **Type:** `bool`
- **Default:** `1` (shown)
- **Constraints:** Accepted values: `0`, `1`, `false`, `true`
- **Commands:** [`.accounts`](commands.md#command--3-accounts)
- **Purpose:** Shows whether each listed account is currently active. When listing multiple accounts, `active::0` suppresses the status lines to show only the remaining fields.
- **Group:** Field Presence

**Examples:**

```text
active::1   → Active:  yes  (default; or "no" for non-active accounts)
active::0   → line omitted
```

---

### Parameter :: 14. `display_name::`

Controls whether the display name line appears in output. Opt-in (default `0`). Source: `displayName` field in `oauthAccount` — read from live `~/.claude.json` (`.credentials.status`) or from the saved `{name}.claude.json` snapshot (`.accounts`).

- **Type:** `bool`
- **Default:** `0` (hidden)
- **Constraints:** Accepted values: `0`, `1`, `false`, `true`
- **Commands:** [`.accounts`](commands.md#command--3-accounts), [`.credentials.status`](commands.md#command--10-credentialsstatus)
- **Purpose:** Exposes the human-readable display name set by the OAuth account. Shows `N/A` when the source file is absent or the field is missing.
- **Group:** Field Presence

**Examples:**

```text
display_name::0   → line omitted  (default)
display_name::1   → Display: alice
```

---

### Parameter :: 15. `role::`

Controls whether the organisation role line appears in output. Opt-in (default `0`). Source: `organizationRole` field in `oauthAccount` — read from live `~/.claude.json` (`.credentials.status`) or from the saved `{name}.claude.json` snapshot (`.accounts`).

- **Type:** `bool`
- **Default:** `0` (hidden)
- **Constraints:** Accepted values: `0`, `1`, `false`, `true`
- **Commands:** [`.accounts`](commands.md#command--3-accounts), [`.credentials.status`](commands.md#command--10-credentialsstatus)
- **Purpose:** Shows the OAuth account's role within its organisation (e.g., `admin`, `member`). Shows `N/A` when the source file is absent or the field is missing.
- **Group:** Field Presence

**Examples:**

```text
role::0   → line omitted  (default)
role::1   → Role:    admin
```

---

### Parameter :: 16. `billing::`

Controls whether the billing type line appears in output. Opt-in (default `0`). Source: `billingType` field in `oauthAccount` — read from live `~/.claude.json` (`.credentials.status`) or from the saved `{name}.claude.json` snapshot (`.accounts`).

- **Type:** `bool`
- **Default:** `0` (hidden)
- **Constraints:** Accepted values: `0`, `1`, `false`, `true`
- **Commands:** [`.accounts`](commands.md#command--3-accounts), [`.credentials.status`](commands.md#command--10-credentialsstatus)
- **Purpose:** Shows the raw billing type string (e.g., `stripe_subscription`). Shows `N/A` when the source file is absent or the field is missing.
- **Group:** Field Presence

**Examples:**

```text
billing::0   → line omitted  (default)
billing::1   → Billing: stripe_subscription
```

---

### Parameter :: 17. `model::`

Controls whether the active model line appears in output. Opt-in (default `0`). Source: `model` field in `settings.json` — read from live `~/.claude/settings.json` (`.credentials.status`) or from the saved `{name}.settings.json` snapshot (`.accounts`).

- **Type:** `bool`
- **Default:** `0` (hidden)
- **Constraints:** Accepted values: `0`, `1`, `false`, `true`
- **Commands:** [`.accounts`](commands.md#command--3-accounts), [`.credentials.status`](commands.md#command--10-credentialsstatus)
- **Purpose:** Shows the model currently selected in Claude Code settings. Shows `N/A` when the source file is absent or the `model` field is missing.
- **Group:** Field Presence

**Examples:**

```text
model::0   → line omitted  (default)
model::1   → Model:   sonnet
```

---

### Parameter :: 18. `current::`

Controls whether the current (live) account line appears in `.accounts` output for each account entry. The current account is the saved account whose `accessToken` matches the live `~/.claude/.credentials.json` file — distinct from the active account (`_active` marker). See [feature/016_current_account_awareness.md](../feature/016_current_account_awareness.md).

- **Type:** `bool`
- **Default:** `1` (shown)
- **Constraints:** Accepted values: `0`, `1`, `false`, `true`; the line is always suppressed when `~/.claude/.credentials.json` is absent or unreadable regardless of the toggle value
- **Commands:** [`.accounts`](commands.md#command--3-accounts)
- **Purpose:** Indicates which saved account corresponds to the credentials currently loaded by Claude Code. When current ≠ active (divergence), both `Active:  yes` and `Current: no` appear on the `_active` account row, and `Active:  no` / `Current: yes` appear on the current account row.
- **Group:** Field Presence

**Examples:**

```text
current::1   → Current: yes  (default; or "no" for accounts not matching live token)
current::0   → line omitted
```

**Notes:**
- When `~/.claude/.credentials.json` is unreadable, the `Current:` line is suppressed for all accounts (equivalent to `current::0`). This prevents misleading `Current: no` output when the live token cannot be determined.
- `format::json` always includes `is_current` per account object regardless of this toggle.

---

### Parameter :: 19. `refresh::`

When an account's quota fetch returns an HTTP auth error (401 or 403), or an HTTP 429 rate-limit error when the per-account credential file has a locally-expired `expiresAt`, silently attempt a token refresh via `claude_profile_core::account::refresh_account_token()` and retry the fetch once before reporting failure.

- **Type:** `bool`
- **Default:** `1` (on — expired tokens silently refreshed before reporting failure)
- **Constraints:** Accepted values: `0`, `1`, `false`, `true`; effective only under `#[cfg(feature = "enabled")]` — in offline builds the parameter is accepted but has no effect
- **Commands:** [`.usage`](commands.md#command--9-usage)
- **Purpose:** Allows `.usage` to silently recover expired OAuth tokens without requiring a manual `clp .account.use` rotation, so the table shows current quota rather than per-account auth error rows.
- **Group:** Fetch Behavior

**Examples:**

```text
refresh::1   → on 401/403 auth error, attempt token refresh via isolated subprocess, then retry once (default)
refresh::0   → auth errors appear as error rows in the table (explicit disable)
```

**Notes:**
- HTTP 401 and 403 always trigger a refresh attempt. HTTP 429 triggers a refresh only when the per-account credential file has a locally-expired `expiresAt` (`expiresAt / 1000 ≤ now`) — this recovers accounts where Claude Code updated the live session file but the saved per-account copy was never re-saved, leaving a stale token. HTTP 429 with a non-expired local token is passed through as-is (the token is valid; no refresh needed).
- The refresh may silently have no effect when: (a) the token is not actually server-expired (claude detects no need to refresh), (b) `run_isolated` times out before credentials are updated, or (c) the refreshToken itself is also expired. Use `trace::1` to see exactly which step stopped the refresh for each account.
- Network timeouts and other non-auth/non-ratelimit errors are not retried — they pass through as error rows in the table.
- Exactly one retry per account per invocation. If the retried fetch also fails, the final error is shown in the account's row.
- The updated credential JSON is written back to the per-account credential file (`{credential_store}/{account}.credentials.json`); the shared live session file (`~/.claude/.credentials.json`) is not touched.

---

### Parameter :: 20. `live::`

Enables continuous refresh mode for `.usage`. When `live::1`, the command enters a loop: fetch all accounts, clear the screen, render the table, display a countdown footer, wait `interval::` seconds (plus up to `jitter::` seconds), then repeat. Ctrl-C exits cleanly.

- **Type:** `bool`
- **Default:** `0` (single-shot — fetch once, render, exit)
- **Constraints:** Accepted values: `0`, `1`, `false`, `true`; incompatible with `format::json` (exits 1 before first fetch if combined); effective only under `#[cfg(feature = "enabled")]`
- **Commands:** [`.usage`](commands.md#command--9-usage)
- **Purpose:** Provides an ambient monitoring dashboard showing live quota utilization for all accounts, refreshing automatically without re-invoking the command.
- **Group:** Fetch Behavior

**Examples:**

```text
live::0   → single fetch, render, exit (default)
live::1   → continuous refresh loop until Ctrl-C
```

**Notes:**
- `live::1 format::json` exits 1 before any fetch with `"live monitor mode is incompatible with format::json"`.
- `interval::` and `jitter::` are only validated when `live::1` is present.
- See [feature/018_live_monitor.md](../feature/018_live_monitor.md) for the full algorithm including screen-clear sequence and countdown footer format.

---

### Parameter :: 21. `interval::`

Sets the number of seconds between full refresh cycles in live mode. Ignored (and not validated) when `live::0`.

- **Type:** `u64`
- **Default:** `30` (seconds)
- **Constraints:** Must be ≥ 30 when `live::1`; values < 30 exit 1 with `"interval must be >= 30"`
- **Commands:** [`.usage`](commands.md#command--9-usage)
- **Purpose:** Controls how frequently the live quota table refreshes. The minimum of 30 seconds prevents excessive API pressure on Anthropic's quota endpoint.
- **Group:** Fetch Behavior

**Examples:**

```text
interval::30    → refresh every 30 seconds (default)
interval::60    → refresh every minute
interval::120   → refresh every 2 minutes
interval::29    → exit 1: "interval must be >= 30" (only when live::1)
```

---

### Parameter :: 22. `jitter::`

Adds a random number of seconds in the range `[0, jitter]` to each outer cycle delay, preventing synchronized refreshes when multiple users run `.usage live::1` with the same `interval::`. Ignored (and not validated) when `live::0`.

- **Type:** `u64`
- **Default:** `0` (no jitter — exact `interval::` timing)
- **Constraints:** Must satisfy `jitter ≤ interval` when `live::1`; violation exits 1 with `"jitter must not exceed interval"`
- **Commands:** [`.usage`](commands.md#command--9-usage)
- **Purpose:** Thunder-herd mitigation — when many users share the same refresh cadence, jitter spreads the API call bursts across a wider time window.
- **Group:** Fetch Behavior

**Examples:**

```text
jitter::0    → no jitter, exact interval timing (default)
jitter::10   → each cycle waits interval + random[0..=10] seconds
jitter::30   → each cycle waits interval + random[0..=30] seconds
jitter::70   → exit 1: "jitter must not exceed interval" (when interval::60)
```

---

### Parameter :: 23. `trace::`

When enabled, writes one `[trace]` diagnostic line to stderr for each internal operation performed by `.usage`: credential file reads, API calls (URL + token prefix), API results, and every step of the `refresh::1` retry path.

- **Type:** `bool`
- **Default:** `0` (off — no diagnostic output)
- **Constraints:** Accepted values: `0`, `1`, `false`, `true`; ignored in live monitor mode (`live::1`)
- **Commands:** [`.usage`](commands.md#command--9-usage)
- **Purpose:** Exposes the internal fetch and refresh mechanics so failures can be diagnosed without guessing. Particularly useful when `refresh::1` appears to have no effect — trace shows exactly whether the refresh was triggered, what `run_isolated` returned, and why the retry was skipped.
- **Group:** Fetch Behavior

**Examples:**

```text
trace::0   → no diagnostic output (default)
trace::1   → print [trace] lines to stderr; stdout output unchanged
```

**Notes:**
- Output goes to stderr so it does not interfere with `format::json` parsing on stdout.
- Token values are truncated to the first 8 characters followed by `...` (`abc12345...`).
- The fetch phase emits one `reading` line and one `GET` + `result` pair per account. The refresh phase emits one `should_retry` line per account plus detailed step lines for accounts where a refresh is attempted.
- Typical trace lines when refresh has no effect (token not server-expired):
  ```
  [trace] refresh  i12@wbox.pro  should_retry=true (reason: HTTP transport error: HTTP 401)
  [trace] refresh  i12@wbox.pro  reading /home/user/.pro/.../i12@wbox.pro.credentials.json
  [trace] refresh  i12@wbox.pro  spawning run_isolated (timeout=30s)
  [trace] refresh  i12@wbox.pro  run_isolated Err(claude timed out after 30 seconds) — skipping
  ```
- For rate-limited accounts (429), the refresh path is not entered:
  ```
  [trace] refresh  i12@wbox.pro  should_retry=false (reason: HTTP transport error: HTTP 429)
  ```

---

### Parameter :: 24. `field::`

When set, outputs the raw resolved path value for a single named field instead of the full path listing. Useful for shell scripts that need one specific path without parsing multi-line output or piping through `jq`.

- **Type:** `String`
- **Default:** `""` (omit to show all paths)
- **Constraints:** Must be one of: `base`, `credentials`, `credential_store`, `projects`, `stats`, `settings`, `session_env`, `sessions`; unknown value exits 1 with an error message listing all valid field names
- **Commands:** [`.paths`](commands.md#command--8-paths)
- **Purpose:** Script integration — eliminates parsing or `jq` when only one path value is needed (e.g., locating the credential store in a refresh script).
- **Group:** Output Selection

**Examples:**

```text
field::credential_store   → /home/user/.persistent/claude/credential/
field::credentials        → /home/user/.claude/.credentials.json
field::base               → /home/user/.claude
field::session_env        → /home/user/.claude/session-env/
field::unknown            → exit 1: "unknown field 'unknown'; valid: base, credentials, credential_store, projects, stats, settings, session_env, sessions"
```

**Notes:**
- When `field::` is set, `format::` is ignored — output is always the raw string value followed by a newline.
- Field names match the JSON keys from `clp .paths format::json` output (underscores, not hyphens).
