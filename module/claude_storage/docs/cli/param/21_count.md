# Parameter :: 21. `count::`

### Scope

- **Purpose**: Specify the `count::` CLI parameter.
- **Responsibility**: Type, defaults, valid values, and command usage for `count::`.
- **In Scope**: Value constraints, default behavior, command interactions.
- **Out of Scope**: Type definitions (→ `type/`), command behavior (→ `command/`).

Boolean mode flag for `.list` that suppresses the full listing and outputs only the count as a bare integer.

**Type:** Boolean

**Fundamental Type:** Boolean (`0`/`1`, `true`/`false`)

**Constraints:**
- Only meaningful with `type::conversation`
- When `1`: outputs bare integer count + newline, no other output
- When `0` (default): outputs full listing

**Default:** `0` (full listing)

**Commands:** `.list`

**Purpose:** Enables scripting use cases where only the count is needed. For example, `clg .list type::conversation count::1 project::abc123` outputs `3` and nothing else.

**Examples:**
```bash
# Count mode on
count::1    # Output bare integer only

# Count mode off (default)
count::0    # Output full listing

# Combined with conversation type
.list type::conversation count::1 project::abc123   # e.g., outputs "3"
```

### Referenced Type
| Type | Kind | Fundamental | Key Constraint |
|------|------|-------------|----------------|
| Boolean | Base type | Boolean flag | `0` (full listing) or `1` (count only) |

### Referenced Parameter Groups

None.

### Referenced Commands
| # | Command | Default | Notes |
|---|---------|---------|-------|
| 2 | [`.list`](../command/02_list.md) | `0` | Only meaningful with `type::conversation` |

### Referenced User Stories
| # | User Story | Persona |
|---|------------|---------|
| 4 | [Query Storage Programmatically](../user_story/004_query_storage_programmatically.md) | developer |
