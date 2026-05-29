# Parameter :: 19. `show_stat::`

Show session statistics footer in content mode.

**Type:** Boolean

**Fundamental Type:** Boolean

**Default:** `0`

**Commands:** `.show`

**Purpose:** When set to `1`, appends a statistics section to session content output — total entry count, user/assistant breakdown, and timestamp range. Has no effect in `show_metadata::1` mode (metadata mode always shows structured fields including timestamps). Independent of `show_tokens::`.

**Examples:**
```bash
show_stat::0    # Default — no statistics footer
show_stat::1    # Append statistics footer after session content
```

**Group:** [Output Control](../param_group/01_output_control.md)

### Referenced Commands
| # | Command | Default | Notes |
|---|---------|---------|-------|
| 3 | [`.show`](../command/03_show.md) | `0` | Appends statistics footer in content mode |

### Referenced Parameter Groups
| # | Group | Membership | Co-members |
|---|-------|------------|------------|
| 1 | [Output Control](../param_group/01_output_control.md) | Full | `show_tokens::`, `show_tree::` |
