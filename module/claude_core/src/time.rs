//! UTC timestamp utilities for diagnostic trace lines.
//!
//! Pure stdlib — no external crate dependencies.

/// Returns the current UTC time as a `"YYYY-MM-DDTHH:MM:SSZ"` ISO-8601 string.
///
/// Uses only [`std::time::SystemTime`] — no external crate dependencies.
/// Resolution is one second; sub-second precision is intentionally omitted.
#[ must_use ]
#[ inline ]
pub fn chrono_now_utc() -> String
{
  use std::time::{ SystemTime, UNIX_EPOCH };
  let secs = SystemTime::now().duration_since( UNIX_EPOCH ).unwrap_or_default().as_secs();
  // 86400 secs/day, days since epoch → year/month/day via civil calendar algorithm
  #[ allow( clippy::cast_possible_wrap ) ]
  let days = ( secs / 86400 ) as i64;
  let tod  = secs % 86400;
  let hh   = tod / 3600;
  let mm   = ( tod % 3600 ) / 60;
  let ss   = tod % 60;
  // Euclidean affine conversion from rata die to Y/M/D (Howard Hinnant algorithm).
  let z   = days + 719_468;
  let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
  let doe = z - era * 146_097;
  let yoe = ( doe - doe / 1460 + doe / 36524 - doe / 146_096 ) / 365;
  let y   = yoe + era * 400;
  let doy = doe - ( 365 * yoe + yoe / 4 - yoe / 100 );
  let mp  = ( 5 * doy + 2 ) / 153;
  let d   = doy - ( 153 * mp + 2 ) / 5 + 1;
  let m   = if mp < 10 { mp + 3 } else { mp - 9 };
  let y   = if m <= 2 { y + 1 } else { y };
  format!( "{y:04}-{m:02}-{d:02}T{hh:02}:{mm:02}:{ss:02}Z" )
}

/// Returns a UTC timestamp prefix string for diagnostic trace lines.
///
/// Format: `"YYYY-MM-DD · HH:MM:SS UTC · "` — two middle dots separate date,
/// time, and body. Use as the first argument in `eprintln!`:
///
/// ```
/// # use claude_core::trace_ts;
/// eprintln!( "{}gate-wait  active=1/6", trace_ts() );
/// ```
///
/// The trailing space after the final `·` lets the caller append a label
/// directly without an extra space.
#[ inline ]
#[ must_use ]
pub fn trace_ts() -> String
{
  let utc = chrono_now_utc();
  // chrono_now_utc produces "YYYY-MM-DDTHH:MM:SSZ"; slice date and time parts.
  format!( "{} · {} UTC · ", &utc[ ..10 ], &utc[ 11..19 ] )
}
