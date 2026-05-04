# Parameters

### All Parameters (14 total)

| # | Parameter | Type | Default | Valid Values | Purpose | Used In |
|---|-----------|------|---------|--------------|---------|---------|
| 1 | `name::` | `AccountName` | Varies | Email address | Account email for save/switch/delete (required); or status/limits query (optional) | 5 cmds |
| 2 | `verbosity::` / `v::` | `VerbosityLevel` | `1` | `0`, `1`, `2` | Output detail: 0=quiet, 1=normal, 2=verbose | 6 cmds |
| 3 | `format::` | `OutputFormat` | `text` | `text`, `json` | Output format: `text` or `json` | 7 cmds |
| 4 | `threshold::` | `WarningThreshold` | `3600` | Non-negative integer (seconds) | Seconds before token expiry to classify as ExpiringSoon | 1 cmd |
| 5 | `dry::` | `bool` | `0` | `0`, `1`, `false`, `true` | Print intended action without executing | 3 cmds |
| 6 | `account::` | `bool` | `1` | `0`, `1` | Show active account name line (`.credentials.status`) | 1 cmd |
| 7 | `sub::` | `bool` | `1` | `0`, `1` | Show subscription type line (`.credentials.status`) | 1 cmd |
| 8 | `tier::` | `bool` | `1` | `0`, `1` | Show rate-limit tier line (`.credentials.status`) | 1 cmd |
| 9 | `token::` | `bool` | `1` | `0`, `1` | Show token status line (`.credentials.status`) | 1 cmd |
| 10 | `expires::` | `bool` | `1` | `0`, `1` | Show token expiry duration line (`.credentials.status`) | 1 cmd |
| 11 | `email::` | `bool` | `1` | `0`, `1` | Show email address line (`.credentials.status`) | 1 cmd |
| 12 | `org::` | `bool` | `1` | `0`, `1` | Show organization name line (`.credentials.status`) | 1 cmd |
| 13 | `file::` | `bool` | `0` | `0`, `1` | Show credentials file path, opt-in (`.credentials.status`) | 1 cmd |
| 14 | `saved::` | `bool` | `0` | `0`, `1` | Show saved account count, opt-in (`.credentials.status`) | 1 cmd |

**Total:** 14 parameters

*Parameters 2-3 form the Output Control group; parameters 6-14 form the Field Presence group*

---

### Parameter :: 1. `name::`

Identifies which named account to operate on. Required for mutation commands; optional on `.account.status` (FR-16) to query a specific account's token state without switching to it.

- **Type:** `AccountName`
- **Default:** **(required)** on `.account.save`, `.account.switch`, `.account.delete`; **optional** on `.account.status` (omit to query the active account)
- **Constraints:** Valid email address (non-empty, must contain `@`, non-empty local part and domain)
- **Commands:** [`.account.status`](commands.md#command--4-accountstatus) *(optional)*, [`.account.save`](commands.md#command--5-accountsave), [`.account.switch`](commands.md#command--6-accountswitch), [`.account.delete`](commands.md#command--7-accountdelete), [`.account.limits`](commands.md#command--12-accountlimits) *(optional)*
- **Purpose:** Selects the target credential file at `{credential_store}/{email}.credentials.json`. Name validation matches the library's `account::validate_name()` rules. On `.account.status`, an invalid name exits 1; a valid but unknown name exits 2.

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
- **Commands:** [`.account.list`](commands.md#command--3-accountlist), [`.account.status`](commands.md#command--4-accountstatus), [`.token.status`](commands.md#command--8-tokenstatus), [`.paths`](commands.md#command--9-paths), [`.usage`](commands.md#command--10-usage), [`.account.limits`](commands.md#command--12-accountlimits)
- **Purpose:** Adapts output density to context: `0` for scripting (bare values), `1` for interactive use (labeled), `2` for debugging (full metadata).
- **Group:** Output Control

**Examples:**

```text
v::0   → bare values only (names, paths, status word)
v::1   → labeled output with human context (default)
v::2   → extended metadata including expiry times
```

---

### Parameter :: 3. `format::`

Selects between human-readable text output and machine-parseable JSON. Text is the default for interactive use; JSON enables pipeline integration.

- **Type:** `OutputFormat`
- **Default:** `text`
- **Constraints:** One of `text`, `json` (case-insensitive)
- **Commands:** [`.account.list`](commands.md#command--3-accountlist), [`.account.status`](commands.md#command--4-accountstatus), [`.token.status`](commands.md#command--8-tokenstatus), [`.paths`](commands.md#command--9-paths), [`.usage`](commands.md#command--10-usage), [`.credentials.status`](commands.md#command--11-credentialsstatus), [`.account.limits`](commands.md#command--12-accountlimits)
- **Purpose:** Enables CLI composability — `format::json` output can be piped to `jq` for structured extraction without parsing fragile text layouts.
- **Group:** Output Control

**Examples:**

```text
format::text   → human-readable labeled output (default)
format::json   → JSON object or array
```

---

### Parameter :: 4. `threshold::`

Overrides the default 60-minute warning window for token expiry classification. Tokens expiring within `threshold::` seconds are classified as `ExpiringSoon` instead of `Valid`.

- **Type:** `WarningThreshold`
- **Default:** `3600` (60 minutes, matching `token::WARNING_THRESHOLD_SECS`)
- **Constraints:** Non-negative integer (seconds)
- **Commands:** [`.token.status`](commands.md#command--8-tokenstatus)
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
- **Commands:** [`.account.save`](commands.md#command--5-accountsave), [`.account.switch`](commands.md#command--6-accountswitch), [`.account.delete`](commands.md#command--7-accountdelete)
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
- **Commands:** [`.credentials.status`](commands.md#command--11-credentialsstatus)
- **Purpose:** Lets callers suppress the account name line when it is irrelevant (e.g., scripting that only needs the token state).
- **Group:** Field Presence

**Examples:**

```text
account::1   → Account: alice@acme.com  (default)
account::0   → line omitted
```

---

### Parameter :: 7. `sub::`

Controls whether the subscription type line appears in `.credentials.status` output.

- **Type:** `bool`
- **Default:** `1` (shown)
- **Constraints:** Accepted values: `0`, `1`, `false`, `true`
- **Commands:** [`.credentials.status`](commands.md#command--11-credentialsstatus)
- **Purpose:** Allows suppression of the subscription type when only token validity matters.
- **Group:** Field Presence

**Examples:**

```text
sub::1   → Sub:     max  (default)
sub::0   → line omitted
```

---

### Parameter :: 8. `tier::`

Controls whether the rate-limit tier line appears in `.credentials.status` output.

- **Type:** `bool`
- **Default:** `1` (shown)
- **Constraints:** Accepted values: `0`, `1`, `false`, `true`
- **Commands:** [`.credentials.status`](commands.md#command--11-credentialsstatus)
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
- **Commands:** [`.credentials.status`](commands.md#command--11-credentialsstatus)
- **Purpose:** Allows suppression of the token status line (rare; usually the most important field).
- **Group:** Field Presence

**Examples:**

```text
token::1   → Token:   valid  (default)
token::0   → line omitted
```

---

### Parameter :: 10. `expires::`

Controls whether the token expiry duration line appears in `.credentials.status` output.

- **Type:** `bool`
- **Default:** `1` (shown)
- **Constraints:** Accepted values: `0`, `1`, `false`, `true`
- **Commands:** [`.credentials.status`](commands.md#command--11-credentialsstatus)
- **Purpose:** Allows suppression of the expiry countdown when exact timing is not needed.
- **Group:** Field Presence

**Examples:**

```text
expires::1   → Expires: in 7h 24m  (default)
expires::0   → line omitted
```

---

### Parameter :: 11. `email::`

Controls whether the email address line appears in `.credentials.status` output. Source: `emailAddress` field in `~/.claude/.claude.json`.

- **Type:** `bool`
- **Default:** `1` (shown)
- **Constraints:** Accepted values: `0`, `1`, `false`, `true`
- **Commands:** [`.credentials.status`](commands.md#command--11-credentialsstatus)
- **Purpose:** Allows suppression of the email line; especially useful when the field is consistently `N/A`.
- **Group:** Field Presence

**Examples:**

```text
email::1   → Email:   alice@acme.com  (default; N/A when absent)
email::0   → line omitted
```

---

### Parameter :: 12. `org::`

Controls whether the organization name line appears in `.credentials.status` output. Source: `organizationName` field in `~/.claude/.claude.json`.

- **Type:** `bool`
- **Default:** `1` (shown)
- **Constraints:** Accepted values: `0`, `1`, `false`, `true`
- **Commands:** [`.credentials.status`](commands.md#command--11-credentialsstatus)
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
- **Commands:** [`.credentials.status`](commands.md#command--11-credentialsstatus)
- **Purpose:** Exposes the resolved path to `~/.claude/.credentials.json` for diagnostics and tooling integration.
- **Group:** Field Presence

**Examples:**

```text
file::0   → line omitted  (default)
file::1   → File:    /home/user/.claude/.credentials.json
```

---

### Parameter :: 14. `saved::`

Controls whether the saved account count line appears in `.credentials.status` output. Opt-in (default `0`). Counts `*.credentials.json` files in the accounts directory.

- **Type:** `bool`
- **Default:** `0` (hidden)
- **Constraints:** Accepted values: `0`, `1`, `false`, `true`
- **Commands:** [`.credentials.status`](commands.md#command--11-credentialsstatus)
- **Purpose:** Shows how many credential profiles are saved in the account store; shows `0` when the accounts directory is absent.
- **Group:** Field Presence

**Examples:**

```text
saved::0   → line omitted  (default)
saved::1   → Saved:   3 account(s)
```
