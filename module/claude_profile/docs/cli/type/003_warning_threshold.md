# Type: 3. `WarningThreshold`

**Purpose:** Configures the boundary between `Valid` and `ExpiringSoon` token classification. Allows callers to tune early-warning sensitivity for automation or interactive use.

**Fundamental Type:** Newtype wrapping `u64` (seconds)

**Constants:**
- `DEFAULT = 3600` — 60 minutes (matches `token::WARNING_THRESHOLD_SECS`)
- `MIN = 0` — never classify as ExpiringSoon

**Constraints:**
- Non-negative integer (unsigned, so always non-negative)
- No upper bound (any u64 value accepted)

**Parsing:**

```
pub fn new( s : &str ) -> Result< Self, String >
```

**Methods:**
- `get() -> u64` — raw seconds value
- `as_duration() -> Duration` — converts to `std::time::Duration`
- `is_disabled() -> bool` — true when threshold is 0

### Referenced Parameters

| # | Parameter | Role |
|---|-----------|------|
| 1 | [`threshold::`](../param/003_threshold.md) | Accepts warning threshold in seconds |

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | [`.token.status`](../command/005_token.md#command-7-tokenstatus) | Token expiry classification with this threshold |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 1 | [Credential Diagnostics](../user_story/005_credential_diagnostics.md) | Threshold tunes Valid / ExpiringSoon boundary |
