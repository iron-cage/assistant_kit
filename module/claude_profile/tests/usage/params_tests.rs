// Integration tests for params.rs — parse_usage_params parameter parsing.
// Accesses pub(crate) items through claude_profile::usage::test_bridge (testing feature).

use claude_profile::usage::test_bridge::parse_usage_params;
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

/// `rotate::1` parses to `params.rotate == true`.
#[ test ]
fn rotate_field_parses_true()
{
  let cmd    = make_cmd( vec![ ( "rotate", Value::Integer( 1 ) ) ] );
  let params = parse_usage_params( &cmd ).expect( "parse must succeed" );
  assert!( params.rotate, "rotate::1 must set params.rotate to true" );
}

/// `rotate::0` (explicit disable) parses to `params.rotate == false`.
#[ test ]
fn rotate_field_parses_false()
{
  let cmd    = make_cmd( vec![ ( "rotate", Value::Integer( 0 ) ) ] );
  let params = parse_usage_params( &cmd ).expect( "parse must succeed" );
  assert!( !params.rotate, "rotate::0 must set params.rotate to false" );
}

/// Omitting `rotate::` defaults to `params.rotate == false`.
#[ test ]
fn rotate_field_default_false()
{
  let cmd    = make_cmd( vec![] );
  let params = parse_usage_params( &cmd ).expect( "parse must succeed" );
  assert!( !params.rotate, "default rotate must be false" );
}

/// `force::1` parses to `params.force == true`.
#[ test ]
fn force_field_parses_true()
{
  let cmd    = make_cmd( vec![ ( "force", Value::String( "1".to_string() ) ) ] );
  let params = parse_usage_params( &cmd ).expect( "parse must succeed" );
  assert!( params.force, "force::1 must set params.force to true" );
}

/// Omitting `who::` defaults to `params.who == None` (auto).
#[ test ]
fn who_field_default_is_none()
{
  let cmd    = make_cmd( vec![] );
  let params = parse_usage_params( &cmd ).expect( "parse must succeed" );
  assert_eq!( params.who, None, "default who must be None (auto)" );
}

/// `who::1` parses to `params.who == Some(true)` (force on).
#[ test ]
fn who_field_parses_one_to_some_true()
{
  let cmd    = make_cmd( vec![ ( "who", Value::Integer( 1 ) ) ] );
  let params = parse_usage_params( &cmd ).expect( "parse must succeed" );
  assert_eq!( params.who, Some( true ), "who::1 must set params.who to Some(true)" );
}

/// `who::0` parses to `params.who == Some(false)` (suppress).
#[ test ]
fn who_field_parses_zero_to_some_false()
{
  let cmd    = make_cmd( vec![ ( "who", Value::Integer( 0 ) ) ] );
  let params = parse_usage_params( &cmd ).expect( "parse must succeed" );
  assert_eq!( params.who, Some( false ), "who::0 must set params.who to Some(false)" );
}

/// EC-7 — `who::2` is rejected; integer outside {0, 1} triggers `ArgumentTypeMismatch`.
///
/// `parse_int_flag` only succeeds for `Value::Integer(0)` and `Value::Integer(1)`.
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
#[ test ]
fn solo_field_default_false()
{
  let cmd    = make_cmd( vec![] );
  let params = parse_usage_params( &cmd ).expect( "parse must succeed" );
  assert!( !params.solo, "default solo must be false (token conservation off)" );
}

/// EC-2 subset: `solo::1` parses to `params.solo == true`.
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
/// `solo::1` restricts fetch to the current+owned account; `rotate::1` requires live
/// fetch for all accounts to compare winner candidates. Combining them is self-contradictory.
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

/// Mutual exclusion guard rejects `rotate::1 live::1`.
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
