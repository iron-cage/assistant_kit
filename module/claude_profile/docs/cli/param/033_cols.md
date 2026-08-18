# Parameter: 33. `cols::`

Controls column visibility in the `.usage` quota table. Accepts comma-separated `+name` (show) and `-name` (hide) modifiers relative to the default column set.

- **Default:** (empty -- use default column set)
- **Constraints:** comma-separated `+col_id` / `-col_id` modifiers
- **Purpose:** Customize which data columns appear in the quota table.

**Column registry:**

| Column ID | Header | Default |
|-----------|--------|---------|
| `status` | (composite emoji) | ON |
| `expires` | Expires | ON |
| `sub` | Sub | **OFF** |
| `renews` | ~Renews | ON |
| `5h_left` | 5h Left | ON |
| `5h_reset` | 5h Reset | ON |
| `7d_left` | 7d Left | ON |
| `7d_son` | 7d(Son) | **OFF** |
| `7d_reset` | 7d Reset | ON |
| `7d_son_reset` | 7d Son Reset | **OFF** |
| `host` | Host | **OFF** |
| `role` | Role | **OFF** |
| `tags` 📋 | Tags | **OFF** |
| `owner` | Owner | ON |
| `next` | → Next | ON |

The `flag` (first column) and `account` (name) columns are structural and always visible.

📋 planned ([feature/075](../../feature/075_account_tags.md)): `tags` shows the account's comma-joined tag set; `role` remains a legacy toggle until per-account lazy migration erases the stored field.

**Examples:**

```text
cols::+sub                     -> add Sub column to default set
cols::+sub,-renews             -> add Sub, remove ~Renews
cols::-5h_reset,-7d_reset      -> hide 5h Reset and 7d Reset
cols::+sub,+7d_son_reset       -> show both hidden-by-default quota columns
cols::+host,+role              -> show machine/host label and legacy role label
cols::+tags                    -> show the account tag set (📋 planned)
```

**See Also:** [feature/009_token_usage.md](../../feature/009_token_usage.md) for column definitions.

### Referenced Type

- **Fundamental Type:** `string`

### Referenced Parameter Groups

| # | Parameter Group | Role |
|---|-----------------|------|
| 1 | [Display Control](../param_group/005_display_control.md) | Member parameter |

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | [`.usage`](../command/006_usage.md#command-9-usage) | Column visibility customization for quota table |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 1 | [Multi-Account Quota Monitoring](../user_story/003_quota_monitoring.md) | Tailored column set for quota monitoring workflows |
| 2 | [Scripted Pipeline Automation](../user_story/004_scripted_automation.md) | Reduced column set for script-friendly output |
