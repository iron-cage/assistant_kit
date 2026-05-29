//! Parameter parsing for the `.usage` command.
//!
//! `parse_usage_params` extracts and validates all flag/strategy values from
//! the incoming `VerifiedCommand`, returning a populated `UsageParams`.

use unilang::semantic::VerifiedCommand;
use unilang::types::Value;
use unilang::data::{ ErrorCode, ErrorData };
use super::types::{ UsageParams, SortStrategy, PreferStrategy, NextStrategy, ColsVisibility, SubprocessModel, SubprocessEffort, UsageOutputFormat, GetField };

// ── Parameter parser ──────────────────────────────────────────────────────────

/// Parse and validate the `.usage`-specific parameters.
///
/// # Errors
///
/// Returns `ErrorData` (exit 1 / `ArgumentTypeMismatch`) for any out-of-range
/// or wrong-type value. `interval` and `jitter` constraint validation is deferred
/// to `usage_routine` because it only applies when `live = 1`.
///
/// Fix(issue-155): `refresh` default is 1 (enabled). Omitting the param ≠
/// "user wants disabled" — auto-refresh is the safer default.
/// Fix(issue-157): strict 0/1 range guard added for `refresh`, `live`, `trace`.
/// Pitfall: bool-typed params (e.g. `touch::`) use `Kind::String` registration so
/// `"true"`/`"false"` pass through; `crate::output::parse_int_flag` is the sole normalisation point.
#[ allow( clippy::too_many_lines ) ]
pub( super ) fn parse_usage_params( cmd : &VerifiedCommand ) -> Result< UsageParams, ErrorData >
{
  // refresh default is 1 (enabled); live/trace default is 0 (disabled); touch default is 1 (enabled).
  let refresh = crate::output::parse_int_flag( cmd, "refresh", 1 )?;
  let live    = crate::output::parse_int_flag( cmd, "live",    0 )?;
  let trace   = crate::output::parse_int_flag( cmd, "trace",   0 )? != 0;
  let touch   = crate::output::parse_int_flag( cmd, "touch",   1 )?;
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
  let next = match cmd.arguments.get( "next" )
  {
    None                       => NextStrategy::Renew,
    Some( Value::String( s ) ) => NextStrategy::parse( s ).map_err( |e| ErrorData::new( ErrorCode::ArgumentTypeMismatch, e ) )?,
    _ => return Err( ErrorData::new(
      ErrorCode::ArgumentTypeMismatch,
      "next:: must be a string".to_string(),
    ) ),
  };
  // sort::next delegates to the active next:: strategy so the → winner always appears first.
  let sort = match sort
  {
    SortStrategy::Next => match next
    {
      NextStrategy::Renew     => SortStrategy::Renew,
      NextStrategy::Drain     => SortStrategy::Drain,
      NextStrategy::Endurance => SortStrategy::Endurance,
    },
    other => other,
  };
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
  Ok( UsageParams
  {
    refresh, live, interval, jitter, trace, sort, desc : desc_param, prefer, next, cols, touch, imodel, effort,
    count, offset, only_active, only_next, min_5h : h5_min, min_7d : d7_min, only_valid, exclude_exhausted,
    format, get, abs, no_color,
  } )
}
