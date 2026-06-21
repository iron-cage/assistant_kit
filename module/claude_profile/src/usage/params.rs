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
pub( super ) fn parse_usage_params( cmd : &VerifiedCommand ) -> Result< UsageParams, ErrorData >
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

// ── Tests ─────────────────────────────────────────────────────────────────────

#[ cfg( test ) ]
mod tests
{
  use super::*;
  use std::collections::HashMap;
  use unilang::data::CommandDefinition;
  use unilang::semantic::VerifiedCommand;
  use unilang::types::Value;

  fn make_cmd( args : Vec< ( &str, Value ) > ) -> VerifiedCommand
  {
    let mut arguments = HashMap::new();
    for ( k, v ) in args { arguments.insert( k.to_string(), v ); }
    let definition = CommandDefinition::former().name( ".usage" ).description( "test" ).end();
    VerifiedCommand { definition, arguments }
  }

  /// PHASE-1 RED GATE: fails to compile until `UsageParams` gains `rotate: bool` in Phase 2.
  /// After Phase 2 this test verifies that `rotate::1` is parsed to `params.rotate == true`.
  #[ test ]
  fn rotate_field_parses_true()
  {
    let cmd    = make_cmd( vec![ ( "rotate", Value::Integer( 1 ) ) ] );
    let params = parse_usage_params( &cmd ).expect( "parse must succeed" );
    assert!( params.rotate, "rotate::1 must set params.rotate to true" );
  }

  /// After Phase 2: `rotate::0` (explicit disable) parses to `params.rotate == false`.
  #[ test ]
  fn rotate_field_parses_false()
  {
    let cmd    = make_cmd( vec![ ( "rotate", Value::Integer( 0 ) ) ] );
    let params = parse_usage_params( &cmd ).expect( "parse must succeed" );
    assert!( !params.rotate, "rotate::0 must set params.rotate to false" );
  }

  /// After Phase 2: omitting `rotate::` defaults to `params.rotate == false`.
  #[ test ]
  fn rotate_field_default_false()
  {
    let cmd    = make_cmd( vec![] );
    let params = parse_usage_params( &cmd ).expect( "parse must succeed" );
    assert!( !params.rotate, "default rotate must be false" );
  }

  /// After Phase 2: `force::1` is parsed to `params.force == true`.
  #[ test ]
  fn force_field_parses_true()
  {
    let cmd    = make_cmd( vec![ ( "force", Value::String( "1".to_string() ) ) ] );
    let params = parse_usage_params( &cmd ).expect( "parse must succeed" );
    assert!( params.force, "force::1 must set params.force to true" );
  }

  /// Phase 1: omitting `who::` defaults to `params.who == None` (auto).
  #[ test ]
  fn who_field_default_is_none()
  {
    let cmd    = make_cmd( vec![] );
    let params = parse_usage_params( &cmd ).expect( "parse must succeed" );
    assert_eq!( params.who, None, "default who must be None (auto)" );
  }

  /// Phase 1: `who::1` parses to `params.who == Some(true)` (force on).
  #[ test ]
  fn who_field_parses_one_to_some_true()
  {
    let cmd    = make_cmd( vec![ ( "who", Value::Integer( 1 ) ) ] );
    let params = parse_usage_params( &cmd ).expect( "parse must succeed" );
    assert_eq!( params.who, Some( true ), "who::1 must set params.who to Some(true)" );
  }

  /// Phase 1: `who::0` parses to `params.who == Some(false)` (suppress).
  #[ test ]
  fn who_field_parses_zero_to_some_false()
  {
    let cmd    = make_cmd( vec![ ( "who", Value::Integer( 0 ) ) ] );
    let params = parse_usage_params( &cmd ).expect( "parse must succeed" );
    assert_eq!( params.who, Some( false ), "who::0 must set params.who to Some(false)" );
  }

  /// EC-7 — `who::2` is rejected; integer outside {0, 1} triggers `ArgumentTypeMismatch` via `parse_int_flag`.
  ///
  /// `parse_int_flag` only succeeds for `Value::Integer(0)` and `Value::Integer(1)`;
  /// `Value::Integer(2)` hits the error arm and `parse_usage_params` propagates the failure.
  ///
  /// Spec: [`tests/docs/cli/param/62_who.md` EC-7]
  #[ test ]
  fn ec7_who_rejects_integer_two()
  {
    let cmd    = make_cmd( vec![ ( "who", Value::Integer( 2 ) ) ] );
    let result = parse_usage_params( &cmd );
    assert!(
      result.is_err(),
      "EC-7: who::2 must be rejected (integer outside {{0, 1}}); got Ok",
    );
  }

  /// EC-1 / default: omitting `solo::` defaults to `params.solo == false` (conservation off).
  ///
  /// Spec: [`tests/docs/cli/param/61_solo.md` EC-1]
  #[ test ]
  fn solo_field_default_false()
  {
    let cmd    = make_cmd( vec![] );
    let params = parse_usage_params( &cmd ).expect( "parse must succeed" );
    assert!( !params.solo, "default solo must be false (token conservation off)" );
  }

  /// EC-2 subset: `solo::1` parses to `params.solo == true`.
  ///
  /// Spec: [`tests/docs/cli/param/61_solo.md` EC-2]
  #[ test ]
  fn solo_field_parses_true()
  {
    let cmd    = make_cmd( vec![ ( "solo", Value::Integer( 1 ) ) ] );
    let params = parse_usage_params( &cmd ).expect( "parse must succeed" );
    assert!( params.solo, "solo::1 must set params.solo to true" );
  }

  /// `solo::0` parses to `params.solo == false` (explicit off).
  #[ test ]
  fn solo_field_parses_false()
  {
    let cmd    = make_cmd( vec![ ( "solo", Value::Integer( 0 ) ) ] );
    let params = parse_usage_params( &cmd ).expect( "parse must succeed" );
    assert!( !params.solo, "solo::0 must set params.solo to false" );
  }

  /// EC-5: `solo::1 rotate::1` → rejected (mutual exclusion).
  ///
  /// `solo::1` restricts fetch to the current+owned account (approximated data for others);
  /// `rotate::1` requires live fetch for all accounts to compare winner candidates.
  /// Combining them is self-contradictory.
  ///
  /// Spec: [`tests/docs/cli/param/61_solo.md` EC-5]
  #[ test ]
  fn ec5_solo_and_rotate_mutual_exclusion()
  {
    let cmd = make_cmd( vec![
      ( "solo",   Value::Integer( 1 ) ),
      ( "rotate", Value::Integer( 1 ) ),
    ] );
    let result = parse_usage_params( &cmd );
    assert!( result.is_err(), "EC-5: solo::1 + rotate::1 must return Err (mutual exclusion)" );
    let err_msg = result.unwrap_err().message;
    assert!(
      err_msg.contains( "solo" ) && err_msg.contains( "rotate" ),
      "EC-5: error message must reference both params, got: {err_msg}",
    );
  }

  /// EC-12: `solo::2` is rejected (integer outside {0, 1}).
  ///
  /// `parse_int_flag` accepts only `Value::Integer(0)` and `Value::Integer(1)`;
  /// `Value::Integer(2)` hits the `_` error arm and `parse_usage_params` propagates the failure.
  ///
  /// Spec: [`tests/docs/cli/param/61_solo.md` EC-12]
  #[ test ]
  fn ec12_solo_rejects_integer_two()
  {
    let cmd    = make_cmd( vec![ ( "solo", Value::Integer( 2 ) ) ] );
    let result = parse_usage_params( &cmd );
    assert!(
      result.is_err(),
      "EC-12: solo::2 must be rejected (integer outside {{0, 1}}); got Ok",
    );
  }

  /// After Phase 2: mutual exclusion guard rejects `rotate::1 live::1`.
  #[ test ]
  fn rotate_and_live_mutual_exclusion()
  {
    let cmd = make_cmd( vec![
      ( "rotate", Value::Integer( 1 ) ),
      ( "live",   Value::Integer( 1 ) ),
    ] );
    let result = parse_usage_params( &cmd );
    assert!( result.is_err(), "rotate::1 + live::1 must return Err (mutual exclusion)" );
    let err_msg = result.unwrap_err().message;
    assert!(
      err_msg.contains( "rotate" ) && err_msg.contains( "live" ),
      "error message must reference both params, got: {err_msg}",
    );
  }
}
