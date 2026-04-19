# Parameters

### All Parameters (5 total)

| # | Parameter | Type | Default | Valid Values | Purpose | Used In |
|---|-----------|------|---------|--------------|---------|---------|
| 1 | `name::` | `AccountName` | Varies | Any filesystem-safe string | Account name for save/switch/delete (required); or status/limits query (optional) | 5 cmds |
| 2 | `verbosity::` / `v::` | `VerbosityLevel` | `1` | `0`, `1`, `2` | Output detail: 0=quiet, 1=normal, 2=verbose | 7 cmds |
| 3 | `format::` | `OutputFormat` | `text` | `text`, `json` | Output format: `text` or `json` | 7 cmds |
| 4 | `threshold::` | `WarningThreshold` | `3600` | Non-negative integer (seconds) | Seconds before token expiry to classify as ExpiringSoon | 1 cmd |
| 5 | `dry::` | `bool` | `0` | `0`, `1`, `false`, `true` | Print intended action without executing | 3 cmds |

**Total:** 5 parameters

*Parameters 2-3 form the Output Control group*

---

### Parameter :: 1. `name::`

Identifies which named account to operate on. Required for mutation commands; optional on `.account.status` (FR-16) to query a specific account's token state without switching to it.

- **Type:** `AccountName`
- **Default:** **(required)** on `.account.save`, `.account.switch`, `.account.delete`; **optional** on `.account.status` (omit to query the active account)
- **Constraints:** Non-empty, no filesystem-forbidden characters (`/\:*?"<>|` or null bytes)
- **Commands:** [`.account.status`](commands.md#command--4-accountstatus) *(optional)*, [`.account.save`](commands.md#command--5-accountsave), [`.account.switch`](commands.md#command--6-accountswitch), [`.account.delete`](commands.md#command--7-accountdelete), [`.account.limits`](commands.md#command--12-accountlimits) *(optional)*
- **Purpose:** Selects the target credential file at `~/.claude/accounts/{name}.credentials.json`. Name validation matches the library's `account::validate_name()` rules. On `.account.status`, an invalid name exits 1; a valid but unknown name exits 2.

**Examples:**

```text
name::work        → ~/.claude/accounts/work.credentials.json
name::personal    → ~/.claude/accounts/personal.credentials.json
name::client-a    → ~/.claude/accounts/client-a.credentials.json
```

---

### Parameter :: 2. `verbosity::` / `v::`

Controls the amount of detail in text output. Higher levels add labels, metadata, and diagnostic context. Does not affect computation or exit codes.

- **Type:** `VerbosityLevel`
- **Default:** `1` (normal output with labels)
- **Constraints:** Integer 0-2
- **Commands:** [`.account.list`](commands.md#command--3-accountlist), [`.account.status`](commands.md#command--4-accountstatus), [`.token.status`](commands.md#command--8-tokenstatus), [`.paths`](commands.md#command--9-paths), [`.usage`](commands.md#command--10-usage), [`.credentials.status`](commands.md#command--11-credentialsstatus), [`.account.limits`](commands.md#command--12-accountlimits)
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
