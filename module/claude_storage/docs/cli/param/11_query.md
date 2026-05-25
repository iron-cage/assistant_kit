# Parameter :: 11. `query::`

Search query string. Required by `.search`.

**Type:** String

**Fundamental Type:** String (raw)

**Constraints:**
- Must be non-empty
- Error on empty: `"query must be non-empty"`

**Default:** none — **required**

**Commands:** `.search`

**Alias:** `q`

**Purpose:** The text to search for in session content. Default matching is case-insensitive substring. Multi-word phrases should be quoted in the shell.

**Examples:**
```bash
# Valid values
query::error                        # Single term
query::"session management"         # Phrase (shell-quoted)
q::version_bump                     # Using alias

# Invalid values
query::                             # "query must be non-empty"
```

### Referenced Type
| Type | Kind | Fundamental | Key Constraint |
|------|------|-------------|----------------|
| String | Base type | String | Non-empty |

### Referenced Commands
| # | Command | Default | Notes |
|---|---------|---------|-------|
| 5 | [`.search`](../command/05_search.md) | none — required | Search term; alias `q` supported |

### Referenced User Stories
| # | User Story | Persona |
|---|------------|---------|
| 2 | [Find Past Conversation](../user_story/002_find_past_conversation.md) | developer |
