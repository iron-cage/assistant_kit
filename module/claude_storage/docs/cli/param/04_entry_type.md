# Parameter :: 4. `entry_type::`

Filter search results by conversation entry type.

**Type:** [`EntryType`](../type/02_entry_type.md)

**Fundamental Type:** String enum wrapper

**Constraints:**
- Valid values: `user`, `assistant`, `all`
- Case-insensitive on input
- Error on invalid: `"entry_type must be user|assistant|all, got {value}"`

**Default:** `all`

**Commands:** `.search`

**Purpose:** Restricts search to only user-authored messages or only assistant-authored messages. Use `entry_type::user` when searching for what you asked about; use `entry_type::assistant` when searching for what the assistant responded with.

**Examples:**
```bash
# Valid values
entry_type::user        # User messages only
entry_type::assistant   # Assistant messages only
entry_type::all         # No filter (default)

# Invalid values (rejected with error)
entry_type::both        # "entry_type must be user|assistant|all, got both"
entry_type::system      # "entry_type must be user|assistant|all, got system"
```

### Referenced Type
| Type | Kind | Fundamental | Key Constraint |
|------|------|-------------|----------------|
| [`EntryType`](../type/02_entry_type.md) | String enum wrapper | String | `user`, `assistant`, or `all` |

### Referenced Commands
| # | Command | Default | Notes |
|---|---------|---------|-------|
| 5 | [`.search`](../command/05_search.md) | `all` | Restricts search to one entry type |

### Referenced User Stories
| # | User Story | Persona |
|---|------------|---------|
| 2 | [Find Past Conversation](../user_story/002_find_past_conversation.md) | developer |
