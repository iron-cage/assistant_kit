# Type: 3. `WarningThreshold`

**Purpose:** Configures the boundary between `Valid` and `ExpiringSoon` token classification. Allows callers to tune early-warning sensitivity for automation or interactive use.

**Fundamental Type:** Raw `u64` (seconds) — no dedicated wrapper type.

**Constants:**
- `DEFAULT = 3600` — 60 minutes (matches `token::WARNING_THRESHOLD_SECS`)
- `MIN = 0` — never classify as ExpiringSoon

**Constraints:**
- Non-negative integer (unsigned, so always non-negative)
- No upper bound (any u64 value accepted)

**Parsing:**

No dedicated parser — parsed inline where consumed, from the unilang `Value::Integer` argument:

```
let threshold_secs = match cmd.arguments.get( "threshold" )
{
  Some( Value::Integer( n ) ) => u64::try_from( *n ).unwrap_or( crate::token::WARNING_THRESHOLD_SECS ),
  _                           => crate::token::WARNING_THRESHOLD_SECS,
};
```

(`src/commands/credentials.rs`) — an out-of-range or absent value falls back to `WARNING_THRESHOLD_SECS` (3600), never an error.

**Methods:**
- No methods exist — the raw `u64` is passed directly to `token::status_with_threshold( warning_secs : u64 )` / `token::classify_ms( expires_at_ms, warning_secs )`, which compare `remaining.as_secs() <= warning_secs` inline.
- No `as_duration()`/`is_disabled()`/`get()` methods exist.

### Referenced Parameters

| # | Parameter | Role |
|---|-----------|------|
| 1 | [`threshold::`](../param/003_threshold.md) | Accepts warning threshold in seconds |

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | [`.credentials.status`](../command/002_credentials.md#command-10-credentialsstatus) | Token expiry classification with this threshold |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 1 | [Credential Diagnostics](../user_story/005_credential_diagnostics.md) | Threshold tunes Valid / ExpiringSoon boundary |
