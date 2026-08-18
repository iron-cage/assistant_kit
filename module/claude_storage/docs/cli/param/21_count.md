# Parameter :: 21. `count::`

### Scope

- **Purpose**: Specify the `count::` CLI parameter.
- **Responsibility**: Type, defaults, valid values, and command usage for `count::`.
- **In Scope**: Value constraints, default behavior, command interactions.
- **Out of Scope**: Type definitions (→ `type/`), command behavior (→ `command/`).

Boolean mode flag for `.projects` that suppresses the full listing and outputs only the count as a bare integer. Absorbed from `.list` (see [`02_list.md`](../command/02_list.md)); now pairs with `ids::` instead of `type::conversation`.

**Type:** Boolean

**Fundamental Type:** Boolean (`0`/`1`, `true`/`false`)

**Constraints:**
- Only meaningful with `ids::1`
- When `1`: outputs bare integer count + newline, no other output
- When `0` (default): outputs full listing

**Default:** `0` (full listing)

**Commands:** [`.projects`](../command/07_projects.md)

**Purpose:** Enables scripting use cases where only the count is needed. For example, `clg .projects project::abc123 ids::1 count::1` outputs `3` and nothing else.

**Examples:**
```bash
# Count mode on
count::1    # Output bare integer only

# Count mode off (default)
count::0    # Output full listing

# Combined with ids:: and project::
.projects project::abc123 ids::1 count::1   # e.g., outputs "3"
```

### Referenced Type
| Type | Kind | Fundamental | Key Constraint |
|------|------|-------------|----------------|
| Boolean | Base type | Boolean flag | `0` (full listing) or `1` (count only) |

### Referenced Commands
| # | Command | Default | Notes |
|---|---------|---------|-------|
| 7 | [`.projects`](../command/07_projects.md) | `0` | Only meaningful with `ids::1`; absorbed from `.list` |

### Referenced User Stories
| # | User Story | Persona |
|---|------------|---------|
| 4 | [Query Storage Programmatically](../user_story/004_query_storage_programmatically.md) | developer |
