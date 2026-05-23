# Format :: 3. table

- **ID:** F03
- **Trigger:** `format::table`
- **Scope:** `.accounts` only — all other commands reject `format::table` with exit 1

### Structure

Compact aligned table with a title, blank line, header row, separator row, and one data row per account. Fixed columns; field-presence parameters are ignored in table mode.

```
Accounts

   Account         Sub   Tier                     Expires     Email
-  --------------  ----  -----------------------  ----------  ----------------
✓  alice@acme.com  max   default_claude_max_20x   in 2h 11m   alice@acme.com
   alice@home.com  pro   default_claude_pro        in 5h 30m   N/A
```

**Columns:** flag (`✓`/`*`/`→`/space), Account, Sub, Tier, Expires, Email.

**Flag semantics:** `✓` = current (live session) account; `*` = `_active`-marker-but-not-current (divergence); `→` = recommended next account (most quota remaining).

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

# Only .accounts accepts format::table
clp .usage format::table
# exit 1: unknown format 'table': expected text or json
```

**Notes:**
- Only `.accounts` accepts `format::table`; all other format-capable commands reject it with exit 1.
- Field-presence parameters (`sub::`, `tier::`, `expires::`, `email::`) are ignored in table mode — all columns always appear.
