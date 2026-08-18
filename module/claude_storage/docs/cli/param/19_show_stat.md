# Parameter :: 19. `show_stat::`

### Scope

- **Purpose**: Specify the `show_stat::` CLI parameter.
- **Responsibility**: Type, defaults, valid values, and command usage for `show_stat::`.
- **In Scope**: Value constraints, default behavior, command interactions.
- **Out of Scope**: Type definitions (→ `type/`), command behavior (→ `command/`).

Accepted for backward compatibility; has no effect on `.show` output.

**Type:** Boolean

**Fundamental Type:** Boolean

**Default:** `0`

**Commands:** `.show`

**Purpose:** Accepted for backward compatibility but has no effect — content mode's key:val attribute block already shows total entry count, user/assistant breakdown, and timestamp range unconditionally, and `show_metadata::1` mode has always shown the same structured fields. Independent of `show_tokens::`.

**Examples:**
```bash
show_stat::0    # Default — no effect (parameter accepted, ignored)
show_stat::1    # No effect — content already shows the equivalent fields unconditionally
```

**Group:** [Output Control](../param_group/01_output_control.md)

### Referenced Type
| Type | Kind | Fundamental | Key Constraint |
|------|------|-------------|----------------|
| Boolean | Base type | Boolean flag | `0` (false) or `1` (true) |

### Referenced Parameter Groups
| # | Group | Membership | Co-members |
|---|-------|------------|------------|
| 1 | [Output Control](../param_group/01_output_control.md) | Full | `show_tokens::`, `show_tree::`, `show_topic::` |

### Referenced Commands
| # | Command | Default | Notes |
|---|---------|---------|-------|
| 3 | [`.show`](../command/03_show.md) | `0` | No effect — accepted for backward compatibility |

### Referenced User Stories
| # | User Story | Persona |
|---|------------|---------|
| 2 | [Find Past Conversation](../user_story/002_find_past_conversation.md) | developer |
