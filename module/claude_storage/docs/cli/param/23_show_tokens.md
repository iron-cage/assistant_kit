# Parameter :: 23. `show_tokens::`

### Scope

- **Purpose**: Specify the `show_tokens::` CLI parameter.
- **Responsibility**: Type, defaults, valid values, and command usage for `show_tokens::`.
- **In Scope**: Value constraints, default behavior, command interactions.
- **Out of Scope**: Type definitions (→ `type/`), command behavior (→ `command/`).

Show token usage statistics section.

**Type:** Boolean

**Fundamental Type:** Boolean

**Default:** `0`

**Commands:** `.show`, `.status`

**Purpose:** When set to `1`, includes a token usage breakdown in the output — input tokens, output tokens, cache read tokens, and cache creation tokens.

In `.status`: triggers full JSONL parsing of all session files to compute token totals. On large storage (thousands of sessions / GB of JSONL) this can take minutes. Default fast path uses filesystem-only stats and completes in under a second.

In `.show`: appends token usage to session output (metadata or content mode).

**Examples:**
```bash
show_tokens::0    # Default — no token usage section
show_tokens::1    # Include token usage section
```

**Group:** [Output Control](../param_group/01_output_control.md)

### Referenced Type
| Type | Kind | Fundamental | Key Constraint |
|------|------|-------------|----------------|
| Boolean | Base type | Boolean flag | `0` (false) or `1` (true) |

### Referenced Parameter Groups
| # | Group | Membership | Co-members |
|---|-------|------------|------------|
| 1 | [Output Control](../param_group/01_output_control.md) | Full | `show_stat::`, `show_tree::` |

### Referenced Commands
| # | Command | Default | Notes |
|---|---------|---------|-------|
| 1 | [`.status`](../command/01_status.md) | `0` | Triggers slow JSONL parse for token totals |
| 3 | [`.show`](../command/03_show.md) | `0` | Appends token usage to session output |

### Referenced User Stories
| # | User Story | Persona |
|---|------------|---------|
| 1 | [Audit Session History](../user_story/001_audit_session_history.md) | developer |
