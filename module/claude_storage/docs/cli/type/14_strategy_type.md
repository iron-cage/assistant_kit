# Type :: 14. `StrategyType`

**Purpose:** Resume strategy for `.session.ensure`. Determines whether a session should continue an existing conversation (`resume`) or start fresh (`fresh`).

**Fundamental Type:** Wrapper around string enum

**Constants:**
- RESUME = `"resume"` (continue existing conversation)
- FRESH = `"fresh"` (start a new conversation)
- DEFAULT = auto-detect (from conversation history presence)

**Constraints:**
- Valid values: `resume`, `fresh`
- Case-insensitive on parse
- Error on invalid: `"strategy must be resume|fresh, got {value}"`

**Parsing:**
```
Parse string to enum variant (case-insensitive):
  Input: "resume" → StrategyType::Resume
  Input: "fresh"  → StrategyType::Fresh
  Error: "strategy must be resume|fresh, got {value}"
```

**Auto-detection** (when not forced):
```
if check_continuation(session_dir):
  StrategyType::Resume
else:
  StrategyType::Fresh
```

**Methods:**
- `get() -> string` — Canonical lowercase variant name (`"resume"` or `"fresh"`)
- `is_resume() -> boolean` — True when strategy is Resume
- `is_fresh() -> boolean` — True when strategy is Fresh

**Commands:** `.session.ensure`

### Referenced Commands

| # | Command | Via Parameter |
|---|---------|---------------|
| 11 | [`.session.ensure`](../command/11_session_ensure.md) | `strategy::` |

### Referenced Parameters

| # | Parameter | Commands |
|---|-----------|----------|
| 20 | [`strategy::`](../param/20_strategy.md) | 1 |
