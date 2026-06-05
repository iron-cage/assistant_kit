# Format :: 1. text

- **ID:** F01
- **Trigger:** `format::text` (default — used when `format::` is omitted)
- **Scope:** All format-capable commands: `.accounts`, `.token.status`, `.paths`, `.usage`, `.credentials.status`, `.account.limits`

### Structure

Human-readable labeled key-value output. Each field appears on its own line with a padded label and value. Multi-record commands separate records with blank lines. `.usage` produces a multi-column aligned table with a header row, data rows, and a footer recommendation line.

```
Label:   value
Label2:  value2
```

### Rendering Mechanism

`data_fmt` text renderer — aligns label widths within each block, pads values for visual alignment. For `.usage`, produces a full table with aligned columns, header row, and footer.

### Example

```bash
clp .credentials.status
# Account: alice@acme.com
# Sub:     max
# Tier:    default_claude_max_20x
# Token:   valid
# Expires: in 7h 24m
# Email:   N/A

clp .accounts
# alice@acme.com
#   Active:  yes
#   Sub:     max
#   Tier:    default_claude_max_20x
#   Expires: in 7h 24m
#   Email:   N/A

clp .usage
# Quota
#
#   ●  Account              5h Left     5h Reset    7d Left  7d(Son)  7d Reset   Expires     ~Renews      → Next
# → 🟢 bob@example.com      🟢 100%    in 4h 58m  🟢 88%   28%      in 6d 14h  in 5h 02m   ~in 30d      in 6d 14h +7d
# ✓ 🟢 alice@example.com    🟢 86%     in 3h 19m  🟢 65%   35%      in 4d 23h  in 7h 24m   ~in 6d       in 4d 23h +7d
#
# Valid: 2 / 2   ->  Next by strategy:
#   endurance  bob@example.com     100% session, 5h resets in 4h 58m
#   drain      bob@example.com     28% 7d left, 7d resets in 6d 14h

clp .token.status
# valid — 7h24m remaining

clp .paths
# Claude JSON:     /home/user/.claude.json
# Credentials:     /home/user/.claude/.credentials.json
# Settings:        /home/user/.claude/settings.json
# Credential store: /home/user/.persistent/claude/credential/
```
