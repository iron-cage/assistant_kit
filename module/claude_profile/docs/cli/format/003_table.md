# Format: 3. table

- **ID:** F03
- **Trigger:** `format::table`
- **Scope:** `.accounts`, `.models`, and `.provider.select` accept `format::table` (all three default to `format::text` when `format::` is omitted); all other format-capable commands reject `format::table` with exit 1

### Structure

Compact aligned table with a title, blank line, header row, separator row, and one data row per account. Fixed columns; field-presence parameters are ignored in table mode.

```
Accounts

   Account         Sub   Tier                     Expires     Email
-  --------------  ----  -----------------------  ----------  ----------------
✓  alice@acme.com  max   default_claude_max_20x   in 2h 11m   alice@acme.com
   alice@home.com  pro   default_claude_pro        in 5h 30m   N/A
```

**Columns:** flag (`✓`/`*`/`@`/space), Account, Sub, Tier, Expires, Email.

**Flag semantics:** `✓` = current (live session) account; `*` = active-marker-but-not-current (divergence); `@` = occupied on another machine. Priority: `✓` > `*` > `@` > blank.

### Rendering Mechanism

`data_fmt` table renderer — pads each column to the width of its widest value; separator row of `-` characters between header and data rows.

### Example

```bash
clp .accounts format::table
# Accounts
#
#    Account         Sub   Tier                     Expires     Email
# -  --------------  ----  -----------------------  ----------  ----------------
# ✓  alice@acme.com  max   default_claude_max_20x   in 2h 11m   alice@acme.com
#    alice@home.com  pro   default_claude_pro        in 5h 30m   N/A

# .accounts, .models, and .provider.select accept format::table — other commands reject it
clp .usage format::table
# exit 1: format::table is only supported by .accounts
```

**Notes:**
- `.accounts`, `.models`, and `.provider.select` accept `format::table`; all other format-capable commands reject it with exit 1 (exact rejection message varies by command).
- Field-presence parameters (`sub::`, `tier::`, `expires::`, `email::`) are ignored in table mode — all columns always appear (applies to `.accounts`'s table; `.models`/`.provider.select` define no field-presence parameters).

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | [`.accounts`](../command/001_account.md#command-3-accounts) | Compact aligned table of all accounts |
| 2 | [`.models`](../command/008_models.md#command-19-models) | Accepts `format::table` (non-default) — same table renderer as `.accounts` |
| 3 | [`.provider.select`](../command/009_provider.md#command-21-providerselect) | Accepts `format::table` (non-default) — rendered identically to `format::text` |

### Referenced User Stories

| # | User Story | Relevance |
|---|------------|-----------|
| 1 | [Account Rotation](../user_story/001_account_rotation.md) | At-a-glance account status before rotation |
| 2 | [Multi-Account Quota Monitoring](../user_story/003_quota_monitoring.md) | Multi-account side-by-side quota comparison |
