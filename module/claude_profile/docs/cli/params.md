# Parameters

### All Parameters (19 total)

| # | Parameter | Type | Default | Valid Values | Purpose | Used In |
|---|-----------|------|---------|--------------|---------|---------|
| 1 | `name::` | `AccountName` | Varies | Email address | Account email for switch/delete (required); save (optional, inferred from `~/.claude.json`); accounts/limits query (optional) | 5 cmds |
| 2 | `verbosity::` / `v::` | `VerbosityLevel` | `1` | `0`, `1`, `2` | Output detail: 0=quiet, 1=normal, 2=verbose | 4 cmds |
| 3 | `format::` / `fmt::` | `OutputFormat` | `text` | `text`, `json` | Output format: `text` or `json` | 6 cmds |
| 4 | `threshold::` | `WarningThreshold` | `3600` | Non-negative integer (seconds) | Seconds before token expiry to classify as ExpiringSoon | 1 cmd |
| 5 | `dry::` | `bool` | `0` | `0`, `1`, `false`, `true` | Print intended action without executing | 3 cmds |
| 6 | `account::` | `bool` | `1` | `0`, `1` | Show active account name line (`.credentials.status`) | 1 cmd |
| 7 | `sub::` | `bool` | `1` | `0`, `1` | Show subscription type line | 2 cmds |
| 8 | `tier::` | `bool` | `1` | `0`, `1` | Show rate-limit tier line | 2 cmds |
| 9 | `token::` | `bool` | `1` | `0`, `1` | Show token status line (`.credentials.status`) | 1 cmd |
| 10 | `expires::` | `bool` | `1` | `0`, `1` | Show token expiry duration line | 2 cmds |
| 11 | `email::` | `bool` | `1` | `0`, `1` | Show email address line (`.credentials.status`) | 1 cmd |
| 12 | `org::` | `bool` | `1` | `0`, `1` | Show organisation name line | 2 cmds |
| 13 | `file::` | `bool` | `0` | `0`, `1` | Show credentials file path, opt-in (`.credentials.status`) | 1 cmd |
| 14 | `saved::` | `bool` | `0` | `0`, `1` | Show saved account count, opt-in (`.credentials.status`) | 1 cmd |
| 15 | `active::` | `bool` | `1` | `0`, `1` | Show active/inactive status line (`.accounts`) | 1 cmd |
| 16 | `display_name::` | `bool` | `0` | `0`, `1` | Show display name from `oauthAccount`, opt-in | 2 cmds |
| 17 | `role::` | `bool` | `0` | `0`, `1` | Show organisation role from `oauthAccount`, opt-in | 2 cmds |
| 18 | `billing::` | `bool` | `0` | `0`, `1` | Show billing type from `oauthAccount`, opt-in | 2 cmds |
| 19 | `model::` | `bool` | `0` | `0`, `1` | Show active model from settings, opt-in | 2 cmds |

**Total:** 19 parameters

*Parameters 2-3 form the Output Control group; parameters 6-19 form the Field Presence group*

---

### Parameter :: 1. `name::`

Identifies which named account to operate on. Required for destructive commands; optional with inference on `.account.save`; optional for query on `.accounts` and `.account.limits`.

- **Type:** `AccountName`
- **Default:** **(required)** on `.account.switch`, `.account.delete`; **inferred** on `.account.save` (reads `emailAddress` from `~/.claude.json`; exits 1 if absent); **optional** on `.accounts` (omit to list all) and `.account.limits` (omit for active account)
- **Constraints:** Valid email address (non-empty, must contain `@`, non-empty local part and domain)
- **Commands:** [`.accounts`](commands.md#command--3-accounts) *(optional)*, [`.account.save`](commands.md#command--4-accountsave) *(optional/inferred)*, [`.account.switch`](commands.md#command--5-accountswitch), [`.account.delete`](commands.md#command--6-accountdelete), [`.account.limits`](commands.md#command--11-accountlimits) *(optional)*
- **Purpose:** Selects the target credential file at `{credential_store}/{email}.credentials.json`. Name validation matches the library's `account::validate_name()` rules. An invalid name exits 1; a valid but unknown name exits 2.

**Examples:**

```text
name::alice@acme.com   → {credential_store}/alice@acme.com.credentials.json
name::alice@home.com   → {credential_store}/alice@home.com.credentials.json
```

---

### Parameter :: 2. `verbosity::` / `v::`

Controls the amount of detail in text output. Higher levels add labels, metadata, and diagnostic context. Does not affect computation or exit codes.

- **Type:** `VerbosityLevel`
- **Default:** `1` (normal output with labels)
- **Constraints:** Integer 0-2
- **Commands:** [`.token.status`](commands.md#command--7-tokenstatus), [`.paths`](commands.md#command--8-paths), [`.usage`](commands.md#command--9-usage), [`.account.limits`](commands.md#command--11-accountlimits)
- **Purpose:** Adapts output density to context: `0` for scripting (bare values), `1` for interactive use (labeled), `2` for debugging (full metadata).
- **Group:** Output Control

**Examples:**

```text
v::0   → bare values only (names, paths, status word)
v::1   → labeled output with human context (default)
v::2   → extended metadata including expiry times
```

---

### Parameter :: 3. `format::` / `fmt::`

Selects between human-readable text output and machine-parseable JSON. Text is the default for interactive use; JSON enables pipeline integration.

- **Type:** `OutputFormat`
- **Default:** `text`
- **Alias:** `fmt::` (short form; both accepted at runtime)
- **Constraints:** One of `text`, `json` (case-insensitive)
- **Commands:** [`.accounts`](commands.md#command--3-accounts), [`.token.status`](commands.md#command--7-tokenstatus), [`.paths`](commands.md#command--8-paths), [`.usage`](commands.md#command--9-usage), [`.credentials.status`](commands.md#command--10-credentialsstatus), [`.account.limits`](commands.md#command--11-accountlimits)
- **Purpose:** Enables CLI composability — `format::json` output can be piped to `jq` for structured extraction without parsing fragile text layouts.
- **Group:** Output Control

**Examples:**

```text
format::text   → human-readable labeled output (default)
format::json   → JSON object or array
fmt::json      → same as format::json (short alias)
```

---

### Parameter :: 4. `threshold::`

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

### Parameter :: 5. `dry::`

Activates simulation mode for mutation commands. When `dry::1`, the command prints what it *would* do without modifying any files. Part of the standard CLI dry-run pattern.

- **Type:** `bool` (`0` / `1` or `false` / `true`)
- **Default:** `0` (execute normally)
- **Constraints:** Accepted values: `0`, `1`, `false`, `true`
- **Commands:** [`.account.save`](commands.md#command--4-accountsave), [`.account.switch`](commands.md#command--5-accountswitch), [`.account.delete`](commands.md#command--6-accountdelete)
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

### Parameter :: 6. `account::`

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

### Parameter :: 7. `sub::`

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

### Parameter :: 8. `tier::`

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

### Parameter :: 9. `token::`

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

### Parameter :: 10. `expires::`

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

### Parameter :: 11. `email::`

Controls whether the email address line appears in `.credentials.status` output. Source: `emailAddress` field in `~/.claude.json`.

- **Type:** `bool`
- **Default:** `1` (shown)
- **Constraints:** Accepted values: `0`, `1`, `false`, `true`
- **Commands:** [`.credentials.status`](commands.md#command--10-credentialsstatus)
- **Purpose:** Allows suppression of the email line; especially useful when the field is consistently `N/A`.
- **Group:** Field Presence

**Examples:**

```text
email::1   → Email:   alice@acme.com  (default; N/A when absent)
email::0   → line omitted
```

---

### Parameter :: 12. `org::`

Controls whether the organisation name line appears in output. Used by both `.accounts` (per stored credential) and `.credentials.status` (from `~/.claude.json`).

- **Type:** `bool`
- **Default:** `1` (shown)
- **Constraints:** Accepted values: `0`, `1`, `false`, `true`
- **Commands:** [`.accounts`](commands.md#command--3-accounts), [`.credentials.status`](commands.md#command--10-credentialsstatus)
- **Purpose:** Allows suppression of the org line; often `N/A` for individual accounts.
- **Group:** Field Presence

**Examples:**

```text
org::1   → Org:     Acme Corp  (default; N/A when absent)
org::0   → line omitted
```

---

### Parameter :: 13. `file::`

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

### Parameter :: 14. `saved::`

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

### Parameter :: 15. `active::`

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

### Parameter :: 16. `display_name::`

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

### Parameter :: 17. `role::`

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

### Parameter :: 18. `billing::`

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

### Parameter :: 19. `model::`

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
