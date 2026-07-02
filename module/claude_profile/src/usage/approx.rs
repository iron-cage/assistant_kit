//! Quota measurement polynomial approximation.
//!
//! Given a slice of timestamped utilization measurements, fits a polynomial
//! (degree 0, 1, or 2) and extrapolates the current utilization value.
//! Used as fallback when the usage API is unavailable (Feature 040).
//!
//! # Algorithm summary (Feature 039 Table 6)
//!
//! | Post-filter count | Degree | Method |
//! |---|---|---|
//! | 0 | — | `None` |
//! | 1 | 0 (constant) | Last value |
//! | 2 | 1 (linear) | Extrapolate from slope |
//! | 3–10 | 2 (quadratic) | Least-squares via 3×3 Cramer |
//!
//! Pre-fit: discard measurements before `window_start = resets_at - window_duration`.
//! If `now_secs > resets_at_secs` → return `0.0` (window expired, AC-07).
//!
//! Post-fit: clamp to `[0.0, 100.0]` (AC-08). Tangent-line continuation when
//! `t_now` exceeds 2× the measurement span beyond the last point (AC-09).
//!
//! Time normalization: subtract `t_values[0]` before computing power sums to
//! avoid f64 precision loss on large Unix timestamps (~1.75 × 10⁹, AC).
//!
//! # Known Pitfalls
//!
//! **Cramer cofactor index confusion (BUG-307):** Each of `det2`/`det1`/`det0` replaces a
//! DIFFERENT column of the normal matrix with the RHS vector `[r2, r1, r0]^T`. Cofactor
//! terms for `det0` (col-3 replaced) involve `r1` from the RHS — never `r2` (the power-sum
//! variable). Copying cofactor terms across `det2`/`det1`/`det0` without re-deriving silently
//! produces wrong coefficients. AC-08 clamping can further mask the error for upward-trending
//! data by returning 100.0 (clamped from an overshot wrong estimate). Always re-derive each
//! minor independently from the correct column-replaced matrix.

/// Approximate current utilization from historical measurements.
///
/// # Parameters
/// - `measurements`: `(unix_seconds, utilization_0_to_100)` pairs from history ring buffer.
/// - `resets_at_secs`: optional Unix timestamp when the current window resets.
/// - `window_duration_s`: seconds in the current window (18000 for 5h, 604800 for 7d).
/// - `now_secs`: current Unix timestamp.
///
/// # Returns
/// - `None` when no data points remain after window filtering.
/// - `Some(0.0)` when the window has already expired (`now_secs > resets_at_secs`).
/// - `Some(value)` clamped to `[0.0, 100.0]` otherwise.
#[ must_use ]
#[ inline ]
pub fn approximate_utilization(
  measurements      : &[ ( u64, f64 ) ],
  resets_at_secs    : Option< u64 >,
  window_duration_s : u64,
  now_secs          : u64,
) -> Option< f64 >
{
  // AC-07: expired window — utilization resets to 0.0.
  if let Some( r ) = resets_at_secs
  {
    if now_secs > r { return Some( 0.0 ); }
  }

  // AC-06: filter measurements outside the current window.
  let window_start = resets_at_secs.map( |r| r.saturating_sub( window_duration_s ) );
  let pts : Vec< ( u64, f64 ) > = measurements.iter()
    .filter( |( t, _ )| window_start.map_or( true, |ws| *t >= ws ) )
    .copied()
    .collect();

  match pts.len()
  {
    0 => None,
    1 => Some( pts[ 0 ].1.clamp( 0.0, 100.0 ) ),
    2 => Some( linear_extrapolate( &pts, now_secs ) ),
    _ => Some( quadratic_fit( &pts, now_secs ) ),
  }
}

/// Linear extrapolation through exactly 2 points, or any set treated as degree-1.
#[ allow( dead_code ) ]
fn linear_extrapolate( pts : &[ ( u64, f64 ) ], now_secs : u64 ) -> f64
{
  let t0   = pts[ 0 ].0;
  let dt   = pts[ 1 ].0.saturating_sub( t0 ) as f64;
  let t_now = now_secs.saturating_sub( t0 ) as f64;
  if dt.abs() < 1e-12
  {
    // Degenerate: identical timestamps — return last value.
    return pts.last().map_or( 0.0, |p| p.1 ).clamp( 0.0, 100.0 );
  }
  let slope = ( pts[ 1 ].1 - pts[ 0 ].1 ) / dt;
  let y     = pts[ 0 ].1 + slope * t_now;
  y.clamp( 0.0, 100.0 )
}

/// Quadratic least-squares fit via 3×3 Cramer's rule.
///
/// Time normalization: subtract `t_values[0]` before power-sum computation
/// to keep magnitudes small and avoid f64 precision loss (AC).
#[ allow( dead_code ) ]
fn quadratic_fit( pts : &[ ( u64, f64 ) ], now_secs : u64 ) -> f64
{
  let t0      = pts[ 0 ].0;
  let t_now_f = now_secs.saturating_sub( t0 ) as f64;

  // Normalized timestamps and utilization values.
  let n = pts.len() as f64;
  let mut s1  = 0_f64; // Σ ti
  let mut s2  = 0_f64; // Σ ti²
  let mut s3  = 0_f64; // Σ ti³
  let mut s4  = 0_f64; // Σ ti⁴
  let mut r0  = 0_f64; // Σ yi
  let mut r1  = 0_f64; // Σ ti·yi
  let mut r2  = 0_f64; // Σ ti²·yi
  let mut t_max = 0_f64;
  let mut t_min = 0_f64;

  for ( i, &( t, y ) ) in pts.iter().enumerate()
  {
    let ti = t.saturating_sub( t0 ) as f64;
    let ti2 = ti * ti;
    s1  += ti;
    s2  += ti2;
    s3  += ti2 * ti;
    s4  += ti2 * ti2;
    r0  += y;
    r1  += ti * y;
    r2  += ti2 * y;
    if i == 0 { t_min = ti; t_max = ti; }
    else
    {
      if ti > t_max { t_max = ti; }
      if ti < t_min { t_min = ti; }
    }
  }

  // 3×3 normal equation: [[s4,s3,s2],[s3,s2,s1],[s2,s1,n]] · [a2,a1,a0] = [r2,r1,r0]
  // Cramer's rule for a2.
  let det = s4 * ( s2 * n - s1 * s1 ) - s3 * ( s3 * n - s1 * s2 ) + s2 * ( s3 * s1 - s2 * s2 );

  let ( a2, a1, a0 ) = if det.abs() < 1e-12
  {
    // AC-10: singular matrix — fall back to linear with 2 endpoints.
    let first = pts[ 0 ];
    let last  = pts[ pts.len() - 1 ];
    let fallback_pts = [ first, last ];
    let y = linear_extrapolate( &fallback_pts, now_secs );
    return y;
  }
  else
  {
    // Cramer's rule: det_a2, det_a1, det_a0.
    // Fix(BUG-307): det0 cofactor used s1*r2 instead of s2*r1 — wrong column-3 Cramer minor.
    // Root cause: cofactor(1,2) of det0 replaces col-3 with RHS; minor is s3*r0-s2*r1, not s3*r0-s1*r2 (s1 is col-2; s2 is col-1 in the replaced-column row).
    // Pitfall: Cramer minors for det2/det1/det0 each replace a DIFFERENT column — never copy cofactor terms across them; re-derive each minor independently.
    let det2 = r2 * ( s2 * n - s1 * s1 ) - s3 * ( r1 * n - s1 * r0 ) + s2 * ( r1 * s1 - s2 * r0 );
    let det1 = s4 * ( r1 * n - s1 * r0 ) - r2 * ( s3 * n - s1 * s2 ) + s2 * ( s3 * r0 - s2 * r1 );
    let det0 = s4 * ( s2 * r0 - s1 * r1 ) - s3 * ( s3 * r0 - s2 * r1 ) + r2 * ( s3 * s1 - s2 * s2 );
    ( det2 / det, det1 / det, det0 / det )
  };

  // AC-09: tangent-line continuation when t_now > t_max + 2·span.
  let span = t_max - t_min;
  let y = if span > 0.0 && ( t_now_f - t_max ) > 2.0 * span
  {
    let slope    = 2.0 * a2 * t_max + a1;
    let y_at_max = a2 * t_max * t_max + a1 * t_max + a0;
    y_at_max + slope * ( t_now_f - t_max )
  }
  else
  {
    a2 * t_now_f * t_now_f + a1 * t_now_f + a0
  };

  // AC-08: clamp to [0.0, 100.0].
  y.clamp( 0.0, 100.0 )
}


// Tests live in tests/usage/approx_tests.rs (integration tests via test_bridge).
