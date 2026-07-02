//! Parameter parsing for the `.usage` command.
//!
//! `parse_usage_params` extracts and validates all flag/strategy values from
//! the incoming `VerifiedCommand`, returning a populated `UsageParams`.

use unilang::semantic::VerifiedCommand;
use unilang::types::Value;
use unilang::data::{ ErrorCode, ErrorData };
use super::types::{ UsageParams, SortStrategy, PreferStrategy, ColsVisibility, SubprocessModel, SubprocessEffort, UsageOutputFormat, GetField, validate_set_model };

// ── Parameter parser ──────────────────────────────────────────────────────────

/// Parse and validate the `.usage`-specific parameters.
///
/// # Errors
///
/// Returns `ErrorData` (exit 1 / `ArgumentTypeMismatch`) for any out-of-range
/// or wrong-type value. `interval` and `jitter` constraint validation is deferred
/// to `usage_routine` because it only applies when `live = 1`.
///
/// Fix(BUG-155): `refresh` default is 1 (enabled). Omitting the param ≠
/// "user wants disabled" — auto-refresh is the safer default.
/// Root cause: original default was 0 (disabled), causing silent no-refresh behaviour
///   when users omitted `refresh::` — the unexpected opt-in requirement caused confusion.
/// Fix(BUG-272): strict 0/1 range guard added for `refresh`, `live`, `trace`.
/// Root cause: without range validation, values like `refresh::2` silently mapped to
///   truthy and were accepted as valid booleans, masking user typos.
/// Pitfall: bool-typed params (e.g. `touch::`) use `Kind::String` registration so
/// `"true"`/`"false"` pass through; `crate::output::parse_int_flag` is the sole normalisation point.
#[ allow( clippy::too_many_lines ) ]
#[ inline ]
pub fn parse_usage_params( cmd : &VerifiedCommand ) -> Result< UsageParams, ErrorData >
{
  // refresh default is 1 (enabled); live/trace/rotate default is 0 (disabled); touch default is 1 (enabled).
  let refresh = crate::output::parse_int_flag( cmd, "refresh", 1 )?;
  let live    = crate::output::parse_int_flag( cmd, "live",    0 )?;
  let trace   = crate::output::parse_int_flag( cmd, "trace",   0 )? != 0;
  let touch   = crate::output::parse_int_flag( cmd, "touch",   1 )?;
  let rotate  = crate::output::parse_int_flag( cmd, "rotate",  0 )? != 0;
  // who:: tri-state: -1 = auto (None), 0 = suppress (Some(false)), 1 = force on (Some(true)).
  let who = match crate::output::parse_int_flag( cmd, "who", -1 )?
  {
    -1 => None,
    0  => Some( false ),
    1  => Some( true ),
    _  => unreachable!(), // parse_int_flag only returns -1, 0, or 1
  };
  // P-2: force is a strict-bool string param ("0"/"1"); default 0.
  let force = match cmd.arguments.get( "force" )
  {
    None                       => false,
    Some( Value::String( s ) ) => match s.as_str()
    {
      "1" | "true"  => true,
      "0" | "false" => false,
      other => return Err( ErrorData::new(
        ErrorCode::ArgumentTypeMismatch,
        format!( "force:: must be 0 or 1, got {other:?}" ),
      ) ),
    },
    _ => return Err( ErrorData::new(
      ErrorCode::ArgumentTypeMismatch,
      "force:: must be a string (0 or 1)".to_string(),
    ) ),
  };
  let solo = crate::output::parse_int_flag( cmd, "solo", 0 )? != 0;
  // Solo+rotate mutual exclusion: solo::1 is token-conservation mode — a one-shot
  // display with approximated data; rotate::1 requires live fetch to pick a winner.
  if solo && rotate
  {
    return Err( ErrorData::new(
      ErrorCode::ArgumentTypeMismatch,
      "solo::1 and rotate::1 are mutually exclusive".to_string(),
    ) );
  }
  // AC-04: rotate::1 and live::1 are mutually exclusive — rotation is a one-shot
  // action incompatible with the continuous-monitor loop.
  if rotate && live != 0
  {
    return Err( ErrorData::new(
      ErrorCode::ArgumentTypeMismatch,
      "rotate::1 and live::1 are mutually exclusive".to_string(),
    ) );
  }
  // Negative values map to 0, which is < 30 and will hit the interval guard.
  let interval = match cmd.arguments.get( "interval" )
  {
    None                        => 30_u64,
    Some( Value::Integer( n ) ) => u64::try_from( *n ).unwrap_or( 0 ),
    _ => return Err( ErrorData::new(
      ErrorCode::ArgumentTypeMismatch,
      "interval:: must be a non-negative integer".to_string(),
    ) ),
  };
  let jitter = match cmd.arguments.get( "jitter" )
  {
    None                        => 0_u64,
    Some( Value::Integer( n ) ) => u64::try_from( *n ).unwrap_or( 0 ),
    _ => return Err( ErrorData::new(
      ErrorCode::ArgumentTypeMismatch,
      "jitter:: must be a non-negative integer".to_string(),
    ) ),
  };
  let sort = match cmd.arguments.get( "sort" )
  {
    None                         => SortStrategy::Renew,
    Some( Value::String( s ) ) => SortStrategy::parse( s ).map_err( |e| ErrorData::new( ErrorCode::ArgumentTypeMismatch, e ) )?,
    _ => return Err( ErrorData::new(
      ErrorCode::ArgumentTypeMismatch,
      "sort:: must be a string".to_string(),
    ) ),
  };
  let desc_param = match cmd.arguments.get( "desc" )
  {
    None                        => None,
    Some( Value::Integer( 0 ) ) => Some( false ),
    Some( Value::Integer( 1 ) ) => Some( true ),
    _ => return Err( ErrorData::new(
      ErrorCode::ArgumentTypeMismatch,
      "desc:: must be 0 or 1".to_string(),
    ) ),
  };
  let prefer = match cmd.arguments.get( "prefer" )
  {
    None                       => PreferStrategy::Any,
    Some( Value::String( s ) ) => PreferStrategy::parse( s ).map_err( |e| ErrorData::new( ErrorCode::ArgumentTypeMismatch, e ) )?,
    _ => return Err( ErrorData::new(
      ErrorCode::ArgumentTypeMismatch,
      "prefer:: must be a string".to_string(),
    ) ),
  };
  // next:: parameter has been removed — sort:: now drives both row ordering and → recommendation.
  // Reject next:: explicitly so users receive a clear migration message.
  if cmd.arguments.contains_key( "next" )
  {
    return Err( ErrorData::new(
      ErrorCode::ArgumentTypeMismatch,
      "next:: parameter has been removed; use sort:: instead (valid values: `name`, `renew`, `renews`)".to_string(),
    ) );
  }
  let cols = match cmd.arguments.get( "cols" )
  {
    None                       => ColsVisibility::default_set(),
    Some( Value::String( s ) ) => ColsVisibility::parse( s ).map_err( |e| ErrorData::new( ErrorCode::ArgumentTypeMismatch, e ) )?,
    _ => return Err( ErrorData::new(
      ErrorCode::ArgumentTypeMismatch,
      "cols:: must be a string".to_string(),
    ) ),
  };
  let imodel = match cmd.arguments.get( "imodel" )
  {
    None                       => SubprocessModel::Auto,
    Some( Value::String( s ) ) => SubprocessModel::parse( s ).map_err( |e| ErrorData::new( ErrorCode::ArgumentTypeMismatch, e ) )?,
    _ => return Err( ErrorData::new(
      ErrorCode::ArgumentTypeMismatch,
      "imodel:: must be a string".to_string(),
    ) ),
  };
  let effort = match cmd.arguments.get( "effort" )
  {
    None                       => SubprocessEffort::Auto,
    Some( Value::String( s ) ) => SubprocessEffort::parse( s ).map_err( |e| ErrorData::new( ErrorCode::ArgumentTypeMismatch, e ) )?,
    _ => return Err( ErrorData::new(
      ErrorCode::ArgumentTypeMismatch,
      "effort:: must be a string".to_string(),
    ) ),
  };
  // ── Row filtering (TSK-223) ───────────────────────────────────────────────
  let count = match cmd.arguments.get( "count" )
  {
    None                        => 0_u64,
    Some( Value::Integer( n ) ) => u64::try_from( *n ).map_err( |_| ErrorData::new( ErrorCode::ArgumentTypeMismatch, "count:: must be a non-negative integer".to_string() ) )?,
    _ => return Err( ErrorData::new( ErrorCode::ArgumentTypeMismatch, "count:: must be a non-negative integer".to_string() ) ),
  };
  let offset = match cmd.arguments.get( "offset" )
  {
    None                        => 0_u64,
    Some( Value::Integer( n ) ) => u64::try_from( *n ).map_err( |_| ErrorData::new( ErrorCode::ArgumentTypeMismatch, "offset:: must be a non-negative integer".to_string() ) )?,
    _ => return Err( ErrorData::new( ErrorCode::ArgumentTypeMismatch, "offset:: must be a non-negative integer".to_string() ) ),
  };
  let only_active       = crate::output::parse_int_flag( cmd, "only_active",       0 )? != 0;
  let only_next         = crate::output::parse_int_flag( cmd, "only_next",         0 )? != 0;
  let only_valid        = crate::output::parse_int_flag( cmd, "only_valid",        0 )? != 0;
  let exclude_exhausted = crate::output::parse_int_flag( cmd, "exclude_exhausted", 0 )? != 0;
  let h5_min = match cmd.arguments.get( "min_5h" )
  {
    None                        => 0_u8,
    Some( Value::Integer( n ) ) =>
    {
      u8::try_from( *n ).map_err( |_| ErrorData::new( ErrorCode::ArgumentTypeMismatch, "min_5h:: must be 0–100".to_string() ) )
        .and_then( |v| if v <= 100 { Ok( v ) } else { Err( ErrorData::new( ErrorCode::ArgumentTypeMismatch, "min_5h:: must be 0–100".to_string() ) ) } )?
    }
    _ => return Err( ErrorData::new( ErrorCode::ArgumentTypeMismatch, "min_5h:: must be an integer 0–100".to_string() ) ),
  };
  let d7_min = match cmd.arguments.get( "min_7d" )
  {
    None                        => 0_u8,
    Some( Value::Integer( n ) ) =>
    {
      u8::try_from( *n ).map_err( |_| ErrorData::new( ErrorCode::ArgumentTypeMismatch, "min_7d:: must be 0–100".to_string() ) )
        .and_then( |v| if v <= 100 { Ok( v ) } else { Err( ErrorData::new( ErrorCode::ArgumentTypeMismatch, "min_7d:: must be 0–100".to_string() ) ) } )?
    }
    _ => return Err( ErrorData::new( ErrorCode::ArgumentTypeMismatch, "min_7d:: must be an integer 0–100".to_string() ) ),
  };
  // ── Format / extraction (TSK-224) ───────────────────────────────────────────
  let format = match cmd.arguments.get( "format" )
  {
    None                       => UsageOutputFormat::Text,
    Some( Value::String( s ) ) => match s.as_str()
    {
      "text"  => UsageOutputFormat::Text,
      "json"  => UsageOutputFormat::Json,
      "tsv"   => UsageOutputFormat::Tsv,
      "plain" => UsageOutputFormat::Plain,
      "value" => UsageOutputFormat::Value,
      "table" => return Err( ErrorData::new(
        ErrorCode::ArgumentTypeMismatch,
        "format::table is only supported by .accounts".to_string(),
      ) ),
      other   => return Err( ErrorData::new(
        ErrorCode::ArgumentTypeMismatch,
        format!( "unknown format {other:?}: valid values are `text`, `json`, `tsv`, `plain`, `value`" ),
      ) ),
    },
    _ => return Err( ErrorData::new(
      ErrorCode::ArgumentTypeMismatch,
      "format:: must be a string".to_string(),
    ) ),
  };
  let get = match cmd.arguments.get( "get" )
  {
    None                       => None,
    Some( Value::String( s ) ) => Some(
      GetField::parse( s ).map_err( |e| ErrorData::new( ErrorCode::ArgumentTypeMismatch, e ) )?
    ),
    _ => return Err( ErrorData::new(
      ErrorCode::ArgumentTypeMismatch,
      "get:: must be a string".to_string(),
    ) ),
  };
  let abs      = crate::output::parse_int_flag( cmd, "abs",      0 )? != 0;
  let no_color = crate::output::parse_int_flag( cmd, "no_color", 0 )? != 0;
  let set_model = match cmd.arguments.get( "set_model" )
  {
    None                       => None,
    Some( Value::String( s ) ) =>
    {
      validate_set_model( s ).map_err( |e| ErrorData::new( ErrorCode::ArgumentTypeMismatch, e ) )?;
      Some( s.clone() )
    }
    _ => return Err( ErrorData::new(
      ErrorCode::ArgumentTypeMismatch,
      "set_model:: must be a string".to_string(),
    ) ),
  };
  Ok( UsageParams
  {
    refresh, live, interval, jitter, trace, sort, desc : desc_param, prefer, cols, touch, imodel, effort,
    count, offset, only_active, only_next, min_5h : h5_min, min_7d : d7_min, only_valid, exclude_exhausted,
    format, get, abs, no_color, set_model,
    rotate, force, who, solo,
  } )
}


// Tests live in tests/usage/params_tests.rs (integration tests via test_bridge).
