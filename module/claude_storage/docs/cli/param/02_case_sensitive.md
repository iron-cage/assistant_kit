# Parameter :: 2. `case_sensitive::`

### Scope

- **Purpose**: Specify the `case_sensitive::` CLI parameter.
- **Responsibility**: Type, defaults, valid values, and command usage for `case_sensitive::`.
- **In Scope**: Value constraints, default behavior, command interactions.
- **Out of Scope**: Type definitions (→ `type/`), command behavior (→ `command/`).

Enable case-sensitive matching in search operations.

**Type:** Boolean

**Fundamental Type:** Boolean flag

**Constraints:**
- Valid values: `0`, `1`
- `0` = case-insensitive (default)
- `1` = case-sensitive

**Default:** `0` (case-insensitive)

**Commands:** `.search`

**Purpose:** Controls whether search matches are case-sensitive. Default case-insensitive mode is practical for most searches; enable case-sensitive when searching for identifiers, variable names, or other case-significant strings.

**Examples:**
```bash
# Valid values
case_sensitive::0     # Case-insensitive (default)
case_sensitive::1     # Case-sensitive

# Invalid values
case_sensitive::true  # Not a boolean: "case_sensitive must be 0 or 1"
```

### Referenced Type
| Type | Kind | Fundamental | Key Constraint |
|------|------|-------------|----------------|
| Boolean | Base type | Boolean flag | `0` (false) or `1` (true) |

### Referenced Commands
| # | Command | Default | Notes |
|---|---------|---------|-------|
| 5 | [`.search`](../command/05_search.md) | `0` | Enables case-sensitive matching |

### Referenced Parameter Groups

None.

### Referenced User Stories
| # | User Story | Persona |
|---|------------|---------|
| 2 | [Find Past Conversation](../user_story/002_find_past_conversation.md) | developer |
