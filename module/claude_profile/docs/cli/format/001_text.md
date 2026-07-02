# Format: 1. text

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
#   🟢 bob@example.com      🟢 100%    in 4h 58m  🟢 88%   28%      in 6d 14h  in 5h 02m   ~in 30d      in 6d 14h +7d
# ✓ 🟢 alice@example.com    🟢 86%     in 3h 19m  🟢 65%   35%      in 4d 23h  in 7h 24m   ~in 6d       in 4d 23h +7d
#
# Valid: 2 / 2   session: claude-sonnet-5  effort: low
# Next (renew): bob@example.com  in 6d 14h +7d  model: sonnet

clp .token.status
# valid — 7h24m remaining

clp .paths
# Claude JSON:     /home/user/.claude.json
# Credentials:     /home/user/.claude/.credentials.json
# Settings:        /home/user/.claude/settings.json
# Credential store: /home/user/.persistent/claude/credential/
```

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | [`.accounts`](../command/001_account.md#command--3-accounts) | Default — account list labeled output |
| 2 | [`.token.status`](../command/005_token.md#command--7-tokenstatus) | Default — token classification labeled output |
| 3 | [`.paths`](../command/004_paths.md#command--8-paths) | Default — path resolution labeled output |
| 4 | [`.usage`](../command/006_usage.md#command--9-usage) | Default — multi-account quota table |
| 5 | [`.credentials.status`](../command/002_credentials.md#command--10-credentialsstatus) | Default — credential metadata labeled output |
| 6 | [`.account.limits`](../command/001_account.md#command--11-accountlimits) | Default — quota limits labeled output |

### Referenced User Stories

| # | User Story | Relevance |
|---|------------|-----------|
| 1 | [Account Rotation](../user_story/001_account_rotation.md) | Default output for rotation status |
| 2 | [Account Onboarding](../user_story/002_onboarding.md) | Default output for lifecycle commands |
| 3 | [Multi-Account Quota Monitoring](../user_story/003_quota_monitoring.md) | Interactive quota table output |
| 4 | [Scripted Pipeline Automation](../user_story/004_scripted_automation.md) | Text output for human-readable logs |
| 5 | [Credential Diagnostics](../user_story/005_credential_diagnostics.md) | Default output for diagnostic commands |
