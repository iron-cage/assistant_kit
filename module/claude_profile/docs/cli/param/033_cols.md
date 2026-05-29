# Parameter :: 33. `cols::`

Controls column visibility in the `.usage` quota table. Accepts comma-separated `+name` (show) and `-name` (hide) modifiers relative to the default column set.

- **Type:** `string`
- **Default:** (empty -- use default column set)
- **Constraints:** comma-separated `+col_id` / `-col_id` modifiers
- **Commands:** [`.usage`](../command/006_usage.md#command--9-usage)
- **Purpose:** Customize which data columns appear in the quota table.
- **Group:** Display Control

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
| `7d_son` | 7d(Son) | ON |
| `7d_reset` | 7d Reset | ON |
| `7d_son_reset` | 7d Son Reset | **OFF** |
| `host` | Host | **OFF** |
| `role` | Role | **OFF** |

The `flag` (first column) and `account` (name) columns are structural and always visible.

**Examples:**

```text
cols::+sub                     -> add Sub column to default set
cols::+sub,-7d_son             -> add Sub, remove 7d(Son)
cols::-renews,-7d_son          -> hide Renews and 7d(Son)
cols::+sub,+7d_son_reset       -> show both hidden-by-default quota columns
cols::+host,+role              -> show machine/host tag and user-defined role label
```

**See Also:** [feature/009_token_usage.md](../../feature/009_token_usage.md) for column definitions.
