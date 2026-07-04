# Parameter :: 20. `strategy::`

### Scope

- **Purpose**: Specify the `strategy::` CLI parameter.
- **Responsibility**: Type, defaults, valid values, and command usage for `strategy::`.
- **In Scope**: Value constraints, default behavior, command interactions.
- **Out of Scope**: Type definitions (→ `type/`), command behavior (→ `command/`).

Resume strategy override for `.session.ensure`.

**Type:** [`StrategyType`](../type/13_strategy_type.md)

**Fundamental Type:** String enum wrapper

**Constraints:**
- Valid values: `resume`, `fresh`
- Case-insensitive on parse
- Error on invalid: `"strategy must be resume|fresh, got {value}"`

**Default:** auto-detect (from conversation history presence)

**Commands:** `.session.ensure`

**Purpose:** Forces the reported resume strategy instead of auto-detecting it from storage history. When `strategy::resume` is forced and no history exists, the command still reports `resume` (the caller's intent is respected). When `strategy::fresh` is forced and history exists, `fresh` is reported regardless.

**Examples:**
```bash
# Valid values
strategy::resume    # Force resume strategy
strategy::fresh     # Force fresh strategy

# Invalid values
strategy::auto      # "strategy must be resume|fresh, got auto"
strategy::restart   # "strategy must be resume|fresh, got restart"
```

### Referenced Type
| Type | Kind | Fundamental | Key Constraint |
|------|------|-------------|----------------|
| [`StrategyType`](../type/13_strategy_type.md) | String enum wrapper | String | `resume` or `fresh`; case-insensitive |

### Referenced Commands
| # | Command | Default | Notes |
|---|---------|---------|-------|
| 11 | [`.session.ensure`](../command/11_session_ensure.md) | auto-detect | Overrides auto-detected resume strategy |

### Referenced User Stories
| # | User Story | Persona |
|---|------------|---------|
| 5 | [Resume Claude Session](../user_story/005_resume_claude_session.md) | developer |
