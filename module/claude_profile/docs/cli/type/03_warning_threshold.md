# Type :: 3. `WarningThreshold`

**Purpose:** Configures the boundary between `Valid` and `ExpiringSoon` token classification. Allows callers to tune early-warning sensitivity for automation or interactive use.

**Fundamental Type:** Newtype wrapping `u64` (seconds)

```rust
pub struct WarningThreshold( u64 );
```

**Constants:**
- `DEFAULT = 3600` — 60 minutes (matches `token::WARNING_THRESHOLD_SECS`)
- `MIN = 0` — never classify as ExpiringSoon

**Constraints:**
- Non-negative integer (unsigned, so always non-negative)
- No upper bound (any u64 value accepted)

**Parsing:**

```rust
impl WarningThreshold
{
  pub fn new( s : &str ) -> Result< Self, String >
  {
    let secs : u64 = s.parse()
      .map_err( | _ | format!( "invalid threshold '{}' — expected seconds as integer", s ) )?;
    Ok( Self( secs ) )
  }
}
```

**Methods:**
- `get() -> u64` — raw seconds value
- `as_duration() -> Duration` — converts to `std::time::Duration`
- `is_disabled() -> bool` — true when threshold is 0

**Parameters:** [`threshold::`](../param/03_threshold.md)

**Commands:** [`.token.status`](../command/token.md#command--7-tokenstatus)
